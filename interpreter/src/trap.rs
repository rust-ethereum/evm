//! Call and create trap handler.

use alloc::vec::Vec;
use core::{cmp::min, convert::Infallible};
use sha3::{Digest, Keccak256};

#[allow(unused_imports)]
use crate::uint::{H160, H256, U256, U256Ext};
use crate::{
	error::{ExitError, ExitException, ExitResult},
	machine::{Machine, Memory},
	runtime::{Context, RuntimeBackend, RuntimeState, Transfer},
	utils::u256_to_usize,
};

/// Consume `T` to get `Rest`.
///
/// For example, an interpreter may return two types of traps, a [CallCreateTrap], and another customized trap.
/// The standard invoker, however, can only handle [CallCreateTrap]. By implementing this trait, the standard
/// invoker can handle just the [CallCreateTrap], and then it returns `Rest` as an additional interrupt that
/// is handled by the user.
pub trait TrapConsume<T> {
	/// Rest type.
	type Rest;

	/// Consume `T` to get `Rest`.
	fn consume(self) -> Result<T, Self::Rest>;
}

impl<T> TrapConsume<T> for T {
	type Rest = Infallible;

	fn consume(self) -> Result<T, Infallible> {
		Ok(self)
	}
}

/// Call create opcodes.
pub enum CallCreateOpcode {
	/// `CALL`
	Call,
	/// `CALLCODE`
	CallCode,
	/// `DELEGATECALL`
	DelegateCall,
	/// `STATICCALL`
	StaticCall,
	/// `CREATE`
	Create,
	/// `CREATE2`
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

impl CallCreateTrap {
	/// Target gas value.
	#[must_use]
	pub const fn target_gas(&self) -> Option<U256> {
		match self {
			Self::Call(CallTrap { gas, .. }) => Some(*gas),
			Self::Create(_) => None,
		}
	}

	/// Create a new trap from the given opcode and the machine state.
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

