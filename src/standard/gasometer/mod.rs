mod consts;
mod costs;
mod utils;

use crate::standard::Config;
use crate::{Gasometer, GasometerMergeStrategy};
use core::cmp::max;
use evm_interpreter::{ExitError, ExitException, Handler, Machine, Opcode, RuntimeState, Stack};
use primitive_types::{H160, H256, U256};

pub struct StandardGasometer<'config> {
	gas_limit: u64,
	memory_gas: u64,
	used_gas: u64,
	refunded_gas: u64,
	config: &'config Config,
}

impl<'config> StandardGasometer<'config> {
	pub fn perform<R, F: FnOnce(&mut Self) -> Result<R, ExitError>>(
		&mut self,
		f: F,
	) -> Result<R, ExitError> {
		match f(self) {
			Ok(r) => Ok(r),
			Err(e) => {
				self.oog();
				Err(e)
			}
		}
	}

	pub fn oog(&mut self) {
		self.memory_gas = 0;
		self.refunded_gas = 0;
		self.used_gas = self.gas_limit;
	}

	/// Total used gas. Simply used gas plus memory cost.
	pub fn total_used_gas(&self) -> u64 {
		self.used_gas + self.memory_gas
	}

	/// Record an explicit cost.
	fn record_cost_nocleanup(&mut self, cost: u64) -> Result<(), ExitError> {
		let all_gas_cost = self.total_used_gas() + cost;
		if self.gas_limit < all_gas_cost {
			Err(ExitException::OutOfGas.into())
		} else {
			self.used_gas += cost;
			Ok(())
		}
	}
}

impl<'config, H: Handler> Gasometer<RuntimeState, H> for StandardGasometer<'config> {
	type Gas = u64;
	type Config = &'config Config;

	fn new(gas_limit: u64, _machine: &Machine<RuntimeState>, config: &'config Config) -> Self {
		Self {
			gas_limit,
			memory_gas: 0,
			used_gas: 0,
			refunded_gas: 0,
			config,
		}
	}

	fn record_stepn(
		&mut self,
		machine: &Machine<RuntimeState>,
		handler: &H,
		is_static: bool,
	) -> Result<usize, ExitError> {
		self.perform(|gasometer| {
			let opcode = machine.peek_opcode().ok_or(ExitException::OutOfGas)?;

			if let Some(cost) = consts::STATIC_COST_TABLE[opcode.as_usize()] {
				gasometer.record_cost_nocleanup(cost)?;
			} else {
				let address = machine.state.context.address;
				let (gas, memory_gas) = dynamic_opcode_cost(
					address,
					opcode,
					&machine.stack,
					is_static,
					gasometer.config,
					handler,
				)?;
				let cost = gas.cost(
					Gasometer::<RuntimeState, H>::gas(gasometer),
					gasometer.config,
				)?;
				let refund = gas.refund(gasometer.config);

				gasometer.record_cost_nocleanup(cost)?;
				if refund >= 0 {
					gasometer.refunded_gas += refund as u64;
				} else {
					gasometer.refunded_gas = gasometer.refunded_gas.saturating_sub(-refund as u64);
				}
				if let Some(memory_gas) = memory_gas {
					let memory_cost = memory_gas.cost()?;
					if let Some(memory_cost) = memory_cost {
						gasometer.memory_gas = max(gasometer.memory_gas, memory_cost);
					}
				}

				let after_gas = Gasometer::<RuntimeState, H>::gas(gasometer);
				gas.extra_check(after_gas, gasometer.config)?;
			}

			Ok(1)
		})
	}

	fn record_codedeposit(&mut self, len: usize) -> Result<(), ExitError> {
		self.perform(|gasometer| {
			let cost = len as u64 * consts::G_CODEDEPOSIT;
			gasometer.record_cost_nocleanup(cost)?;
			Ok(())
		})
	}

	fn gas(&self) -> u64 {
		self.gas_limit - self.memory_gas - self.used_gas
	}

	fn merge(&mut self, other: Self, strategy: GasometerMergeStrategy) {
		match strategy {
			GasometerMergeStrategy::Commit => {
				self.used_gas -= Gasometer::<RuntimeState, H>::gas(&other);
				self.refunded_gas += other.refunded_gas;
			}
			GasometerMergeStrategy::Revert => {
				self.used_gas -= Gasometer::<RuntimeState, H>::gas(&other);
			}
		}
	}
}

