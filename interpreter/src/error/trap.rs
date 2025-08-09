//! Call and create trap handler.

use alloc::vec::Vec;
use core::{
	cmp::{max, min},
	convert::Infallible,
};

use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

use crate::{
	error::{ExitError, ExitException, ExitFatal, ExitResult},
	machine::{AsMachineMut, Machine, Memory},
	runtime::{Context, RuntimeBackend, RuntimeState, Transfer},
	utils::{h256_to_u256, u256_to_h256, u256_to_usize},
};

pub trait Trap<I: ?Sized> {
	type Feedback;

	fn feedback(self, feedback: Self::Feedback, interpreter: &mut I) -> Result<(), ExitError>;
}

impl<I> Trap<I> for () {
	type Feedback = Infallible;

	fn feedback(self, feedback: Infallible, _interpreter: &mut I) -> Result<(), ExitError> {
		match feedback {}
	}
}

impl<I> Trap<I> for Infallible {
	type Feedback = Infallible;

	fn feedback(self, feedback: Infallible, _interpreter: &mut I) -> Result<(), ExitError> {
		match feedback {}
	}
}

pub trait TrapConsume<T> {
	type Rest;

	fn consume(self) -> Result<T, Self::Rest>;
}

impl<T> TrapConsume<T> for T {
	type Rest = Infallible;

	fn consume(self) -> Result<T, Infallible> {
		Ok(self)
	}
}

pub enum CallCreateOpcode {
	Call,
	CallCode,
	DelegateCall,
	StaticCall,
	Create,
	Create2,
}

/// Combined call create trap data.
#[derive(Debug)]
pub enum CallCreateTrap {
	/// A call trap data.
	Call(CallTrap),
	/// A create trap data.
	Create(CreateTrap),
}

#[derive(Debug)]
pub enum CallCreateFeedback {
	Call(CallFeedback),
	Create(CreateFeedback),
}

impl CallCreateTrap {
	#[must_use]
	pub const fn target_gas(&self) -> Option<U256> {
		match self {
			Self::Call(CallTrap { gas, .. }) => Some(*gas),
			Self::Create(_) => None,
		}
	}

	pub fn new_from<S: AsRef<RuntimeState> + AsMut<RuntimeState>>(
		opcode: CallCreateOpcode,
		machine: &mut Machine<S>,
	) -> Result<Self, ExitError> {
		match opcode {
			CallCreateOpcode::Create => Ok(Self::Create(CreateTrap::new_create_from(machine)?)),
			CallCreateOpcode::Create2 => Ok(Self::Create(CreateTrap::new_create2_from(machine)?)),
			CallCreateOpcode::Call => {
				Ok(Self::Call(CallTrap::new_from(CallScheme::Call, machine)?))
			}
			CallCreateOpcode::CallCode => Ok(Self::Call(CallTrap::new_from(
				CallScheme::CallCode,
				machine,
			)?)),
			CallCreateOpcode::DelegateCall => Ok(Self::Call(CallTrap::new_from(
				CallScheme::DelegateCall,
				machine,
			)?)),
			CallCreateOpcode::StaticCall => Ok(Self::Call(CallTrap::new_from(
				CallScheme::StaticCall,
				machine,
			)?)),
		}
	}

	pub fn code<H: RuntimeBackend>(&self, handler: &H) -> Vec<u8> {
		match self {
			Self::Call(trap) => handler.code(trap.target),
			Self::Create(trap) => trap.code.clone(),
		}
	}
}

impl<I: AsMachineMut> Trap<I> for CallCreateTrap
where
	I::State: AsRef<RuntimeState> + AsMut<RuntimeState>,
{
	type Feedback = CallCreateFeedback;

	fn feedback(self, feedback: CallCreateFeedback, interpreter: &mut I) -> Result<(), ExitError> {
		match (self, feedback) {
			(Self::Call(trap), CallCreateFeedback::Call(feedback)) => {
				trap.feedback(feedback, interpreter)
			}
			(Self::Create(trap), CallCreateFeedback::Create(feedback)) => {
				trap.feedback(feedback, interpreter)
			}
			_ => Err(ExitFatal::InvalidFeedback.into()),
		}
	}
}

/// Call scheme.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CallScheme {
	/// `CALL`
	Call,
	/// `CALLCODE`
	CallCode,
	/// `DELEGATECALL`
	DelegateCall,
	/// `STATICCALL`
	StaticCall,
}

#[derive(Debug)]
pub struct CallTrap {
	pub target: H160,
	pub transfer: Option<Transfer>,
	pub input: Vec<u8>,
	pub gas: U256,
	pub is_static: bool,
	pub out_offset: U256,
	pub out_len: U256,
	pub context: Context,
	pub scheme: CallScheme,
}

#[derive(Debug)]
pub struct CallFeedback {
	pub reason: ExitResult,
	pub retbuf: Vec<u8>,
}

