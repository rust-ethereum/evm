#![cfg_attr(not(feature = "std"), no_std)]

mod consts;
mod costs;
mod memory;
mod utils;

use core::cmp::max;
use primitive_types::{H160, H256, U256};
use evm_core::{ExternalOpcode, Opcode, ExitError, Stack};
use evm_runtime::Handler;

#[derive(Clone)]
pub struct Gasometer<'config> {
	gas_limit: usize,
	config: &'config Config,
	inner: Result<Inner<'config>, ExitError>
}

impl<'config> Gasometer<'config> {
	pub fn new(gas_limit: usize, config: &'config Config) -> Self {
		Self {
			gas_limit,
			config,
			inner: Ok(Inner {
				memory_cost: 0,
				used_gas: 0,
				refunded_gas: 0,
				config,
			}),
		}
	}

	fn inner_mut(
		&mut self
	) -> Result<&mut Inner<'config>, ExitError> {
		self.inner.as_mut().map_err(|e| *e)
	}

	pub fn merge<'oconfig>(&mut self, other: Gasometer<'oconfig>) -> Result<(), ExitError> {
		let other_refunded_gas = other.refunded_gas();
		let other_total_used_gas = other.total_used_gas();

		let all_gas_cost = self.total_used_gas() + other_total_used_gas;
		if self.gas_limit < all_gas_cost {
			self.inner = Err(ExitError::OutOfGas);
			return Err(ExitError::OutOfGas)
		}

		self.inner_mut()?.used_gas += other_total_used_gas;
		self.inner_mut()?.refunded_gas += other_refunded_gas;

		Ok(())
	}

	pub fn config(&self) -> &'config Config {
		self.config
	}

	pub fn gas(&self) -> usize {
		match self.inner.as_ref() {
			Ok(inner) => {
				self.gas_limit - inner.used_gas -
					memory::memory_gas(inner.memory_cost).expect("Checked via record")
			},
			Err(_) => 0,
		}
	}

	pub fn total_used_gas(&self) -> usize {
		match self.inner.as_ref() {
			Ok(inner) => inner.used_gas +
				memory::memory_gas(inner.memory_cost).expect("Checked via record"),
			Err(_) => self.gas_limit,
		}
	}

	pub fn refunded_gas(&self) -> isize {
		match self.inner.as_ref() {
			Ok(inner) => inner.refunded_gas,
			Err(_) => 0,
		}
	}

	pub fn record(
		&mut self,
		cost: GasCost,
		memory: Option<MemoryCost>,
	) -> Result<(), ExitError> {
		macro_rules! try_or_fail {
			( $e:expr ) => (
				match $e {
					Ok(value) => value,
					Err(e) => {
						self.inner = Err(e);
						return Err(e)
					},
				}
			)
		}

		let memory_cost = match memory {
			Some(memory) => try_or_fail!(self.inner_mut()?.memory_cost(memory)),
			None => self.inner_mut()?.memory_cost,
		};
		let memory_gas = try_or_fail!(memory::memory_gas(memory_cost));
		let gas_cost = try_or_fail!(self.inner_mut()?.gas_cost(cost.clone()));
		let gas_stipend = self.inner_mut()?.gas_stipend(cost.clone());
		let gas_refund = self.inner_mut()?.gas_refund(cost.clone());
		let used_gas = self.inner_mut()?.used_gas;

		let all_gas_cost = memory_gas + used_gas + gas_cost;
		if self.gas_limit < all_gas_cost {
			self.inner = Err(ExitError::OutOfGas);
			return Err(ExitError::OutOfGas)
		}

		let after_gas = self.gas_limit - all_gas_cost;
		try_or_fail!(self.inner_mut()?.extra_check(cost, after_gas));

		self.inner_mut()?.used_gas += gas_cost - gas_stipend;
		self.inner_mut()?.memory_cost = memory_cost;
		self.inner_mut()?.refunded_gas += gas_refund;

		Ok(())
	}
}