	/// Target code or init code.
	pub fn code<H: RuntimeBackend>(&self, handler: &H) -> Vec<u8> {
		match self {
			Self::Call(trap) => handler.code(trap.target),
			Self::Create(trap) => trap.code.clone(),
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

/// Trap for a call.
#[derive(Debug)]
pub struct CallTrap {
	/// Call target.
	pub target: H160,
	/// Transfer instruction, if any.
	pub transfer: Option<Transfer>,
	/// Input data.
	pub input: Vec<u8>,
	/// Gas.
	pub gas: U256,
	/// Whether it is `STATICCALL`
	pub is_static: bool,
	/// Out value offset.
	pub out_offset: U256,
	/// Out value length.
	pub out_len: U256,
	/// Call context.
	pub context: Context,
	/// Call scheme.
	pub scheme: CallScheme,
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
		let value = value.unwrap_or(U256::ZERO);

		let in_end = in_offset
			.checked_add(in_len)
			.ok_or(ExitException::InvalidRange)?;
		if in_len != U256::ZERO {
			memory.resize_end(in_end)?;
		}
		let out_end = out_offset
			.checked_add(out_len)
			.ok_or(ExitException::InvalidRange)?;
		if out_len != U256::ZERO {
			memory.resize_end(out_end)?;
		}

		let in_offset_len = if in_len == U256::ZERO {
			None
		} else {
			Some((u256_to_usize(in_offset)?, u256_to_usize(in_len)?))
		};

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

	/// Create a new call trap from the given call scheme and the machine state.
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
						to.to_h256(),
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
						to.to_h256(),
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

	/// Whether the call has value.
	#[must_use]
	pub fn has_value(&self) -> bool {
		self.transfer
			.as_ref()
			.is_some_and(|t| t.value != U256::ZERO)
	}
}

/// Feedback value of a call trap.
#[derive(Debug)]
pub struct CallFeedback {
	/// The original call trap.
	pub trap: CallTrap,
	/// Exit reason.
	pub reason: ExitResult,
	/// Return value.
	pub retbuf: Vec<u8>,
}

impl CallFeedback {
	/// Apply the call feedback into a machine state.
	pub fn to_machine<
		S: AsRef<RuntimeState> + AsMut<RuntimeState>,
		I: AsRef<Machine<S>> + AsMut<Machine<S>>,
	>(
		self,
		interpreter: &mut I,
	) -> Result<(), ExitError> {
		let machine: &mut Machine<S> = interpreter.as_mut();

		let reason = self.reason;
		let retbuf = self.retbuf;
		let target_len = min(self.trap.out_len, U256::from_usize(retbuf.len()));
		let out_offset = self.trap.out_offset;

		let ret = match reason {
			Ok(_) => {
				match machine
					.memory
					.copy_large(out_offset, U256::ZERO, target_len, &retbuf[..])
				{
					Ok(()) => {
						machine.stack.push(U256::ONE)?;

						Ok(())
					}
					Err(_) => {
						machine.stack.push(U256::ZERO)?;

						Ok(())
					}
				}
			}
			Err(ExitError::Reverted) => {
				machine.stack.push(U256::ZERO)?;

				let _ =
					machine
						.memory
						.copy_large(out_offset, U256::ZERO, target_len, &retbuf[..]);

				Ok(())
			}
			Err(ExitError::Exception(_)) => {
				machine.stack.push(U256::ZERO)?;

				Ok(())
			}
			Err(ExitError::Fatal(e)) => {
				machine.stack.push(U256::ZERO)?;

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
	/// Resolved address.
	#[allow(deprecated)]
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
				nonce.append_to_rlp_stream(&mut stream);
				H256::from_slice(Keccak256::digest(stream.out()).as_slice()).into()
			}
		}
	}

	/// Caller address.
	#[must_use]
	pub const fn caller(&self) -> H160 {
		match self {
			Self::Create2 { caller, .. } => *caller,
			Self::Legacy { caller } => *caller,
		}
	}
}

/// Call trap.
#[derive(Debug)]
pub struct CreateTrap {
	/// Call scheme.
	pub scheme: CreateScheme,
	/// Value passed to the call.
	pub value: U256,
	/// Init code.
	pub code: Vec<u8>,
}

impl CreateTrap {
	/// Create a new `CREATE` trap from the machine state.
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

			let code_offset_len = if code_len == &U256::ZERO {
				None
			} else {
				Some((u256_to_usize(*code_offset)?, u256_to_usize(*code_len)?))
			};

			if *code_len != U256::ZERO {
				memory.resize_end(code_end)?;
			}

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

	/// Create a new `CREATE2` trap from the machine state.
	#[allow(deprecated)]
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

			let code_offset_len = if code_len == &U256::ZERO {
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
				salt: salt.to_h256(),
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

/// Feedback value of the create trap.
#[derive(Debug)]
pub struct CreateFeedback {
	/// Original create trap.
	pub trap: CreateTrap,
	/// Exit reason and new contract address.
	pub reason: Result<H160, ExitError>,
	/// Return value.
	pub retbuf: Vec<u8>,
}

impl CreateFeedback {
	/// Apply the trap feedback to the machine state.
	pub fn to_machine<
		S: AsRef<RuntimeState> + AsMut<RuntimeState>,
		I: AsRef<Machine<S>> + AsMut<Machine<S>>,
	>(
		self,
		interpreter: &mut I,
	) -> Result<(), ExitError> {
		let machine: &mut Machine<S> = interpreter.as_mut();

		let reason = self.reason;
		let retbuf = self.retbuf;

		let ret = match reason {
			Ok(address) => {
				machine.stack.push(U256::from_h160(address))?;
				Ok(())
			}
			Err(ExitError::Reverted) => {
				machine.stack.push(U256::ZERO)?;
				Ok(())
			}
			Err(ExitError::Exception(_)) => {
				machine.stack.push(U256::ZERO)?;
				Ok(())
			}
			Err(ExitError::Fatal(e)) => {
				machine.stack.push(U256::ZERO)?;
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