/// Calculate the opcode cost.
#[allow(clippy::nonminimal_bool)]
pub fn dynamic_opcode_cost<H: Handler>(
	address: H160,
	opcode: Opcode,
	stack: &Stack,
	is_static: bool,
	config: &Config,
	handler: &H,
) -> Result<(GasCost, Option<MemoryCost>), ExitError> {
	let gas_cost = match opcode {
		Opcode::RETURN => GasCost::Zero,

		Opcode::MLOAD | Opcode::MSTORE | Opcode::MSTORE8 => GasCost::VeryLow,

		Opcode::REVERT if config.has_revert => GasCost::Zero,
		Opcode::REVERT => GasCost::Invalid(opcode),

		Opcode::CHAINID if config.has_chain_id => GasCost::Base,
		Opcode::CHAINID => GasCost::Invalid(opcode),

		Opcode::SHL | Opcode::SHR | Opcode::SAR if config.has_bitwise_shifting => GasCost::VeryLow,
		Opcode::SHL | Opcode::SHR | Opcode::SAR => GasCost::Invalid(opcode),

		Opcode::SELFBALANCE if config.has_self_balance => GasCost::Low,
		Opcode::SELFBALANCE => GasCost::Invalid(opcode),

		Opcode::BASEFEE if config.has_base_fee => GasCost::Base,
		Opcode::BASEFEE => GasCost::Invalid(opcode),

		Opcode::EXTCODESIZE => {
			let target = stack.peek(0)?.into();
			GasCost::ExtCodeSize {
				target_is_cold: handler.is_cold(target, None),
			}
		}
		Opcode::BALANCE => {
			let target = stack.peek(0)?.into();
			GasCost::Balance {
				target_is_cold: handler.is_cold(target, None),
			}
		}
		Opcode::BLOCKHASH => GasCost::BlockHash,

		Opcode::EXTCODEHASH if config.has_ext_code_hash => {
			let target = stack.peek(0)?.into();
			GasCost::ExtCodeHash {
				target_is_cold: handler.is_cold(target, None),
			}
		}
		Opcode::EXTCODEHASH => GasCost::Invalid(opcode),

		Opcode::CALLCODE => {
			let target = stack.peek(1)?.into();
			GasCost::CallCode {
				value: U256::from_big_endian(&stack.peek(2)?[..]),
				gas: U256::from_big_endian(&stack.peek(0)?[..]),
				target_is_cold: handler.is_cold(target, None),
				target_exists: { handler.exists(target) },
			}
		}
		Opcode::STATICCALL => {
			let target = stack.peek(1)?.into();
			GasCost::StaticCall {
				gas: U256::from_big_endian(&stack.peek(0)?[..]),
				target_is_cold: handler.is_cold(target, None),
				target_exists: { handler.exists(target) },
			}
		}
		Opcode::SHA3 => GasCost::Sha3 {
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Opcode::EXTCODECOPY => {
			let target = stack.peek(0)?.into();
			GasCost::ExtCodeCopy {
				target_is_cold: handler.is_cold(target, None),
				len: U256::from_big_endian(&stack.peek(3)?[..]),
			}
		}
		Opcode::CALLDATACOPY | Opcode::CODECOPY => GasCost::VeryLowCopy {
			len: U256::from_big_endian(&stack.peek(2)?[..]),
		},
		Opcode::EXP => GasCost::Exp {
			power: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Opcode::SLOAD => {
			let index = stack.peek(0)?;
			GasCost::SLoad {
				target_is_cold: handler.is_cold(address, Some(index)),
			}
		}

		Opcode::DELEGATECALL if config.has_delegate_call => {
			let target = stack.peek(1)?.into();
			GasCost::DelegateCall {
				gas: U256::from_big_endian(&stack.peek(0)?[..]),
				target_is_cold: handler.is_cold(target, None),
				target_exists: { handler.exists(target) },
			}
		}
		Opcode::DELEGATECALL => GasCost::Invalid(opcode),

		Opcode::RETURNDATASIZE if config.has_return_data => GasCost::Base,
		Opcode::RETURNDATACOPY if config.has_return_data => GasCost::VeryLowCopy {
			len: U256::from_big_endian(&stack.peek(2)?[..]),
		},
		Opcode::RETURNDATASIZE | Opcode::RETURNDATACOPY => GasCost::Invalid(opcode),

		Opcode::SSTORE if !is_static => {
			let index = stack.peek(0)?;
			let value = stack.peek(1)?;

			GasCost::SStore {
				original: handler.original_storage(address, index),
				current: handler.storage(address, index),
				new: value,
				target_is_cold: handler.is_cold(address, Some(index)),
			}
		}
		Opcode::LOG0 if !is_static => GasCost::Log {
			n: 0,
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Opcode::LOG1 if !is_static => GasCost::Log {
			n: 1,
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Opcode::LOG2 if !is_static => GasCost::Log {
			n: 2,
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Opcode::LOG3 if !is_static => GasCost::Log {
			n: 3,
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Opcode::LOG4 if !is_static => GasCost::Log {
			n: 4,
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		},
		Opcode::CREATE if !is_static => GasCost::Create,
		Opcode::CREATE2 if !is_static && config.has_create2 => GasCost::Create2 {
			len: U256::from_big_endian(&stack.peek(2)?[..]),
		},
		Opcode::SUICIDE if !is_static => {
			let target = stack.peek(0)?.into();
			GasCost::Suicide {
				value: handler.balance(address),
				target_is_cold: handler.is_cold(target, None),
				target_exists: { handler.exists(target) },
				already_removed: handler.deleted(address),
			}
		}
		Opcode::CALL
			if !is_static
				|| (is_static && U256::from_big_endian(&stack.peek(2)?[..]) == U256::zero()) =>
		{
			let target = stack.peek(1)?.into();
			GasCost::Call {
				value: U256::from_big_endian(&stack.peek(2)?[..]),
				gas: U256::from_big_endian(&stack.peek(0)?[..]),
				target_is_cold: handler.is_cold(target, None),
				target_exists: { handler.exists(target) },
			}
		}

		Opcode::PUSH0 if config.has_push0 => GasCost::Base,

		_ => GasCost::Invalid(opcode),
	};

	let memory_cost = match opcode {
		Opcode::SHA3
		| Opcode::RETURN
		| Opcode::REVERT
		| Opcode::LOG0
		| Opcode::LOG1
		| Opcode::LOG2
		| Opcode::LOG3
		| Opcode::LOG4 => Some(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(0)?[..]),
			len: U256::from_big_endian(&stack.peek(1)?[..]),
		}),

		Opcode::CODECOPY | Opcode::CALLDATACOPY | Opcode::RETURNDATACOPY => Some(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(0)?[..]),
			len: U256::from_big_endian(&stack.peek(2)?[..]),
		}),

		Opcode::EXTCODECOPY => Some(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(1)?[..]),
			len: U256::from_big_endian(&stack.peek(3)?[..]),
		}),

		Opcode::MLOAD | Opcode::MSTORE => Some(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(0)?[..]),
			len: U256::from(32),
		}),

		Opcode::MSTORE8 => Some(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(0)?[..]),
			len: U256::from(1),
		}),

		Opcode::CREATE | Opcode::CREATE2 => Some(MemoryCost {
			offset: U256::from_big_endian(&stack.peek(1)?[..]),
			len: U256::from_big_endian(&stack.peek(2)?[..]),
		}),

		Opcode::CALL | Opcode::CALLCODE => Some(
			MemoryCost {
				offset: U256::from_big_endian(&stack.peek(3)?[..]),
				len: U256::from_big_endian(&stack.peek(4)?[..]),
			}
			.join(MemoryCost {
				offset: U256::from_big_endian(&stack.peek(5)?[..]),
				len: U256::from_big_endian(&stack.peek(6)?[..]),
			}),
		),

		Opcode::DELEGATECALL | Opcode::STATICCALL => Some(
			MemoryCost {
				offset: U256::from_big_endian(&stack.peek(2)?[..]),
				len: U256::from_big_endian(&stack.peek(3)?[..]),
			}
			.join(MemoryCost {
				offset: U256::from_big_endian(&stack.peek(4)?[..]),
				len: U256::from_big_endian(&stack.peek(5)?[..]),
			}),
		),

		_ => None,
	};

	Ok((gas_cost, memory_cost))
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
	/// Fail the gasometer.
	Invalid(Opcode),

	/// Gas cost for `EXTCODESIZE`.
	ExtCodeSize {
		/// True if address has not been previously accessed in this transaction
		target_is_cold: bool,
	},
	/// Gas cost for `BALANCE`.
	Balance {
		/// True if address has not been previously accessed in this transaction
		target_is_cold: bool,
	},
	/// Gas cost for `BLOCKHASH`.
	BlockHash,
	/// Gas cost for `EXTBLOCKHASH`.
	ExtCodeHash {
		/// True if address has not been previously accessed in this transaction
		target_is_cold: bool,
	},

	/// Gas cost for `CALL`.
	Call {
		/// Call value.
		value: U256,
		/// Call gas.
		gas: U256,
		/// True if target has not been previously accessed in this transaction
		target_is_cold: bool,
		/// Whether the target exists.
		target_exists: bool,
	},
	/// Gas cost for `CALLCODE.
	CallCode {
		/// Call value.
		value: U256,
		/// Call gas.
		gas: U256,
		/// True if target has not been previously accessed in this transaction
		target_is_cold: bool,
		/// Whether the target exists.
		target_exists: bool,
	},
	/// Gas cost for `DELEGATECALL`.
	DelegateCall {
		/// Call gas.
		gas: U256,
		/// True if target has not been previously accessed in this transaction
		target_is_cold: bool,
		/// Whether the target exists.
		target_exists: bool,
	},
	/// Gas cost for `STATICCALL`.
	StaticCall {
		/// Call gas.
		gas: U256,
		/// True if target has not been previously accessed in this transaction
		target_is_cold: bool,
		/// Whether the target exists.
		target_exists: bool,
	},
	/// Gas cost for `SUICIDE`.
	Suicide {
		/// Value.
		value: U256,
		/// True if target has not been previously accessed in this transaction
		target_is_cold: bool,
		/// Whether the target exists.
		target_exists: bool,
		/// Whether the target has already been removed.
		already_removed: bool,
	},
	/// Gas cost for `SSTORE`.
	SStore {
		/// Original value.
		original: H256,
		/// Current value.
		current: H256,
		/// New value.
		new: H256,
		/// True if target has not been previously accessed in this transaction
		target_is_cold: bool,
	},
	/// Gas cost for `SHA3`.
	Sha3 {
		/// Length of the data.
		len: U256,
	},
	/// Gas cost for `LOG`.
	Log {
		/// Topic length.
		n: u8,
		/// Data length.
		len: U256,
	},
	/// Gas cost for `EXTCODECOPY`.
	ExtCodeCopy {
		/// True if target has not been previously accessed in this transaction
		target_is_cold: bool,
		/// Length.
		len: U256,
	},
	/// Gas cost for some copy opcodes that is documented as `VERYLOW`.
	VeryLowCopy {
		/// Length.
		len: U256,
	},
	/// Gas cost for `EXP`.
	Exp {
		/// Power of `EXP`.
		power: U256,
	},
	/// Gas cost for `CREATE`.
	Create,
	/// Gas cost for `CREATE2`.
	Create2 {
		/// Length.
		len: U256,
	},
	/// Gas cost for `SLOAD`.
	SLoad {
		/// True if target has not been previously accessed in this transaction
		target_is_cold: bool,
	},
}

