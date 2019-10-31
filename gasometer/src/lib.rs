#![cfg_attr(not(feature = "std"), no_std)]

mod consts;
mod costs;
mod memory;
mod utils;

use core::cmp::max;
use primitive_types::{H160, H256, U256};
use evm_core::{ExternalOpcode, Opcode, ExitError, Stack};
use evm_runtime::{Handler, Config};

macro_rules! try_or_fail {
	( $inner:expr, $e:expr ) => (
		match $e {
			Ok(value) => value,
			Err(e) => {
				$inner = Err(e);
				return Err(e)
			},
		}
	)
}

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

	pub fn fail(&mut self) -> ExitError {
		self.inner = Err(ExitError::OutOfGas);
		ExitError::OutOfGas
	}

	pub fn record_cost(
		&mut self,
		cost: usize
	) -> Result<(), ExitError> {
		let all_gas_cost = self.total_used_gas() + cost;
		if self.gas_limit < all_gas_cost {
			self.inner = Err(ExitError::OutOfGas);
			return Err(ExitError::OutOfGas)
		}

		self.inner_mut()?.used_gas += cost;
		Ok(())
	}

	pub fn record_refund(
		&mut self,
		refund: isize,
	) -> Result<(), ExitError> {
		self.inner_mut()?.refunded_gas += refund;
		Ok(())
	}

	pub fn record_deposit(
		&mut self,
		len: usize
	) -> Result<(), ExitError> {
		let cost = len * consts::G_CODEDEPOSIT;
		self.record_cost(cost)
	}

	pub fn record_opcode(
		&mut self,
		cost: GasCost,
		memory: Option<MemoryCost>,
	) -> Result<(), ExitError> {
		let gas = self.gas();

		let memory_cost = match memory {
			Some(memory) => try_or_fail!(self.inner, self.inner_mut()?.memory_cost(memory)),
			None => self.inner_mut()?.memory_cost,
		};
		let memory_gas = try_or_fail!(self.inner, memory::memory_gas(memory_cost));
		let gas_cost = try_or_fail!(self.inner, self.inner_mut()?.gas_cost(cost.clone(), gas));
		let gas_refund = self.inner_mut()?.gas_refund(cost.clone());
		let used_gas = self.inner_mut()?.used_gas;

		let all_gas_cost = memory_gas + used_gas + gas_cost;
		if self.gas_limit < all_gas_cost {
			self.inner = Err(ExitError::OutOfGas);
			return Err(ExitError::OutOfGas)
		}

		let after_gas = self.gas_limit - all_gas_cost;
		try_or_fail!(self.inner, self.inner_mut()?.extra_check(cost, after_gas));

		self.inner_mut()?.used_gas += gas_cost;
		self.inner_mut()?.memory_cost = memory_cost;
		self.inner_mut()?.refunded_gas += gas_refund;

		Ok(())
	}

	pub fn record_stipend(
		&mut self,
		stipend: usize,
	) -> Result<(), ExitError> {
		self.inner_mut()?.used_gas -= stipend;
		Ok(())
	}

	pub fn record_transaction(
		&mut self,
		cost: TransactionCost,
	) -> Result<(), ExitError> {
		let gas_cost = match cost {
			TransactionCost::Call { zero_data_len, non_zero_data_len } => {
				self.config.gas_transaction_call +
					zero_data_len * self.config.gas_transaction_zero_data +
					non_zero_data_len * self.config.gas_transaction_non_zero_data
			},
			TransactionCost::Create { zero_data_len, non_zero_data_len } => {
				self.config.gas_transaction_create +
					zero_data_len * self.config.gas_transaction_zero_data +
					non_zero_data_len * self.config.gas_transaction_non_zero_data
			},
		};

		if self.gas() < gas_cost {
			self.inner = Err(ExitError::OutOfGas);
			return Err(ExitError::OutOfGas);
		}

		self.inner_mut()?.used_gas += gas_cost;
		Ok(())
	}
}

pub fn call_transaction_cost(
	data: &[u8]
) -> TransactionCost {
	let zero_data_len = data.iter().filter(|v| **v == 0).count();
	let non_zero_data_len = data.len() - zero_data_len;

	TransactionCost::Call { zero_data_len, non_zero_data_len }
}