pub fn cost<H: Handler>(
	address: H160,
	opcode: Result<Opcode, ExternalOpcode>,
	stack: &Stack,
	handler: &H
) -> Result<(GasCost, Option<MemoryCost>), ExitError> {
	let gas_cost = match opcode {
		Ok(Opcode::Stop) | Ok(Opcode::Return) | Ok(Opcode::Revert) => GasCost::Zero,

		Err(ExternalOpcode::Address) | Err(ExternalOpcode::Origin) | Err(ExternalOpcode::Caller) |
		Err(ExternalOpcode::CallValue) | Ok(Opcode::CallDataSize) |
		Err(ExternalOpcode::ReturnDataSize) |
		Ok(Opcode::CodeSize) | Err(ExternalOpcode::GasPrice) | Err(ExternalOpcode::Coinbase) |
		Err(ExternalOpcode::Timestamp) | Err(ExternalOpcode::Number) |
		Err(ExternalOpcode::Difficulty) |
		Err(ExternalOpcode::GasLimit) | Ok(Opcode::Pop) | Ok(Opcode::PC) |
		Ok(Opcode::MSize) | Err(ExternalOpcode::Gas) => GasCost::Base,

		Ok(Opcode::Add) | Ok(Opcode::Sub) | Ok(Opcode::Not) | Ok(Opcode::Lt) |
		Ok(Opcode::Gt) | Ok(Opcode::SLt) | Ok(Opcode::SGt) | Ok(Opcode::Eq) |
		Ok(Opcode::IsZero) | Ok(Opcode::And) | Ok(Opcode::Or) | Ok(Opcode::Xor) |
		Ok(Opcode::Byte) | Ok(Opcode::CallDataLoad) | Ok(Opcode::MLoad) |
		Ok(Opcode::MStore) | Ok(Opcode::MStore8) | Ok(Opcode::Push(_)) |
		Ok(Opcode::Dup(_)) | Ok(Opcode::Swap(_)) | Ok(Opcode::Shl) | Ok(Opcode::Shr) |
		Ok(Opcode::Sar) => GasCost::VeryLow,

		Ok(Opcode::Mul) | Ok(Opcode::Div) | Ok(Opcode::SDiv) | Ok(Opcode::Mod) |
		Ok(Opcode::SMod) | Ok(Opcode::SignExtend) => GasCost::Low,

		Ok(Opcode::AddMod) | Ok(Opcode::MulMod) | Ok(Opcode::Jump) => GasCost::Mid,

		Ok(Opcode::JumpI) => GasCost::High,

		Err(ExternalOpcode::ExtCodeSize) => GasCost::ExtCodeSize,
		Err(ExternalOpcode::Balance) => GasCost::Balance,
		Err(ExternalOpcode::BlockHash) => GasCost::BlockHash,
		Err(ExternalOpcode::ExtCodeHash) => GasCost::ExtCodeHash,

		Err(ExternalOpcode::Call) => GasCost::Call {
			value: U256::from_big_endian(&stack.peek(2)?[..]),
			gas: U256::from_big_endian(&stack.peek(0)?[..]),
			target_exists: handler.exists(stack.peek(1)?.into()),
		},
		Err(ExternalOpcode::CallCode) => GasCost::CallCode {
			value: U256::from_big_endian(&stack.peek(2)?[..]),
			gas: U256::from_big_endian(&stack.peek(0)?[..]),
			target_exists: handler.exists(stack.peek(1)?.into()),
		},
		Err(ExternalOpcode::DelegateCall) => GasCost::DelegateCall {
			gas: U256::from_big_endian(&stack.peek(0)?[..]),
			target_exists: handler.exists(stack.peek(1)?.into()),
		},
		Err(ExternalOpcode::StaticCall) => GasCost::StaticCall {
			gas: U256::from_big_endian(&stack.peek(0)?[..]),
			target_exists: handler.exists(stack.peek(1)?.into()),
		},
		Err(ExternalOpcode::Suicide) => GasCost::Suicide {
			value: handler.balance(address),
			target_exists: handler.exists(stack.peek(0)?.into()),
			already_removed: handler.deleted(address),
		},
		Err(ExternalOpcode::SStore) => {
			let index = stack.peek(0)?;
			let value = stack.peek(1)?;

			GasCost::SStore {
				original: handler.original_storage(address, index),
				current: handler.storage(address, index),
				new: value,
			}
		},
		Err(ExternalOpcode::Sha3) => GasCost::Sha3 {
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Err(ExternalOpcode::Log(n)) => GasCost::Log {
			n,
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Err(ExternalOpcode::ExtCodeCopy) => GasCost::ExtCodeCopy {
			len: U256::from_big_endian(&stack.peek(3)?[..]),
		},
		Ok(Opcode::CallDataCopy) | Ok(Opcode::CodeCopy) |
		Err(ExternalOpcode::ReturnDataCopy) => GasCost::VeryLowCopy {
			len: U256::from_big_endian(&stack.peek(2)?[..]),
		},
		Ok(Opcode::Exp) => GasCost::Exp {
			power: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Err(ExternalOpcode::Create) => GasCost::Create,
		Err(ExternalOpcode::Create2) => GasCost::Create2 {
			len: U256::from_big_endian(&stack.peek(2)?[..]),
		},
		Ok(Opcode::JumpDest) => GasCost::JumpDest,
		Err(ExternalOpcode::SLoad) => GasCost::SLoad,

		Ok(Opcode::Invalid) => GasCost::Invalid,
		Err(ExternalOpcode::Other(_)) => GasCost::Invalid,
	};

	let memory_cost = match opcode {
		Err(ExternalOpcode::Sha3) | Ok(Opcode::Return) | Ok(Opcode::Revert) |
		Err(ExternalOpcode::Log(_)) => Some(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(0)?[..]),
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		}),

		Ok(Opcode::CodeCopy) | Ok(Opcode::CallDataCopy) |
		Err(ExternalOpcode::ReturnDataCopy) => Some(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(0)?[..]),
			len: U256::from_big_endian(&stack.peek(2)?[..]),
		}),

		Err(ExternalOpcode::ExtCodeCopy) => Some(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(1)?[..]),
			len: U256::from_big_endian(&stack.peek(3)?[..]),
		}),

		Ok(Opcode::MLoad) | Ok(Opcode::MStore) => Some(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(0)?[..]),
			len: U256::from(32),
		}),

		Ok(Opcode::MStore8) => Some(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(0)?[..]),
			len: U256::from(1),
		}),

		Err(ExternalOpcode::Create) | Err(ExternalOpcode::Create2) => Some(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(1)?[..]),
			len: U256::from_big_endian(&stack.peek(2)?[..]),
		}),

		Err(ExternalOpcode::Call) | Err(ExternalOpcode::CallCode) => Some(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(3)?[..]),
			len: U256::from_big_endian(&stack.peek(4)?[..]),
		}.join(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(5)?[..]),
			len: U256::from_big_endian(&stack.peek(6)?[..]),
		})),

		Err(ExternalOpcode::DelegateCall) |
		Err(ExternalOpcode::StaticCall) => Some(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(2)?[..]),
			len: U256::from_big_endian(&stack.peek(3)?[..]),
		}.join(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(4)?[..]),
			len: U256::from_big_endian(&stack.peek(5)?[..]),
		})),

		_ => None,
	};

	Ok((gas_cost, memory_cost))
}