impl GasCost {
	/// Returns the gas cost numerical value.
	pub fn cost(&self, gas: u64, config: &Config) -> Result<u64, ExitError> {
		Ok(match *self {
			GasCost::Call {
				value,
				target_is_cold,
				target_exists,
				..
			} => costs::call_cost(value, target_is_cold, true, true, !target_exists, config),
			GasCost::CallCode {
				value,
				target_is_cold,
				target_exists,
				..
			} => costs::call_cost(value, target_is_cold, true, false, !target_exists, config),
			GasCost::DelegateCall {
				target_is_cold,
				target_exists,
				..
			} => costs::call_cost(
				U256::zero(),
				target_is_cold,
				false,
				false,
				!target_exists,
				config,
			),
			GasCost::StaticCall {
				target_is_cold,
				target_exists,
				..
			} => costs::call_cost(
				U256::zero(),
				target_is_cold,
				false,
				true,
				!target_exists,
				config,
			),

			GasCost::Suicide {
				value,
				target_is_cold,
				target_exists,
				..
			} => costs::suicide_cost(value, target_is_cold, target_exists, config),
			GasCost::SStore {
				original,
				current,
				new,
				target_is_cold,
			} => costs::sstore_cost(original, current, new, gas, target_is_cold, config)?,

			GasCost::Sha3 { len } => costs::sha3_cost(len)?,
			GasCost::Log { n, len } => costs::log_cost(n, len)?,
			GasCost::VeryLowCopy { len } => costs::verylowcopy_cost(len)?,
			GasCost::Exp { power } => costs::exp_cost(power, config)?,
			GasCost::Create => consts::G_CREATE,
			GasCost::Create2 { len } => costs::create2_cost(len)?,
			GasCost::SLoad { target_is_cold } => costs::sload_cost(target_is_cold, config),

			GasCost::Zero => consts::G_ZERO,
			GasCost::Base => consts::G_BASE,
			GasCost::VeryLow => consts::G_VERYLOW,
			GasCost::Low => consts::G_LOW,
			GasCost::Invalid(opcode) => return Err(ExitException::InvalidOpcode(opcode).into()),

			GasCost::ExtCodeSize { target_is_cold } => {
				costs::address_access_cost(target_is_cold, config.gas_ext_code, config)
			}
			GasCost::ExtCodeCopy {
				target_is_cold,
				len,
			} => costs::extcodecopy_cost(len, target_is_cold, config)?,
			GasCost::Balance { target_is_cold } => {
				costs::address_access_cost(target_is_cold, config.gas_balance, config)
			}
			GasCost::BlockHash => consts::G_BLOCKHASH,
			GasCost::ExtCodeHash { target_is_cold } => {
				costs::address_access_cost(target_is_cold, config.gas_ext_code_hash, config)
			}
		})
	}

