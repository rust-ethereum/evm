mod consts;
mod costs;
mod utils;

use alloc::vec::Vec;
use core::cmp::{max, min};

use evm_interpreter::{
	error::{ExitError, ExitException},
	etable::Control,
	machine::{Machine, Stack},
	opcode::Opcode,
	runtime::{RuntimeBackend, RuntimeState},
};
use primitive_types::{H160, H256, U256};

use crate::{standard::Config, MergeStrategy};

pub struct GasometerState<'config> {
	gas_limit: u64,
	memory_gas: u64,
	used_gas: u64,
	refunded_gas: u64,
	pub is_static: bool,
	pub config: &'config Config,
}

impl<'config> GasometerState<'config> {
	/// Perform any operation on the gasometer. Set the gasometer to `OutOfGas`
	/// if the operation fails.
	#[inline]
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

	/// Set the current gasometer to `OutOfGas`.
	pub fn oog(&mut self) {
		self.memory_gas = 0;
		self.refunded_gas = 0;
		self.used_gas = self.gas_limit;
	}

	/// Total used gas. Simply used gas plus memory cost.
	pub fn total_used_gas(&self) -> u64 {
		self.used_gas + self.memory_gas
	}

	/// Left gas that is supposed to be available to the current interpreter.
	pub fn gas64(&self) -> u64 {
		self.gas_limit - self.memory_gas - self.used_gas
	}

	pub fn gas(&self) -> U256 {
		self.gas64().into()
	}

	/// Record an explicit cost.
	pub fn record_gas64(&mut self, cost: u64) -> Result<(), ExitError> {
		let all_gas_cost = self.total_used_gas().checked_add(cost);
		if let Some(all_gas_cost) = all_gas_cost {
			if self.gas_limit < all_gas_cost {
				Err(ExitException::OutOfGas.into())
			} else {
				self.used_gas += cost;
				Ok(())
			}
		} else {
			Err(ExitException::OutOfGas.into())
		}
	}

	pub fn record_gas(&mut self, cost: U256) -> Result<(), ExitError> {
		if cost > U256::from(u64::MAX) {
			return Err(ExitException::OutOfGas.into());
		}

		self.record_gas64(cost.as_u64())
	}

	pub fn record_codedeposit(&mut self, len: usize) -> Result<(), ExitError> {
		self.perform(|gasometer| {
			let cost = len as u64 * consts::G_CODEDEPOSIT;
			gasometer.record_gas64(cost)?;
			Ok(())
		})
	}

	/// Set memory gas usage.
	pub fn set_memory_gas(&mut self, memory_cost: u64) -> Result<(), ExitError> {
		let all_gas_cost = self.used_gas.checked_add(memory_cost);
		if let Some(all_gas_cost) = all_gas_cost {
			if self.gas_limit < all_gas_cost {
				Err(ExitException::OutOfGas.into())
			} else {
				self.memory_gas = memory_cost;
				Ok(())
			}
		} else {
			Err(ExitException::OutOfGas.into())
		}
	}

	/// Create a new gasometer with the given gas limit and chain config.
	pub fn new(gas_limit: u64, is_static: bool, config: &'config Config) -> Self {
		Self {
			gas_limit,
			memory_gas: 0,
			used_gas: 0,
			refunded_gas: 0,
			is_static,
			config,
		}
	}

	pub fn new_transact_call(
		gas_limit: U256,
		data: &[u8],
		access_list: &[(H160, Vec<H256>)],
		config: &'config Config,
	) -> Result<Self, ExitError> {
		let gas_limit = if gas_limit > U256::from(u64::MAX) {
			return Err(ExitException::OutOfGas.into());
		} else {
			gas_limit.as_u64()
		};

		let mut s = Self::new(gas_limit, false, config);
		let transaction_cost = TransactionCost::call(data, access_list).cost(config);

		s.record_gas64(transaction_cost)?;
		Ok(s)
	}

