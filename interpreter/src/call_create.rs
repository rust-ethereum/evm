//! Call and create trap handler.

use crate::utils::{h256_to_u256, u256_to_usize};
use crate::{
	Context, ExitError, ExitException, ExitResult, Machine, Memory, RuntimeBackend, RuntimeState,
	Transfer, TrapConstruct, TrapConsume,
};
use alloc::vec::Vec;
use core::cmp::{max, min};
use core::convert::Infallible;
use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

/// Create scheme.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CreateScheme {
	/// Legacy create scheme of `CREATE`.
	Legacy {
		/// Caller of the create.
		caller: H160,
	},
	/// Create scheme of `CREATE2`.
	Create2 {
		/// Caller of the create.
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
				H256::from_slice(Keccak256::digest(&stream.out()).as_slice()).into()
			}
		}
	}

	pub const fn caller(&self) -> H160 {
		match self {
			Self::Create2 { caller, .. } => *caller,
			Self::Legacy { caller } => *caller,
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

pub enum CallCreateTrap {
	Create,
	Create2,
	Call,
	CallCode,
	DelegateCall,
	StaticCall,
}

impl TrapConstruct<CallCreateTrap> for CallCreateTrap {
	fn construct(v: CallCreateTrap) -> Self {
		v
	}
}

impl TrapConsume<CallCreateTrap> for CallCreateTrap {
	type Rest = Infallible;

	fn consume(self) -> Result<CallCreateTrap, Infallible> {
		Ok(self)
	}
}

/// Combined call create trap data.
pub enum CallCreateTrapData {
	/// A call trap data.
	Call(CallTrapData),
	/// A create trap data.
	Create(CreateTrapData),
}

impl CallCreateTrapData {
	pub const fn target_gas(&self) -> Option<U256> {
		match self {
			Self::Call(CallTrapData { gas, .. }) => Some(*gas),
			Self::Create(_) => None,
		}
	}

	pub fn new_from<S: AsRef<RuntimeState> + AsMut<RuntimeState>>(
		opcode: CallCreateTrap,
		machine: &mut Machine<S>,
	) -> Result<Self, ExitError> {
		match opcode {
			CallCreateTrap::Create => Ok(Self::Create(CreateTrapData::new_create_from(machine)?)),
			CallCreateTrap::Create2 => Ok(Self::Create(CreateTrapData::new_create2_from(machine)?)),
			CallCreateTrap::Call => Ok(Self::Call(CallTrapData::new_from(
				CallScheme::Call,
				machine,
			)?)),
			CallCreateTrap::CallCode => Ok(Self::Call(CallTrapData::new_from(
				CallScheme::CallCode,
				machine,
			)?)),
			CallCreateTrap::DelegateCall => Ok(Self::Call(CallTrapData::new_from(
				CallScheme::DelegateCall,
				machine,
			)?)),
			CallCreateTrap::StaticCall => Ok(Self::Call(CallTrapData::new_from(
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

pub struct CallTrapData {
	pub target: H160,
	pub transfer: Option<Transfer>,
	pub input: Vec<u8>,
	pub gas: U256,
	pub is_static: bool,
	pub out_offset: U256,
	pub out_len: U256,
	pub context: Context,
}

impl CallTrapData {
	#[allow(clippy::too_many_arguments)]
	fn new_from_params<S: AsRef<RuntimeState> + AsMut<RuntimeState>>(
		scheme: CallScheme,
		memory: &mut Memory,
		state: &mut S,
		gas: &H256,
		to: &H256,
		value: Option<&H256>,
		in_offset: &H256,
		in_len: &H256,
		out_offset: &H256,
		out_len: &H256,
	) -> Result<((), Self), ExitError> {
		let gas = h256_to_u256(*gas);
		let value = value.map(|v| h256_to_u256(*v)).unwrap_or(U256::zero());
		let in_offset = h256_to_u256(*in_offset);
		let in_len = h256_to_u256(*in_len);
		let out_offset = h256_to_u256(*out_offset);
		let out_len = h256_to_u256(*out_len);

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
				address: (*to).into(),
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
				target: (*to).into(),
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
				target: (*to).into(),
				transfer,
				input,
				gas,
				is_static: scheme == CallScheme::StaticCall,
				context,
				out_offset,
				out_len,
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
						gas,
						to,
						Some(value),
						in_offset,
						in_len,
						out_offset,
						out_len,
					)
				},
			),
			CallScheme::DelegateCall | CallScheme::StaticCall => {
				stack.perform_pop6_push0(|gas, to, in_offset, in_len, out_offset, out_len| {
					Self::new_from_params(
						scheme, memory, state, gas, to, None, in_offset, in_len, out_offset,
						out_len,
					)
				})
			}
		}
	}

	pub fn feedback<S: AsRef<RuntimeState> + AsMut<RuntimeState>>(
		self,
		reason: ExitResult,
		retbuf: Vec<u8>,
		machine: &mut Machine<S>,
	) -> Result<(), ExitError> {
		let target_len = min(self.out_len, U256::from(retbuf.len()));
		let out_offset = self.out_offset;

		let ret = match reason {
			Ok(_) => {
				match machine
					.memory
					.copy_large(out_offset, U256::zero(), target_len, &retbuf[..])
				{
					Ok(()) => {
						let mut value = H256::default();
						U256::one().to_big_endian(&mut value[..]);
						machine.stack.push(value)?;

						Ok(())
					}
					Err(_) => {
						machine.stack.push(H256::default())?;

						Ok(())
					}
				}
			}
			Err(ExitError::Reverted) => {
				machine.stack.push(H256::default())?;

				let _ =
					machine
						.memory
						.copy_large(out_offset, U256::zero(), target_len, &retbuf[..]);

				Ok(())
			}
			Err(ExitError::Exception(_)) => {
				machine.stack.push(H256::default())?;

				Ok(())
			}
			Err(ExitError::Fatal(e)) => {
				machine.stack.push(H256::default())?;

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

	pub fn has_value(&self) -> bool {
		self.transfer
			.as_ref()
			.map(|t| t.value != U256::zero())
			.unwrap_or(false)
	}
}

#[derive(Clone, Debug)]
pub struct CreateTrapData {
	pub scheme: CreateScheme,
	pub value: U256,
	pub code: Vec<u8>,
}

impl CreateTrapData {
	pub fn new_create_from<S: AsRef<RuntimeState> + AsMut<RuntimeState>>(
		machine: &mut Machine<S>,
	) -> Result<Self, ExitError> {
		let stack = &mut machine.stack;
		let memory = &mut machine.memory;
		let state = &mut machine.state;

		stack.perform_pop3_push0(|value, code_offset, code_len| {
			let value = h256_to_u256(*value);
			let code_offset = h256_to_u256(*code_offset);
			let code_len = h256_to_u256(*code_len);

			let code_offset_len = if code_len == U256::zero() {
				None
			} else {
				Some((u256_to_usize(code_offset)?, u256_to_usize(code_len)?))
			};

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
					value,
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
			let value = h256_to_u256(*value);
			let code_offset = h256_to_u256(*code_offset);
			let code_len = h256_to_u256(*code_len);

			let code_offset_len = if code_len == U256::zero() {
				None
			} else {
				Some((u256_to_usize(code_offset)?, u256_to_usize(code_len)?))
			};

			let code = code_offset_len
				.map(|(code_offset, code_len)| memory.get(code_offset, code_len))
				.unwrap_or(Vec::new());

			let code_hash = H256::from_slice(Keccak256::digest(&code).as_slice());

			let scheme = CreateScheme::Create2 {
				caller: state.as_ref().context.address,
				salt: *salt,
				code_hash,
			};

			state.as_mut().retbuf = Vec::new();

			Ok((
				(),
				Self {
					scheme,
					value,
					code,
				},
			))
		})
	}

	pub fn feedback<S: AsRef<RuntimeState> + AsMut<RuntimeState>>(
		self,
		reason: Result<H160, ExitError>,
		retbuf: Vec<u8>,
		machine: &mut Machine<S>,
	) -> Result<(), ExitError> {
		let ret = match reason {
			Ok(address) => {
				machine.stack.push(address.into())?;
				Ok(())
			}
			Err(ExitError::Reverted) => {
				machine.stack.push(H256::default())?;
				Ok(())
			}
			Err(ExitError::Exception(_)) => {
				machine.stack.push(H256::default())?;
				Ok(())
			}
			Err(ExitError::Fatal(e)) => {
				machine.stack.push(H256::default())?;
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