	/// Numeric value for the refund.
	pub fn refund(&self, config: &Config) -> i64 {
		match *self {
			GasCost::SStore {
				original,
				current,
				new,
				..
			} => costs::sstore_refund(original, current, new, config),
			GasCost::Suicide {
				already_removed, ..
			} if !config.decrease_clears_refund => costs::suicide_refund(already_removed),
			_ => 0,
		}
	}

	/// Extra check of the cost.
	pub fn extra_check(&self, after_gas: u64, config: &Config) -> Result<(), ExitException> {
		match *self {
			GasCost::Call { gas, .. } => costs::call_extra_check(gas, after_gas, config),
			GasCost::CallCode { gas, .. } => costs::call_extra_check(gas, after_gas, config),
			GasCost::DelegateCall { gas, .. } => costs::call_extra_check(gas, after_gas, config),
			GasCost::StaticCall { gas, .. } => costs::call_extra_check(gas, after_gas, config),
			_ => Ok(()),
		}
	}
}

/// Memory cost.
#[derive(Debug, Clone, Copy)]
pub struct MemoryCost {
	/// Affected memory offset.
	pub offset: U256,
	/// Affected length.
	pub len: U256,
}

impl MemoryCost {
	/// Join two memory cost together.
	pub fn join(self, other: MemoryCost) -> MemoryCost {
		if self.len == U256::zero() {
			return other;
		}

		if other.len == U256::zero() {
			return self;
		}

		let self_end = self.offset.saturating_add(self.len);
		let other_end = other.offset.saturating_add(other.len);

		if self_end >= other_end {
			self
		} else {
			other
		}
	}

	/// Numeric value of the cost.
	pub fn cost(&self) -> Result<Option<u64>, ExitError> {
		let from = self.offset;
		let len = self.len;

		if len == U256::zero() {
			return Ok(None);
		}

		let end = from.checked_add(len).ok_or(ExitException::OutOfGas)?;

		if end > U256::from(usize::MAX) {
			return Err(ExitException::OutOfGas.into());
		}
		let end = end.as_usize();

		let rem = end % 32;
		let new = if rem == 0 { end / 32 } else { end / 32 + 1 };

		Ok(Some(costs::memory_gas(new)?))
	}
}