	pub fn new_transact_create(
		gas_limit: U256,
		code: &[u8],
		access_list: &[(H160, Vec<H256>)],
		config: &'config Config,
	) -> Result<Self, ExitError> {
		let gas_limit = if gas_limit > U256::from(u64::MAX) {
			return Err(ExitException::OutOfGas.into());
		} else {
			gas_limit.as_u64()
		};

		let mut s = Self::new(gas_limit, false, config);
		let transaction_cost = TransactionCost::create(code, access_list).cost(config);

		s.record_gas64(transaction_cost)?;
		Ok(s)
	}

	pub fn effective_gas(&self) -> U256 {
		U256::from(
			self.gas_limit
				- (self.total_used_gas()
					- min(
						self.total_used_gas() / self.config.max_refund_quotient,
						self.refunded_gas,
					)),
		)
	}

	pub fn submeter(
		&mut self,
		gas_limit: U256,
		is_static: bool,
		call_has_value: bool,
	) -> Result<Self, ExitError> {
		let mut gas_limit = if gas_limit > U256::from(u64::MAX) {
			return Err(ExitException::OutOfGas.into());
		} else {
			gas_limit.as_u64()
		};

		self.record_gas64(gas_limit)?;

		if call_has_value {
			gas_limit = gas_limit.saturating_add(self.config.call_stipend);
		}

		Ok(Self::new(gas_limit, is_static, self.config))
	}

	pub fn merge(&mut self, other: Self, strategy: MergeStrategy) {
		match strategy {
			MergeStrategy::Commit => {
				self.used_gas -= other.gas64();
				self.refunded_gas += other.refunded_gas;
			}
			MergeStrategy::Revert => {
				self.used_gas -= other.gas64();
			}
			MergeStrategy::Discard => {}
		}
	}
}

pub fn eval<'config, S, H, Tr>(
	machine: &mut Machine<S>,
	handler: &mut H,
	opcode: Opcode,
	position: usize,
) -> Control<Tr>
where
	S: AsRef<GasometerState<'config>> + AsMut<GasometerState<'config>> + AsRef<RuntimeState>,
	H: RuntimeBackend,
{
	match eval_to_result(machine, handler, opcode, position) {
		Ok(()) => Control::Continue,
		Err(err) => Control::Exit(Err(err)),
	}
}

fn eval_to_result<'config, S, H>(
	machine: &mut Machine<S>,
	handler: &mut H,
	opcode: Opcode,
	_position: usize,
) -> Result<(), ExitError>
where
	S: AsRef<GasometerState<'config>> + AsMut<GasometerState<'config>> + AsRef<RuntimeState>,
	H: RuntimeBackend,
{
	if machine.code().is_empty() {
		return Ok(());
	}

	let address = AsRef::<RuntimeState>::as_ref(&machine.state)
		.context
		.address;

	machine.state.as_mut().perform(|gasometer| {
		if let Some(cost) = consts::STATIC_COST_TABLE[opcode.as_usize()] {
			gasometer.record_gas64(cost)?;
		} else {
			let (gas, memory_gas) = dynamic_opcode_cost(
				address,
				opcode,
				&machine.stack,
				gasometer.is_static,
				gasometer.config,
				handler,
			)?;
			let cost = gas.cost(gasometer.gas64(), gasometer.config)?;
			let refund = gas.refund(gasometer.config);

			gasometer.record_gas64(cost)?;
			if refund >= 0 {
				gasometer.refunded_gas += refund as u64;
			} else {
				gasometer.refunded_gas = gasometer.refunded_gas.saturating_sub(-refund as u64);
			}
			if let Some(memory_gas) = memory_gas {
				let memory_cost = memory_gas.cost()?;
				if let Some(memory_cost) = memory_cost {
					gasometer.set_memory_gas(max(gasometer.memory_gas, memory_cost))?;
				}
			}

			let after_gas = gasometer.gas64();
			gas.extra_check(after_gas, gasometer.config)?;
		}

		Ok(())
	})
}