pub fn create_transaction_cost(
	data: &[u8]
) -> TransactionCost {
	let zero_data_len = data.iter().filter(|v| **v == 0).count();
	let non_zero_data_len = data.len() - zero_data_len;

	TransactionCost::Create { zero_data_len, non_zero_data_len }
}

pub fn opcode_cost<H: Handler>(
	address: H160,
	opcode: Result<Opcode, ExternalOpcode>,
	stack: &Stack,
	is_static: bool,
	config: &Config,
	handler: &H
) -> Result<(GasCost, Option<MemoryCost>), ExitError> {
	let gas_cost = match opcode {
		Ok(Opcode::Stop) | Ok(Opcode::Return) => GasCost::Zero,

		Ok(Opcode::Revert) if config.has_revert => GasCost::Zero,
		Ok(Opcode::Revert) => GasCost::Invalid,

		Err(ExternalOpcode::Address) | Err(ExternalOpcode::Origin) | Err(ExternalOpcode::Caller) |
		Err(ExternalOpcode::CallValue) | Ok(Opcode::CallDataSize) |
		Ok(Opcode::CodeSize) | Err(ExternalOpcode::GasPrice) | Err(ExternalOpcode::Coinbase) |
		Err(ExternalOpcode::Timestamp) | Err(ExternalOpcode::Number) |
		Err(ExternalOpcode::Difficulty) |
		Err(ExternalOpcode::GasLimit) | Ok(Opcode::Pop) | Ok(Opcode::PC) |
		Ok(Opcode::MSize) | Err(ExternalOpcode::Gas) => GasCost::Base,

		Err(ExternalOpcode::ChainId) if config.has_chain_id => GasCost::Base,
		Err(ExternalOpcode::ChainId) => GasCost::Invalid,

		Ok(Opcode::Add) | Ok(Opcode::Sub) | Ok(Opcode::Not) | Ok(Opcode::Lt) |
		Ok(Opcode::Gt) | Ok(Opcode::SLt) | Ok(Opcode::SGt) | Ok(Opcode::Eq) |
		Ok(Opcode::IsZero) | Ok(Opcode::And) | Ok(Opcode::Or) | Ok(Opcode::Xor) |
		Ok(Opcode::Byte) | Ok(Opcode::CallDataLoad) | Ok(Opcode::MLoad) |
		Ok(Opcode::MStore) | Ok(Opcode::MStore8) | Ok(Opcode::Push(_)) |
		Ok(Opcode::Dup(_)) | Ok(Opcode::Swap(_)) => GasCost::VeryLow,

		Ok(Opcode::Shl) | Ok(Opcode::Shr) | Ok(Opcode::Sar) if config.has_bitwise_shifting =>
			GasCost::VeryLow,
		Ok(Opcode::Shl) | Ok(Opcode::Shr) | Ok(Opcode::Sar) => GasCost::Invalid,

		Ok(Opcode::Mul) | Ok(Opcode::Div) | Ok(Opcode::SDiv) | Ok(Opcode::Mod) |
		Ok(Opcode::SMod) | Ok(Opcode::SignExtend) => GasCost::Low,

		Err(ExternalOpcode::SelfBalance) if config.has_self_balance => GasCost::Low,
		Err(ExternalOpcode::SelfBalance) => GasCost::Invalid,

		Ok(Opcode::AddMod) | Ok(Opcode::MulMod) | Ok(Opcode::Jump) => GasCost::Mid,

		Ok(Opcode::JumpI) => GasCost::High,

		Err(ExternalOpcode::ExtCodeSize) => GasCost::ExtCodeSize,
		Err(ExternalOpcode::Balance) => GasCost::Balance,
		Err(ExternalOpcode::BlockHash) => GasCost::BlockHash,

		Err(ExternalOpcode::ExtCodeHash) if config.has_ext_code_hash => GasCost::ExtCodeHash,
		Err(ExternalOpcode::ExtCodeHash) => GasCost::Invalid,

		Err(ExternalOpcode::CallCode) => GasCost::CallCode {
			value: U256::from_big_endian(&stack.peek(2)?[..]),
			gas: U256::from_big_endian(&stack.peek(0)?[..]),
			target_exists: handler.exists(stack.peek(1)?.into()),
		},
		Err(ExternalOpcode::StaticCall) => GasCost::StaticCall {
			gas: U256::from_big_endian(&stack.peek(0)?[..]),
			target_exists: handler.exists(stack.peek(1)?.into()),
		},
		Err(ExternalOpcode::Sha3) => GasCost::Sha3 {
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Err(ExternalOpcode::ExtCodeCopy) => GasCost::ExtCodeCopy {
			len: U256::from_big_endian(&stack.peek(3)?[..]),
		},
		Ok(Opcode::CallDataCopy) | Ok(Opcode::CodeCopy) => GasCost::VeryLowCopy {
			len: U256::from_big_endian(&stack.peek(2)?[..]),
		},
		Ok(Opcode::Exp) => GasCost::Exp {
			power: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Ok(Opcode::JumpDest) => GasCost::JumpDest,
		Err(ExternalOpcode::SLoad) => GasCost::SLoad,

		Err(ExternalOpcode::DelegateCall) if config.has_delegate_call => GasCost::DelegateCall {
			gas: U256::from_big_endian(&stack.peek(0)?[..]),
			target_exists: handler.exists(stack.peek(1)?.into()),
		},
		Err(ExternalOpcode::DelegateCall) => GasCost::Invalid,

		Err(ExternalOpcode::ReturnDataSize) if config.has_return_data => GasCost::Base,
		Err(ExternalOpcode::ReturnDataCopy) if config.has_return_data => GasCost::VeryLowCopy {
			len: U256::from_big_endian(&stack.peek(2)?[..]),
		},
		Err(ExternalOpcode::ReturnDataSize) | Err(ExternalOpcode::ReturnDataCopy) => GasCost::Invalid,

		Err(ExternalOpcode::SStore) if !is_static => {
			let index = stack.peek(0)?;
			let value = stack.peek(1)?;

			GasCost::SStore {
				original: handler.original_storage(address, index),
				current: handler.storage(address, index),
				new: value,
			}
		},
		Err(ExternalOpcode::Log(n)) if !is_static => GasCost::Log {
			n,
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Err(ExternalOpcode::Create) if !is_static => GasCost::Create,
		Err(ExternalOpcode::Create2) if !is_static && config.has_create2 => GasCost::Create2 {
			len: U256::from_big_endian(&stack.peek(2)?[..]),
		},
		Err(ExternalOpcode::Suicide) if !is_static => GasCost::Suicide {
			value: handler.balance(address),
			target_exists: handler.exists(stack.peek(0)?.into()),
			already_removed: handler.deleted(address),
		},
		Err(ExternalOpcode::Call)
			if !is_static ||
			(is_static && U256::from_big_endian(&stack.peek(2)?[..]) == U256::zero()) =>
			GasCost::Call {
				value: U256::from_big_endian(&stack.peek(2)?[..]),
				gas: U256::from_big_endian(&stack.peek(0)?[..]),
				target_exists: handler.exists(stack.peek(1)?.into()),
			},

		Ok(Opcode::Invalid) => GasCost::Invalid,

		Err(ExternalOpcode::Create) | Err(ExternalOpcode::Create2) |
		Err(ExternalOpcode::SStore) | Err(ExternalOpcode::Log(_)) |
		Err(ExternalOpcode::Suicide) | Err(ExternalOpcode::Call) |

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
		gas: usize,
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
				costs::sstore_cost(original, current, new, gas, self.config)?,

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

			GasCost::ExtCodeSize => self.config.gas_ext_code,
			GasCost::Balance => self.config.gas_balance,
			GasCost::BlockHash => consts::G_BLOCKHASH,
			GasCost::ExtCodeHash => self.config.gas_ext_code_hash,
		})
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct MemoryCost {
	pub offset: U256,
	pub len: U256,
}

#[derive(Debug, Clone)]
pub enum TransactionCost {
	Call { zero_data_len: usize, non_zero_data_len: usize },
	Create { zero_data_len: usize, non_zero_data_len: usize },
}

impl MemoryCost {
	pub fn join(self, other: MemoryCost) -> MemoryCost {
		if self.len == U256::zero() {
			return other
		}

		if other.len == U256::zero() {
			return self
		}

		let self_end = self.offset.saturating_add(self.len);
		let other_end = other.offset.saturating_add(other.len);

		if self_end >= other_end {
			self
		} else {
			other
		}
	}
}