impl CallTrap {
	#[allow(clippy::too_many_arguments)]
	fn new_from_params<S: AsRef<RuntimeState> + AsMut<RuntimeState>>(
		scheme: CallScheme,
		memory: &mut Memory,
		state: &mut S,
		gas: U256,
		to: H256,
		value: Option<U256>,
		in_offset: U256,
		in_len: U256,
		out_offset: U256,
		out_len: U256,
	) -> Result<((), Self), ExitError> {
		let value = value.unwrap_or(U256::zero());

		let in_end = in_offset
			.checked_add(in_len)
			.ok_or(ExitException::InvalidRange)?;
		let out_end = out_offset
			.checked_add(out_len)
			.ok_or(ExitException::InvalidRange)?;

		let in_offset_len = if in_len == U256::zero() {
			None
		} else {
			Some((u256_to_usize(in_offset)?, u256_to_usize(in_len)?))
		};

		memory.resize_end(max(in_end, out_end))?;

		let input = in_offset_len
			.map(|(in_offset, in_len)| memory.get(in_offset, in_len))
			.unwrap_or(Vec::new());

		let context = match scheme {
			CallScheme::Call | CallScheme::StaticCall => Context {
				address: to.into(),
				caller: state.as_ref().context.address,
				apparent_value: value,
			},
			CallScheme::CallCode => Context {
				address: state.as_ref().context.address,
				caller: state.as_ref().context.address,
				apparent_value: value,
			},
			CallScheme::DelegateCall => Context {
				address: state.as_ref().context.address,
				caller: state.as_ref().context.caller,
				apparent_value: state.as_ref().context.apparent_value,
			},
		};

		let transfer = if scheme == CallScheme::Call {
			Some(Transfer {
				source: state.as_ref().context.address,
				target: to.into(),
				value,
			})
		} else if scheme == CallScheme::CallCode {
			Some(Transfer {
				source: state.as_ref().context.address,
				target: state.as_ref().context.address,
				value,
			})
		} else {
			None
		};

		state.as_mut().retbuf = Vec::new();

		Ok((
			(),
			Self {
				target: to.into(),
				transfer,
				input,
				gas,
				is_static: scheme == CallScheme::StaticCall,
				context,
				out_offset,
				out_len,
				scheme,
			},
		))
	}

	pub fn new_from<S: AsRef<RuntimeState> + AsMut<RuntimeState>>(
		scheme: CallScheme,
		machine: &mut Machine<S>,
	) -> Result<Self, ExitError> {
		let stack = &mut machine.stack;
		let memory = &mut machine.memory;
		let state = &mut machine.state;

		match scheme {
			CallScheme::Call | CallScheme::CallCode => stack.perform_pop7_push0(
				|gas, to, value, in_offset, in_len, out_offset, out_len| {
					Self::new_from_params(
						scheme,
						memory,
						state,
						*gas,
						u256_to_h256(*to),
						Some(*value),
						*in_offset,
						*in_len,
						*out_offset,
						*out_len,
					)
				},
			),
			CallScheme::DelegateCall | CallScheme::StaticCall => {
				stack.perform_pop6_push0(|gas, to, in_offset, in_len, out_offset, out_len| {
					Self::new_from_params(
						scheme,
						memory,
						state,
						*gas,
						u256_to_h256(*to),
						None,
						*in_offset,
						*in_len,
						*out_offset,
						*out_len,
					)
				})
			}
		}
	}

	#[must_use]
	pub fn has_value(&self) -> bool {
		self.transfer
			.as_ref()
			.is_some_and(|t| t.value != U256::zero())
	}
}

impl<I: AsMachineMut> Trap<I> for CallTrap
where
	I::State: AsRef<RuntimeState> + AsMut<RuntimeState>,
{
	type Feedback = CallFeedback;

	fn feedback(self, feedback: CallFeedback, interpreter: &mut I) -> Result<(), ExitError> {
		let machine = interpreter.as_machine_mut();

		let reason = feedback.reason;
		let retbuf = feedback.retbuf;
		let target_len = min(self.out_len, U256::from(retbuf.len()));
		let out_offset = self.out_offset;

		let ret = match reason {
			Ok(_) => {
				match machine
					.memory
					.copy_large(out_offset, U256::zero(), target_len, &retbuf[..])
				{
					Ok(()) => {
						machine.stack.push(U256::one())?;

						Ok(())
					}
					Err(_) => {
						machine.stack.push(U256::zero())?;

						Ok(())
					}
				}
			}
			Err(ExitError::Reverted) => {
				machine.stack.push(U256::zero())?;

				let _ =
					machine
						.memory
						.copy_large(out_offset, U256::zero(), target_len, &retbuf[..]);

				Ok(())
			}
			Err(ExitError::Exception(_)) => {
				machine.stack.push(U256::zero())?;

				Ok(())
			}
			Err(ExitError::Fatal(e)) => {
				machine.stack.push(U256::zero())?;

				Err(e.into())
			}
		};

		match ret {
			Ok(()) => {
				machine.state.as_mut().retbuf = retbuf;
				Ok(())
			}
			Err(e) => Err(e),
		}
	}
}