#[derive(Clone)]
struct Inner<'config> {
	memory_cost: usize,
	used_gas: usize,
	refunded_gas: isize,
	config: &'config Config,
}

impl<'config> Inner<'config> {
	fn memory_cost(
		&self,
		memory: MemoryCost,
	) -> Result<usize, ExitError> {
		let from = memory.offset;
		let len = memory.len;

		if len == U256::zero() {
			return Ok(self.memory_cost)
		}

		let end = from.checked_add(len).ok_or(ExitError::OutOfGas)?;

		if end > U256::from(usize::max_value()) {
			return Err(ExitError::OutOfGas)
		}
		let end = end.as_usize();

		let rem = end % 32;
		let new = if rem == 0 {
			end / 32
		} else {
			end / 32 + 1
		};

		Ok(max(self.memory_cost, new))
	}

	fn extra_check(
		&self,
		cost: GasCost,
		after_gas: usize,
	) -> Result<(), ExitError> {
		match cost {
			GasCost::Call { gas, .. } => costs::call_extra_check(gas, after_gas, self.config),
			GasCost::CallCode { gas, .. } => costs::call_extra_check(gas, after_gas, self.config),
			GasCost::DelegateCall { gas, .. } => costs::call_extra_check(gas, after_gas, self.config),
			GasCost::StaticCall { gas, .. } => costs::call_extra_check(gas, after_gas, self.config),
			_ => Ok(()),
		}
	}