/// Calculate the opcode cost.
#[allow(clippy::nonminimal_bool)]
fn dynamic_opcode_cost<H: RuntimeBackend>(
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
enum GasCost {
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
struct MemoryCost {
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

/// Transaction cost.
#[derive(Debug, Clone, Copy)]
enum TransactionCost {
	/// Call transaction cost.
	Call {
		/// Length of zeros in transaction data.
		zero_data_len: usize,
		/// Length of non-zeros in transaction data.
		non_zero_data_len: usize,
		/// Number of addresses in transaction access list (see EIP-2930)
		access_list_address_len: usize,
		/// Total number of storage keys in transaction access list (see EIP-2930)
		access_list_storage_len: usize,
	},
	/// Create transaction cost.
	Create {
		/// Length of zeros in transaction data.
		zero_data_len: usize,
		/// Length of non-zeros in transaction data.
		non_zero_data_len: usize,
		/// Number of addresses in transaction access list (see EIP-2930)
		access_list_address_len: usize,
		/// Total number of storage keys in transaction access list (see EIP-2930)
		access_list_storage_len: usize,
		/// Cost of initcode = 2 * ceil(len(initcode) / 32) (see EIP-3860)
		initcode_cost: u64,
	},
}

impl TransactionCost {
	pub fn call(data: &[u8], access_list: &[(H160, Vec<H256>)]) -> TransactionCost {
		let zero_data_len = data.iter().filter(|v| **v == 0).count();
		let non_zero_data_len = data.len() - zero_data_len;
		let (access_list_address_len, access_list_storage_len) = count_access_list(access_list);

		TransactionCost::Call {
			zero_data_len,
			non_zero_data_len,
			access_list_address_len,
			access_list_storage_len,
		}
	}

	pub fn create(data: &[u8], access_list: &[(H160, Vec<H256>)]) -> TransactionCost {
		let zero_data_len = data.iter().filter(|v| **v == 0).count();
		let non_zero_data_len = data.len() - zero_data_len;
		let (access_list_address_len, access_list_storage_len) = count_access_list(access_list);
		let initcode_cost = init_code_cost(data);

		TransactionCost::Create {
			zero_data_len,
			non_zero_data_len,
			access_list_address_len,
			access_list_storage_len,
			initcode_cost,
		}
	}

	pub fn cost(&self, config: &Config) -> u64 {
		match self {
			TransactionCost::Call {
				zero_data_len,
				non_zero_data_len,
				access_list_address_len,
				access_list_storage_len,
			} => {
				#[deny(clippy::let_and_return)]
				let cost = config.gas_transaction_call
					+ *zero_data_len as u64 * config.gas_transaction_zero_data
					+ *non_zero_data_len as u64 * config.gas_transaction_non_zero_data
					+ *access_list_address_len as u64 * config.gas_access_list_address
					+ *access_list_storage_len as u64 * config.gas_access_list_storage_key;

				cost
			}
			TransactionCost::Create {
				zero_data_len,
				non_zero_data_len,
				access_list_address_len,
				access_list_storage_len,
				initcode_cost,
			} => {
				let mut cost = config.gas_transaction_create
					+ *zero_data_len as u64 * config.gas_transaction_zero_data
					+ *non_zero_data_len as u64 * config.gas_transaction_non_zero_data
					+ *access_list_address_len as u64 * config.gas_access_list_address
					+ *access_list_storage_len as u64 * config.gas_access_list_storage_key;
				if config.max_initcode_size.is_some() {
					cost += initcode_cost;
				}

				cost
			}
		}
	}
}

/// Counts the number of addresses and storage keys in the access list
fn count_access_list(access_list: &[(H160, Vec<H256>)]) -> (usize, usize) {
	let access_list_address_len = access_list.len();
	let access_list_storage_len = access_list.iter().map(|(_, keys)| keys.len()).sum();

	(access_list_address_len, access_list_storage_len)
}

fn init_code_cost(data: &[u8]) -> u64 {
	// As per EIP-3860:
	// > We define initcode_cost(initcode) to equal INITCODE_WORD_COST * ceil(len(initcode) / 32).
	// where INITCODE_WORD_COST is 2.
	2 * ((data.len() as u64 + 31) / 32)
}