/// Create scheme.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CreateScheme {
	/// Legacy create scheme of `CREATE`.
	Legacy {
		/// Caller of the create call.
		caller: H160,
	},
	/// Create scheme of `CREATE2`.
	Create2 {
		/// Caller of the create call.
		caller: H160,
		/// Code hash.
		code_hash: H256,
		/// Salt.
		salt: H256,
	},
}

impl CreateScheme {
	pub fn address<H: RuntimeBackend>(&self, handler: &H) -> H160 {
		match self {
			Self::Create2 {
				caller,
				code_hash,
				salt,
			} => {
				let mut hasher = Keccak256::new();
				hasher.update([0xff]);
				hasher.update(&caller[..]);
				hasher.update(&salt[..]);
				hasher.update(&code_hash[..]);
				H256::from_slice(hasher.finalize().as_slice()).into()
			}
			Self::Legacy { caller } => {
				let nonce = handler.nonce(*caller);
				let mut stream = rlp::RlpStream::new_list(2);
				stream.append(caller);
				stream.append(&nonce);
				H256::from_slice(Keccak256::digest(stream.out()).as_slice()).into()
			}
		}
	}

	#[must_use]
	pub const fn caller(&self) -> H160 {
		match self {
			Self::Create2 { caller, .. } => *caller,
			Self::Legacy { caller } => *caller,
		}
	}
}

#[derive(Debug)]
pub struct CreateTrap {
	pub scheme: CreateScheme,
	pub value: U256,
	pub code: Vec<u8>,
}

#[derive(Debug)]
pub struct CreateFeedback {
	pub reason: Result<H160, ExitError>,
	pub retbuf: Vec<u8>,
}

impl CreateTrap {
	pub fn new_create_from<S: AsRef<RuntimeState> + AsMut<RuntimeState>>(
		machine: &mut Machine<S>,
	) -> Result<Self, ExitError> {
		let stack = &mut machine.stack;
		let memory = &mut machine.memory;
		let state = &mut machine.state;

		stack.perform_pop3_push0(|value, code_offset, code_len| {
			let code_end = code_offset
				.checked_add(*code_len)
				.ok_or(ExitException::InvalidRange)?;

			let code_offset_len = if code_len == &U256::zero() {
				None
			} else {
				Some((u256_to_usize(*code_offset)?, u256_to_usize(*code_len)?))
			};

			memory.resize_end(code_end)?;

			let code = code_offset_len
				.map(|(code_offset, code_len)| memory.get(code_offset, code_len))
				.unwrap_or(Vec::new());

			let scheme = CreateScheme::Legacy {
				caller: state.as_ref().context.address,
			};

			state.as_mut().retbuf = Vec::new();

			Ok((
				(),
				Self {
					scheme,
					value: *value,
					code,
				},
			))
		})
	}

	pub fn new_create2_from<S: AsRef<RuntimeState> + AsMut<RuntimeState>>(
		machine: &mut Machine<S>,
	) -> Result<Self, ExitError> {
		let stack = &mut machine.stack;
		let memory = &mut machine.memory;
		let state = &mut machine.state;

		stack.perform_pop4_push0(|value, code_offset, code_len, salt| {
			let code_end = code_offset
				.checked_add(*code_len)
				.ok_or(ExitException::InvalidRange)?;

			let code_offset_len = if code_len == &U256::zero() {
				None
			} else {
				Some((u256_to_usize(*code_offset)?, u256_to_usize(*code_len)?))
			};

			memory.resize_end(code_end)?;

			let code = code_offset_len
				.map(|(code_offset, code_len)| memory.get(code_offset, code_len))
				.unwrap_or(Vec::new());

			let code_hash = H256::from_slice(Keccak256::digest(&code).as_slice());

			let scheme = CreateScheme::Create2 {
				caller: state.as_ref().context.address,
				salt: u256_to_h256(*salt),
				code_hash,
			};

			state.as_mut().retbuf = Vec::new();

			Ok((
				(),
				Self {
					scheme,
					value: *value,
					code,
				},
			))
		})
	}
}

impl<I: AsMachineMut> Trap<I> for CreateTrap
where
	I::State: AsRef<RuntimeState> + AsMut<RuntimeState>,
{
	type Feedback = CreateFeedback;

	fn feedback(self, feedback: CreateFeedback, interpreter: &mut I) -> Result<(), ExitError> {
		let machine = interpreter.as_machine_mut();

		let reason = feedback.reason;
		let retbuf = feedback.retbuf;

		let ret = match reason {
			Ok(address) => {
				machine.stack.push(h256_to_u256(address.into()))?;
				Ok(())
			}
			Err(ExitError::Reverted) => {
				machine.stack.push(U256::zero())?;
				Ok(())
			}
			Err(ExitError::Exception(_)) => {
				machine.stack.push(U256::zero())?;
				Ok(())
			}
			Err(ExitError::Fatal(e)) => {
				machine.stack.push(U256::zero())?;
				Err(e.into())
			}
		};

		match ret {
			Ok(()) => {
				machine.state.as_mut().retbuf = retbuf;
				Ok(())
			}
			Err(e) => Err(e),
		}
	}
}