	fn gas_cost(
		&self,
		cost: GasCost,
	) -> Result<usize, ExitError> {
		Ok(match cost {
			GasCost::Call { value, target_exists, .. } =>
				costs::call_cost(value, true, true, !target_exists, self.config),
			GasCost::CallCode { value, target_exists, .. } =>
				costs::call_cost(value, true, false, !target_exists, self.config),
			GasCost::DelegateCall { target_exists, .. } =>
				costs::call_cost(U256::zero(), false, false, !target_exists, self.config),
			GasCost::StaticCall { target_exists, .. } =>
				costs::call_cost(U256::zero(), false, true, !target_exists, self.config),
			GasCost::Suicide { value, target_exists, .. } =>
				costs::suicide_cost(value, target_exists, self.config),
			GasCost::SStore { original, current, new } =>
				costs::sstore_cost(original, current, new, self.config),

			GasCost::Sha3 { len } => costs::sha3_cost(len)?,
			GasCost::Log { n, len } => costs::log_cost(n, len)?,
			GasCost::ExtCodeCopy { len } => costs::extcodecopy_cost(len, self.config)?,
			GasCost::VeryLowCopy { len } => costs::verylowcopy_cost(len)?,
			GasCost::Exp { power } => costs::exp_cost(power, self.config)?,
			GasCost::Create => consts::G_CREATE,
			GasCost::Create2 { len } => costs::create2_cost(len)?,
			GasCost::JumpDest => consts::G_JUMPDEST,
			GasCost::SLoad => self.config.gas_sload,

			GasCost::Zero => consts::G_ZERO,
			GasCost::Base => consts::G_BASE,
			GasCost::VeryLow => consts::G_VERYLOW,
			GasCost::Low => consts::G_LOW,
			GasCost::Mid => consts::G_MID,
			GasCost::High => consts::G_HIGH,
			GasCost::Invalid => return Err(ExitError::OutOfGas),

			GasCost::ExtCodeSize => self.config.gas_extcode,
			GasCost::Balance => self.config.gas_balance,
			GasCost::BlockHash => consts::G_BLOCKHASH,
			GasCost::ExtCodeHash => consts::G_EXTCODEHASH,
		})
	}

	fn gas_stipend(
		&self,
		cost: GasCost
	) -> usize {
		match cost {
			GasCost::Call { value, .. } => costs::call_callcode_stipend(value),
			GasCost::CallCode { value, .. } => costs::call_callcode_stipend(value),
			_ => 0,
		}
	}

	fn gas_refund(
		&self,
		cost: GasCost
	) -> isize {
		match cost {
			GasCost::SStore { original, current, new } =>
				costs::sstore_refund(original, current, new, self.config),
			GasCost::Suicide { already_removed, .. } =>
				costs::suicide_refund(already_removed),
			_ => 0,
		}
	}
}

pub struct Config {
	/// Gas paid for extcode.
	pub gas_extcode: usize,
	/// Gas paid for BALANCE opcode.
	pub gas_balance: usize,
	/// Gas paid for SLOAD opcode.
	pub gas_sload: usize,
	/// Gas paid for SUICIDE opcode.
	pub gas_suicide: usize,
	/// Gas paid for SUICIDE opcode when it hits a new account.
	pub gas_suicide_new_account: usize,
	/// Gas paid for CALL opcode.
	pub gas_call: usize,
	/// Gas paid for EXP opcode for every byte.
	pub gas_expbyte: usize,
	/// Gas paid for a contract creation transaction.
	pub gas_transaction_create: usize,
	pub has_reduced_sstore_gas_metering: bool,
	/// Whether to throw out of gas error when
	/// CALL/CALLCODE/DELEGATECALL requires more than maximum amount
	/// of gas.
	pub err_on_call_with_more_gas: bool,
	/// Whether empty account is considered exists.
	pub empty_considered_exists: bool,
}

impl Config {
	pub const fn frontier() -> Config {
		Config {
			gas_extcode: 20,
			gas_balance: 20,
			gas_sload: 50,
			gas_suicide: 0,
			gas_suicide_new_account: 0,
			gas_call: 40,
			gas_expbyte: 10,
			gas_transaction_create: 0,
			has_reduced_sstore_gas_metering: false,
			err_on_call_with_more_gas: true,
			empty_considered_exists: true,
		}
	}
}

#[derive(Clone)]
pub enum GasCost {
	Zero,
	Base,
	VeryLow,
	Low,
	Mid,
	High,
	Invalid,

	ExtCodeSize,
	Balance,
	BlockHash,
	ExtCodeHash,

	Call { value: U256, gas: U256, target_exists: bool },
	CallCode { value: U256, gas: U256, target_exists: bool },
	DelegateCall { gas: U256, target_exists: bool },
	StaticCall { gas: U256, target_exists: bool },
	Suicide { value: U256, target_exists: bool, already_removed: bool },
	SStore { original: H256, current: H256, new: H256 },
	Sha3 { len: U256 },
	Log { n: u8, len: U256 },
	ExtCodeCopy { len: U256 },
	VeryLowCopy { len: U256 },
	Exp { power: U256 },
	Create,
	Create2 { len: U256 },
	JumpDest,
	SLoad,
}

pub struct MemoryCost {
	pub offset: U256,
	pub len: U256,
}

impl MemoryCost {
	pub fn join(self, other: MemoryCost) -> MemoryCost {
		let self_end = self.offset.saturating_add(self.len);
		let other_end = other.offset.saturating_add(other.len);

		if self_end >= other_end {
			self
		} else {
			other
		}
	}
}
