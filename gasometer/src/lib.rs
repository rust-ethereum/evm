//! EVM gasometer.

#![deny(warnings)]
#![forbid(unsafe_code, unused_variables, unused_imports)]

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
				$inner = Err(e.clone());
				return Err(e)
			},
		}
	)
}

/// EVM gasometer.
#[derive(Clone)]
pub struct Gasometer<'config> {
	gas_limit: u64,
	config: &'config Config,
	inner: Result<Inner<'config>, ExitError>
}

impl<'config> Gasometer<'config> {
	/// Create a new gasometer with given gas limit and config.
	pub fn new(gas_limit: u64, config: &'config Config) -> Self {
		Self {
			gas_limit,
			config,
			inner: Ok(Inner {
				memory_gas: 0,
				used_gas: 0,
				refunded_gas: 0,
				config,
			}),
		}
	}

	fn inner_mut(
		&mut self
	) -> Result<&mut Inner<'config>, ExitError> {
		self.inner.as_mut().map_err(|e| e.clone())
	}

	/// Reference of the config.
	pub fn config(&self) -> &'config Config {
		self.config
	}

	/// Remaining gas.
	pub fn gas(&self) -> u64 {
		match self.inner.as_ref() {
			Ok(inner) => self.gas_limit - inner.used_gas - inner.memory_gas,
			Err(_) => 0,
		}
	}

	/// Total used gas.
	pub fn total_used_gas(&self) -> u64 {
		match self.inner.as_ref() {
			Ok(inner) => inner.used_gas + inner.memory_gas,
			Err(_) => self.gas_limit,
		}
	}

	/// Refunded gas.
	pub fn refunded_gas(&self) -> i64 {
		match self.inner.as_ref() {
			Ok(inner) => inner.refunded_gas,
			Err(_) => 0,
		}
	}

	/// Explictly fail the gasometer with out of gas. Return `OutOfGas` error.
	pub fn fail(&mut self) -> ExitError {
		self.inner = Err(ExitError::OutOfGas);
		ExitError::OutOfGas
	}

	/// Record an explict cost.
	pub fn record_cost(
		&mut self,
		cost: u64,
	) -> Result<(), ExitError> {
		let all_gas_cost = self.total_used_gas() + cost;
		if self.gas_limit < all_gas_cost {
			self.inner = Err(ExitError::OutOfGas);
			return Err(ExitError::OutOfGas)
		}

		self.inner_mut()?.used_gas += cost;
		Ok(())
	}

	/// Record an explict refund.
	pub fn record_refund(
		&mut self,
		refund: i64,
	) -> Result<(), ExitError> {
		self.inner_mut()?.refunded_gas += refund;
		Ok(())
	}

	/// Record `CREATE` code deposit.
	pub fn record_deposit(
		&mut self,
		len: usize,
	) -> Result<(), ExitError> {
		let cost = len as u64 * consts::G_CODEDEPOSIT;
		self.record_cost(cost)
	}

	/// Record opcode gas cost.
	pub fn record_dynamic_cost(
		&mut self,
		cost: GasCost,
		memory: Option<MemoryCost>,
	) -> Result<(), ExitError> {
		let gas = self.gas();

		let memory_gas = match memory {
			Some(memory) => try_or_fail!(self.inner, self.inner_mut()?.memory_gas(memory)),
			None => self.inner_mut()?.memory_gas,
		};
		let gas_cost = try_or_fail!(self.inner, self.inner_mut()?.gas_cost(cost, gas));
		let gas_refund = self.inner_mut()?.gas_refund(cost);
		let used_gas = self.inner_mut()?.used_gas;

		let all_gas_cost = memory_gas + used_gas + gas_cost;
		if self.gas_limit < all_gas_cost {
			self.inner = Err(ExitError::OutOfGas);
			return Err(ExitError::OutOfGas)
		}

		let after_gas = self.gas_limit - all_gas_cost;
		try_or_fail!(self.inner, self.inner_mut()?.extra_check(cost, after_gas));

		self.inner_mut()?.used_gas += gas_cost;
		self.inner_mut()?.memory_gas = memory_gas;
		self.inner_mut()?.refunded_gas += gas_refund;

		Ok(())
	}

	/// Record opcode stipend.
	pub fn record_stipend(
		&mut self,
		stipend: u64,
	) -> Result<(), ExitError> {
		self.inner_mut()?.used_gas -= stipend;
		Ok(())
	}

	/// Record transaction cost.
	pub fn record_transaction(
		&mut self,
		cost: TransactionCost,
	) -> Result<(), ExitError> {
		let gas_cost = match cost {
			TransactionCost::Call { zero_data_len, non_zero_data_len } => {
				self.config.gas_transaction_call +
					zero_data_len as u64 * self.config.gas_transaction_zero_data +
					non_zero_data_len as u64 * self.config.gas_transaction_non_zero_data
			},
			TransactionCost::Create { zero_data_len, non_zero_data_len } => {
				self.config.gas_transaction_create +
					zero_data_len as u64 * self.config.gas_transaction_zero_data +
					non_zero_data_len as u64 * self.config.gas_transaction_non_zero_data
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

/// Calculate the call transaction cost.
pub fn call_transaction_cost(
	data: &[u8]
) -> TransactionCost {
	let zero_data_len = data.iter().filter(|v| **v == 0).count();
	let non_zero_data_len = data.len() - zero_data_len;

	TransactionCost::Call { zero_data_len, non_zero_data_len }
}

/// Calculate the create transaction cost.
pub fn create_transaction_cost(
	data: &[u8]
) -> TransactionCost {
	let zero_data_len = data.iter().filter(|v| **v == 0).count();
	let non_zero_data_len = data.len() - zero_data_len;

	TransactionCost::Create { zero_data_len, non_zero_data_len }
}

static STATIC_OPCODE_COST_TABLE: [Option<u64>; 256] = {
	let mut table = [None; 256];

	table[Opcode::Stop as usize] = Some(consts::G_ZERO);
	table[Opcode::CallDataSize as usize] = Some(consts::G_BASE);
	table[Opcode::CodeSize as usize] = Some(consts::G_BASE);
	table[Opcode::Pop as usize] = Some(consts::G_BASE);
	table[Opcode::PC as usize] = Some(consts::G_BASE);
	table[Opcode::MSize as usize] = Some(consts::G_BASE);

	table[ExternalOpcode::Address as usize] = Some(consts::G_BASE);
	table[ExternalOpcode::Origin as usize] = Some(consts::G_BASE);
	table[ExternalOpcode::Caller as usize] = Some(consts::G_BASE);
	table[ExternalOpcode::CallValue as usize] = Some(consts::G_BASE);
	table[ExternalOpcode::Coinbase as usize] = Some(consts::G_BASE);
	table[ExternalOpcode::Timestamp as usize] = Some(consts::G_BASE);
	table[ExternalOpcode::Number as usize] = Some(consts::G_BASE);
	table[ExternalOpcode::Difficulty as usize] = Some(consts::G_BASE);
	table[ExternalOpcode::GasLimit as usize] = Some(consts::G_BASE);
	table[ExternalOpcode::GasPrice as usize] = Some(consts::G_BASE);
	table[ExternalOpcode::Gas as usize] = Some(consts::G_BASE);

	table[Opcode::Add as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Sub as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Not as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Lt as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Gt as usize] = Some(consts::G_VERYLOW);
	table[Opcode::SLt as usize] = Some(consts::G_VERYLOW);
	table[Opcode::SGt as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Eq as usize] = Some(consts::G_VERYLOW);
	table[Opcode::IsZero as usize] = Some(consts::G_VERYLOW);
	table[Opcode::And as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Or as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Xor as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Byte as usize] = Some(consts::G_VERYLOW);
	table[Opcode::CallDataLoad as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push1 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push2 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push3 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push4 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push5 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push6 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push7 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push8 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push9 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push10 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push11 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push12 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push13 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push14 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push15 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push16 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push17 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push18 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push19 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push20 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push21 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push22 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push23 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push24 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push25 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push26 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push27 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push28 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push29 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push30 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push31 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Push32 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup1 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup2 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup3 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup4 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup5 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup6 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup7 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup8 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup9 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup10 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup11 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup12 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup13 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup14 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup15 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Dup16 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap1 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap2 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap3 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap4 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap5 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap6 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap7 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap8 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap9 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap10 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap11 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap12 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap13 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap14 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap15 as usize] = Some(consts::G_VERYLOW);
	table[Opcode::Swap16 as usize] = Some(consts::G_VERYLOW);

	table[Opcode::Mul as usize] = Some(consts::G_LOW);
	table[Opcode::Div as usize] = Some(consts::G_LOW);
	table[Opcode::SDiv as usize] = Some(consts::G_LOW);
	table[Opcode::Mod as usize] = Some(consts::G_LOW);
	table[Opcode::SMod as usize] = Some(consts::G_LOW);
	table[Opcode::SignExtend as usize] = Some(consts::G_LOW);

	table[Opcode::AddMod as usize] = Some(consts::G_MID);
	table[Opcode::MulMod as usize] = Some(consts::G_MID);
	table[Opcode::Jump as usize] = Some(consts::G_MID);

	table[Opcode::JumpI as usize] = Some(consts::G_HIGH);

	table
};

pub fn static_opcode_cost(
	opcode: Result<Opcode, ExternalOpcode>,
) -> Option<u64> {
	let index = match opcode {
		Ok(opcode) => opcode as usize,
		Err(opcode) => opcode as usize,
	};

	STATIC_OPCODE_COST_TABLE[index]
}

/// Calculate the opcode cost.
pub fn dynamic_opcode_cost<H: Handler>(
	address: H160,
	opcode: Result<Opcode, ExternalOpcode>,
	stack: &Stack,
	is_static: bool,
	config: &Config,
	handler: &H
) -> Result<(GasCost, Option<MemoryCost>), ExitError> {
	let gas_cost = match opcode {
		Ok(Opcode::Return) => GasCost::Zero,

		Ok(Opcode::MLoad) | Ok(Opcode::MStore) | Ok(Opcode::MStore8) => GasCost::VeryLow,

		Ok(Opcode::Revert) if config.has_revert => GasCost::Zero,
		Ok(Opcode::Revert) => GasCost::Invalid,

		Err(ExternalOpcode::ChainId) if config.has_chain_id => GasCost::Base,
		Err(ExternalOpcode::ChainId) => GasCost::Invalid,

		Ok(Opcode::Shl) | Ok(Opcode::Shr) | Ok(Opcode::Sar) if config.has_bitwise_shifting =>
			GasCost::VeryLow,
		Ok(Opcode::Shl) | Ok(Opcode::Shr) | Ok(Opcode::Sar) => GasCost::Invalid,

		Err(ExternalOpcode::SelfBalance) if config.has_self_balance => GasCost::Low,
		Err(ExternalOpcode::SelfBalance) => GasCost::Invalid,

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
		Err(ExternalOpcode::Log0) if !is_static => GasCost::Log {
			n: 0,
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Err(ExternalOpcode::Log1) if !is_static => GasCost::Log {
			n: 1,
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Err(ExternalOpcode::Log2) if !is_static => GasCost::Log {
			n: 2,
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Err(ExternalOpcode::Log3) if !is_static => GasCost::Log {
			n: 3,
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Err(ExternalOpcode::Log4) if !is_static => GasCost::Log {
			n: 4,
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
		Err(ExternalOpcode::SStore) | Err(ExternalOpcode::Log0) |
		Err(ExternalOpcode::Log1) | Err(ExternalOpcode::Log2) |
		Err(ExternalOpcode::Log3) | Err(ExternalOpcode::Log4) |
		Err(ExternalOpcode::Suicide) | Err(ExternalOpcode::Call) |

		_ => GasCost::Invalid,
	};

	let memory_cost = match opcode {
		Err(ExternalOpcode::Sha3) | Ok(Opcode::Return) | Ok(Opcode::Revert) |
		Err(ExternalOpcode::Log0) | Err(ExternalOpcode::Log1) | Err(ExternalOpcode::Log2) |
		Err(ExternalOpcode::Log3) | Err(ExternalOpcode::Log4) => Some(MemoryCost {
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
	memory_gas: u64,
	used_gas: u64,
	refunded_gas: i64,
	config: &'config Config,
}

impl<'config> Inner<'config> {
	fn memory_gas(
		&self,
		memory: MemoryCost,
	) -> Result<u64, ExitError> {
		let from = memory.offset;
		let len = memory.len;

		if len == U256::zero() {
			return Ok(self.memory_gas)
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

		Ok(max(self.memory_gas, memory::memory_gas(new)?))
	}

	fn extra_check(
		&self,
		cost: GasCost,
		after_gas: u64,
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
		gas: u64,
	) -> Result<u64, ExitError> {
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
			GasCost::SStore { .. } if self.config.estimate => self.config.gas_sstore_set,
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
	) -> i64 {
		match cost {
			_ if self.config.estimate => 0,

			GasCost::SStore { original, current, new } =>
				costs::sstore_refund(original, current, new, self.config),
			GasCost::Suicide { already_removed, .. } =>
				costs::suicide_refund(already_removed),
			_ => 0,
		}
	}
}

/// Gas cost.
#[derive(Debug, Clone, Copy)]
pub enum GasCost {
	/// Zero gas cost.
	Zero,
	/// Base gas cost.
	Base,
	/// Very low gas cost.
	VeryLow,
	/// Low gas cost.
	Low,
	/// Mid gas cost.
	Mid,
	/// High gas cost.
	High,
	/// Fail the gasometer.
	Invalid,

	/// Gas cost for `EXTCODESIZE`.
	ExtCodeSize,
	/// Gas cost for `BALANCE`.
	Balance,
	/// Gas cost for `BLOCKHASH`.
	BlockHash,
	/// Gas cost for `EXTBLOCKHASH`.
	ExtCodeHash,

	/// Gas cost for `CALL`.
	Call {
		/// Call value.
		value: U256,
		/// Call gas.
		gas: U256,
		/// Whether the target exists.
		target_exists: bool
	},
	/// Gas cost for `CALLCODE.
	CallCode {
		/// Call value.
		value: U256,
		/// Call gas.
		gas: U256,
		/// Whether the target exists.
		target_exists: bool
	},
	/// Gas cost for `DELEGATECALL`.
	DelegateCall {
		/// Call gas.
		gas: U256,
		/// Whether the target exists.
		target_exists: bool
	},
	/// Gas cost for `STATICCALL`.
	StaticCall {
		/// Call gas.
		gas: U256,
		/// Whether the target exists.
		target_exists: bool
	},
	/// Gas cost for `SUICIDE`.
	Suicide {
		/// Value.
		value: U256,
		/// Whether the target exists.
		target_exists: bool,
		/// Whether the target has already been removed.
		already_removed: bool
	},
	/// Gas cost for `SSTORE`.
	SStore {
		/// Original value.
		original: H256,
		/// Current value.
		current: H256,
		/// New value.
		new: H256
	},
	/// Gas cost for `SHA3`.
	Sha3 {
		/// Length of the data.
		len: U256
	},
	/// Gas cost for `LOG`.
	Log {
		/// Topic length.
		n: u8,
		/// Data length.
		len: U256
	},
	/// Gas cost for `EXTCODECOPY`.
	ExtCodeCopy {
		/// Length.
		len: U256
	},
	/// Gas cost for some copy opcodes that is documented as `VERYLOW`.
	VeryLowCopy {
		/// Length.
		len: U256
	},
	/// Gas cost for `EXP`.
	Exp {
		/// Power of `EXP`.
		power: U256
	},
	/// Gas cost for `CREATE`.
	Create,
	/// Gas cost for `CREATE2`.
	Create2 {
		/// Length.
		len: U256
	},
	/// Gas cost for `JUMPDEST`.
	JumpDest,
	/// Gas cost for `SLOAD`.
	SLoad,
}

/// Memory cost.
#[derive(Debug, Clone, Copy)]
pub struct MemoryCost {
	/// Affected memory offset.
	pub offset: U256,
	/// Affected length.
	pub len: U256,
}

/// Transaction cost.
#[derive(Debug, Clone, Copy)]
pub enum TransactionCost {
	/// Call transaction cost.
	Call {
		/// Length of zeros in transaction data.
		zero_data_len: usize,
		/// Length of non-zeros in transaction data.
		non_zero_data_len: usize
	},
	/// Create transaction cost.
	Create {
		/// Length of zeros in transaction data.
		zero_data_len: usize,
		/// Length of non-zeros in transaction data.
		non_zero_data_len: usize
	},
}

impl MemoryCost {
	/// Join two memory cost together.
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
