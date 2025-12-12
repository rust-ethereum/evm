mod consts;
mod costs;

use alloc::vec::Vec;
use core::cmp::max;

use evm_interpreter::uint::{H160, H256, U256, U256Ext};
use evm_interpreter::{
	Control, ExitError, ExitException, Machine, Opcode, Stack,
	runtime::{RuntimeBackend, RuntimeState, TouchKind},
	utils::u256_to_usize,
};

use crate::{MergeStrategy, standard::Config};

/// Gasometer state.
pub struct GasometerState {
	gas_limit: u64,
	memory_gas: u64,
	used_gas: u64,
	floor_gas: u64,
	refunded_gas: i64,
	/// Whether the gasometer is in static context.
	pub is_static: bool,
}

impl GasometerState {
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

	/// Left gas. Same as [Self::gas64] but in [U256].
	pub fn gas(&self) -> U256 {
		U256::from_u64(self.gas64())
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

	/// Record an arbitrary gas.
	pub fn record_gas(&mut self, cost: U256) -> Result<(), ExitError> {
		if cost > U256::from(u64::MAX) {
			return Err(ExitException::OutOfGas.into());
		}

		self.record_gas64(cost.as_u64())
	}

	/// Record code deposit gas.
	pub fn record_codedeposit(&mut self, len: usize) -> Result<(), ExitError> {
		let cost = len as u64 * consts::G_CODEDEPOSIT;
		self.record_gas64(cost)?;
		Ok(())
	}

	/// Record used and floor costs of a transaction.
	pub fn records_transaction_cost(&mut self, cost: TransactionGas) -> Result<(), ExitError> {
		self.record_gas64(cost.used)?;
		self.floor_gas = cost.floor;
		Ok(())
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
	pub fn new(gas_limit: u64, is_static: bool) -> Self {
		Self {
			gas_limit,
			memory_gas: 0,
			used_gas: 0,
			floor_gas: 0,
			refunded_gas: 0,
			is_static,
		}
	}

	/// Create a new gasometer for a call transaction.
	pub fn new_transact_call(
		gas_limit: U256,
		data: &[u8],
		access_list: &[(H160, Vec<H256>)],
		config: &Config,
	) -> Result<Self, ExitError> {
		let gas_limit = if gas_limit > U256::from(u64::MAX) {
			return Err(ExitException::OutOfGas.into());
		} else {
			gas_limit.as_u64()
		};

		let cost = TransactionCost::call(data, access_list).cost(config);

		// EIP-7623: Check if gas limit meets the floor requirement
		if config.eip7623_calldata_floor && gas_limit < cost.floor {
			return Err(ExitException::OutOfGas.into());
		}

		let mut s = Self::new(gas_limit, false);
		s.records_transaction_cost(cost)?;
		Ok(s)
	}

	/// Create a new gasometer for a create transaction.
	pub fn new_transact_create(
		gas_limit: U256,
		code: &[u8],
		access_list: &[(H160, Vec<H256>)],
		config: &Config,
	) -> Result<Self, ExitError> {
		let gas_limit = if gas_limit > U256::from(u64::MAX) {
			return Err(ExitException::OutOfGas.into());
		} else {
			gas_limit.as_u64()
		};

		let cost = TransactionCost::create(code, access_list).cost(config);

		// EIP-7623: Check if gas limit meets the floor requirement
		if config.eip7623_calldata_floor && gas_limit < cost.floor {
			return Err(ExitException::OutOfGas.into());
		}

		let mut s = Self::new(gas_limit, false);
		s.records_transaction_cost(cost)?;
		Ok(s)
	}

	/// The effective used gas at the end of the transaction.
	///
	/// In case of revert, refunded gas are not taken into account.
	pub fn effective_gas(&self, with_refund: bool, config: &Config) -> U256 {
		let refunded_gas = self.refunded_gas.max(0) as u64;

		let used_gas = if with_refund {
			let max_refund = self.total_used_gas() / config.max_refund_quotient();
			self.total_used_gas() - refunded_gas.min(max_refund)
		} else {
			self.total_used_gas()
		};

		let used_gas = if config.eip7623_calldata_floor {
			used_gas.max(self.floor_gas)
		} else {
			used_gas
		};

		U256::from(self.gas_limit - used_gas)
	}

	/// Create a submeter.
	pub fn submeter(
		&mut self,
		gas_limit: U256,
		is_static: bool,
		call_has_value: bool,
		config: &Config,
	) -> Result<Self, ExitError> {
		let mut gas_limit = if gas_limit > U256::from(u64::MAX) {
			return Err(ExitException::OutOfGas.into());
		} else {
			gas_limit.as_u64()
		};

		self.record_gas64(gas_limit)?;

		if call_has_value {
			gas_limit = gas_limit.saturating_add(config.call_stipend());
		}

		Ok(Self::new(gas_limit, is_static))
	}

	/// Merge another gasometer to this one using the given merge strategy.
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

/// The eval function of the entire gasometer.
///
/// Usually wrapped by [crate::interpreter::etable::Single].
pub fn eval<S, H, Tr>(machine: &mut Machine<S>, handler: &mut H, position: usize) -> Control<Tr>
where
	S: AsRef<GasometerState> + AsMut<GasometerState> + AsRef<RuntimeState> + AsRef<Config>,
	H: RuntimeBackend,
{
	match eval_to_result(machine, handler, position) {
		Ok(()) => Control::NoAction,
		Err(err) => Control::Exit(Err(err)),
	}
}

fn eval_to_result<S, H>(
	machine: &mut Machine<S>,
	handler: &mut H,
	position: usize,
) -> Result<(), ExitError>
where
	S: AsRef<GasometerState> + AsMut<GasometerState> + AsRef<RuntimeState> + AsRef<Config>,
	H: RuntimeBackend,
{
	let opcode = Opcode(machine.code()[position]);

	let address = AsRef::<RuntimeState>::as_ref(&machine.state)
		.context
		.address;

	let mut f = || {
		if let Some(cost) = consts::STATIC_COST_TABLE[opcode.as_usize()] {
			machine.state.as_mut().record_gas64(cost)?;
		} else {
			let (gas, memory_gas) = dynamic_opcode_cost(
				address,
				opcode,
				&machine.stack,
				machine.state.as_mut().is_static,
				machine.state.as_ref(),
				handler,
			)?;
			let cost = gas.cost(machine.state.as_mut().gas64(), machine.state.as_ref())?;
			let refund = gas.refund(machine.state.as_ref());

			machine.state.as_mut().record_gas64(cost)?;
			machine.state.as_mut().refunded_gas += refund;

			if let Some(memory_gas) = memory_gas {
				let memory_cost = memory_gas.cost()?;
				if let Some(memory_cost) = memory_cost {
					let memory_gas = max(machine.state.as_mut().memory_gas, memory_cost);
					machine.state.as_mut().set_memory_gas(memory_gas)?;
				}
			}

			let after_gas = machine.state.as_mut().gas64();
			gas.extra_check(after_gas, machine.state.as_ref())?;
		}

		Ok(())
	};

	match f() {
		Ok(r) => Ok(r),
		Err(e) => {
			machine.state.as_mut().oog();
			Err(e)
		}
	}
}

/// Calculate the opcode cost.
#[allow(clippy::nonminimal_bool)]
fn dynamic_opcode_cost<H: RuntimeBackend>(
	address: H160,
	opcode: Opcode,
	stack: &Stack,
	is_static: bool,
	config: &Config,
	handler: &mut H,
) -> Result<(GasCost, Option<MemoryCost>), ExitError> {
	let gas_cost = match opcode {
		Opcode::RETURN => GasCost::Zero,

		Opcode::MLOAD | Opcode::MSTORE | Opcode::MSTORE8 => GasCost::VeryLow,

		Opcode::REVERT if config.eip140_revert => GasCost::Zero,
		Opcode::REVERT => GasCost::Invalid(opcode),

		Opcode::CHAINID if config.eip1344_chain_id => GasCost::Base,
		Opcode::CHAINID => GasCost::Invalid(opcode),

		Opcode::SHL | Opcode::SHR | Opcode::SAR if config.eip145_bitwise_shifting => {
			GasCost::VeryLow
		}
		Opcode::SHL | Opcode::SHR | Opcode::SAR => GasCost::Invalid(opcode),

		Opcode::SELFBALANCE if config.eip1884_self_balance => GasCost::Low,
		Opcode::SELFBALANCE => GasCost::Invalid(opcode),

		Opcode::BASEFEE if config.eip3198_base_fee => GasCost::Base,
		Opcode::BASEFEE => GasCost::Invalid(opcode),

		Opcode::BLOBHASH if config.eip4844_shard_blob => GasCost::VeryLow,
		Opcode::BLOBHASH => GasCost::Invalid(opcode),

		Opcode::BLOBBASEFEE if config.eip7516_blob_base_fee => GasCost::Base,
		Opcode::BLOBBASEFEE => GasCost::Invalid(opcode),

		Opcode::EXTCODESIZE => {
			let target = stack.peek(0)?.to_h160();

			// https://eips.ethereum.org/EIPS/eip-2929
			let target_is_cold = handler.is_cold(target, None);
			handler.mark_hot(target, TouchKind::Access);

			GasCost::ExtCodeSize { target_is_cold }
		}
		Opcode::BALANCE => {
			let target = stack.peek(0)?.to_h160();

			// https://eips.ethereum.org/EIPS/eip-2929
			let target_is_cold = handler.is_cold(target, None);
			handler.mark_hot(target, TouchKind::Access);

			GasCost::Balance { target_is_cold }
		}
		Opcode::BLOCKHASH => GasCost::BlockHash,

		Opcode::EXTCODEHASH if config.eip1052_ext_code_hash => {
			let target = stack.peek(0)?.to_h160();

			// https://eips.ethereum.org/EIPS/eip-2929
			let target_is_cold = handler.is_cold(target, None);
			handler.mark_hot(target, TouchKind::Access);

			GasCost::ExtCodeHash { target_is_cold }
		}
		Opcode::EXTCODEHASH => GasCost::Invalid(opcode),

		Opcode::CALLCODE => {
			let target = stack.peek(1)?.to_h160();
			let target_exists = handler.exists(target);

			// https://eips.ethereum.org/EIPS/eip-2929
			let target_is_cold = handler.is_cold(target, None);
			handler.mark_hot(target, TouchKind::Access);

			GasCost::CallCode {
				value: stack.peek(2)?,
				gas: stack.peek(0)?,
				target_is_cold,
				target_exists,
			}
		}

		Opcode::STATICCALL if config.eip214_static_call => {
			let target = stack.peek(1)?.to_h160();
			let target_exists = handler.exists(target);

			// https://eips.ethereum.org/EIPS/eip-2929
			let target_is_cold = handler.is_cold(target, None);
			handler.mark_hot(target, TouchKind::Access);

			GasCost::StaticCall {
				gas: stack.peek(0)?,
				target_is_cold,
				target_exists,
			}
		}
		Opcode::STATICCALL => GasCost::Invalid(opcode),

		Opcode::SHA3 => GasCost::Sha3 {
			len: stack.peek(1)?,
		},
		Opcode::EXTCODECOPY => {
			let target = stack.peek(0)?.to_h160();

			// https://eips.ethereum.org/EIPS/eip-2929
			let target_is_cold = handler.is_cold(target, None);
			handler.mark_hot(target, TouchKind::Access);

			GasCost::ExtCodeCopy {
				target_is_cold,
				len: stack.peek(3)?,
			}
		}
		Opcode::MCOPY if config.eip5656_mcopy => GasCost::VeryLowCopy {
			len: stack.peek(2)?,
		},
		Opcode::CALLDATACOPY | Opcode::CODECOPY => GasCost::VeryLowCopy {
			len: stack.peek(2)?,
		},
		Opcode::EXP => GasCost::Exp {
			power: stack.peek(1)?,
		},
		Opcode::SLOAD => {
			let index = stack.peek(0)?.to_h256();

			// https://eips.ethereum.org/EIPS/eip-2929
			let target_is_cold = handler.is_cold(address, Some(index));
			handler.mark_storage_hot(address, index);

			GasCost::SLoad { target_is_cold }
		}
		Opcode::TLOAD if config.eip1153_transient_storage => GasCost::TLoad,

		Opcode::DELEGATECALL if config.eip7_delegate_call => {
			let target = stack.peek(1)?.to_h160();
			let target_exists = handler.exists(target);

			// https://eips.ethereum.org/EIPS/eip-2929
			let target_is_cold = handler.is_cold(target, None);
			handler.mark_hot(target, TouchKind::Access);

			GasCost::DelegateCall {
				gas: stack.peek(0)?,
				target_is_cold,
				target_exists,
			}
		}
		Opcode::DELEGATECALL => GasCost::Invalid(opcode),

		Opcode::RETURNDATASIZE if config.eip211_return_data => GasCost::Base,
		Opcode::RETURNDATACOPY if config.eip211_return_data => GasCost::VeryLowCopy {
			len: stack.peek(2)?,
		},
		Opcode::RETURNDATASIZE | Opcode::RETURNDATACOPY => GasCost::Invalid(opcode),

		Opcode::SSTORE if !is_static => {
			let index = stack.peek(0)?.to_h256();
			let value = stack.peek(1)?.to_h256();

			// https://eips.ethereum.org/EIPS/eip-2929
			let target_is_cold = handler.is_cold(address, Some(index));
			handler.mark_storage_hot(address, index);

			GasCost::SStore {
				original: handler.original_storage(address, index),
				current: handler.storage(address, index),
				new: value,
				target_is_cold,
			}
		}
		Opcode::TSTORE if !is_static && config.eip1153_transient_storage => GasCost::TStore,
		Opcode::LOG0 if !is_static => GasCost::Log {
			n: 0,
			len: stack.peek(1)?,
		},
		Opcode::LOG1 if !is_static => GasCost::Log {
			n: 1,
			len: stack.peek(1)?,
		},
		Opcode::LOG2 if !is_static => GasCost::Log {
			n: 2,
			len: stack.peek(1)?,
		},
		Opcode::LOG3 if !is_static => GasCost::Log {
			n: 3,
			len: stack.peek(1)?,
		},
		Opcode::LOG4 if !is_static => GasCost::Log {
			n: 4,
			len: stack.peek(1)?,
		},
		Opcode::CREATE if !is_static => GasCost::Create {
			len: stack.peek(2)?,
		},
		Opcode::CREATE2 if !is_static && config.eip1014_create2 => GasCost::Create2 {
			len: stack.peek(2)?,
		},
		Opcode::SUICIDE if !is_static => {
			let target = stack.peek(0)?.to_h160();
			let target_exists = handler.exists(target);

			// https://eips.ethereum.org/EIPS/eip-2929
			let target_is_cold = handler.is_cold(target, None);
			handler.mark_hot(target, TouchKind::Access);
			handler.mark_hot(target, TouchKind::StateChange);

			GasCost::Suicide {
				value: handler.balance(address),
				target_is_cold,
				target_exists,
				already_removed: handler.deleted(address),
			}
		}
		Opcode::CALL if !is_static || (is_static && stack.peek(2)? == U256::ZERO) => {
			let target = stack.peek(1)?.to_h160();
			let target_exists = handler.exists(target);

			// https://eips.ethereum.org/EIPS/eip-2929
			let target_is_cold = handler.is_cold(target, None);
			handler.mark_hot(target, TouchKind::Access);

			GasCost::Call {
				value: stack.peek(2)?,
				gas: stack.peek(0)?,
				target_is_cold,
				target_exists,
			}
		}

		Opcode::PUSH0 if config.eip3855_push0 => GasCost::Base,

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
			offset: stack.peek(0)?,
			len: stack.peek(1)?,
		}),

		Opcode::MCOPY => {
			let top0 = stack.peek(0)?;
			let top1 = stack.peek(1)?;
			let offset = top0.max(top1);
			Some(MemoryCost {
				offset,
				len: stack.peek(2)?,
			})
		}

		Opcode::CODECOPY | Opcode::CALLDATACOPY | Opcode::RETURNDATACOPY => Some(MemoryCost {
			offset: stack.peek(0)?,
			len: stack.peek(2)?,
		}),

		Opcode::EXTCODECOPY => Some(MemoryCost {
			offset: stack.peek(1)?,
			len: stack.peek(3)?,
		}),

		Opcode::MLOAD | Opcode::MSTORE => Some(MemoryCost {
			offset: stack.peek(0)?,
			len: U256::from(32),
		}),

		Opcode::MSTORE8 => Some(MemoryCost {
			offset: stack.peek(0)?,
			len: U256::from(1),
		}),

		Opcode::CREATE | Opcode::CREATE2 => Some(MemoryCost {
			offset: stack.peek(1)?,
			len: stack.peek(2)?,
		}),

		Opcode::CALL | Opcode::CALLCODE => Some(
			MemoryCost {
				offset: stack.peek(3)?,
				len: stack.peek(4)?,
			}
			.join(MemoryCost {
				offset: stack.peek(5)?,
				len: stack.peek(6)?,
			}),
		),

		Opcode::DELEGATECALL | Opcode::STATICCALL => Some(
			MemoryCost {
				offset: stack.peek(2)?,
				len: stack.peek(3)?,
			}
			.join(MemoryCost {
				offset: stack.peek(4)?,
				len: stack.peek(5)?,
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
	/// Gas cost for `TLOAD`.
	TLoad,
	/// Gas cost for `TSTORE`.
	TStore,
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
	Create {
		/// Length.
		len: U256,
	},
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
				U256::ZERO,
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
				U256::ZERO,
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
			GasCost::TLoad => costs::tload_cost(config)?,
			GasCost::TStore => costs::tstore_cost(config)?,
			GasCost::Sha3 { len } => costs::sha3_cost(len)?,
			GasCost::Log { n, len } => costs::log_cost(n, len)?,
			GasCost::VeryLowCopy { len } => costs::verylowcopy_cost(len)?,
			GasCost::Exp { power } => costs::exp_cost(power, config)?,
			GasCost::Create { len } => {
				let base = consts::G_CREATE;
				if config.eip3860_max_initcode_size {
					let len = u256_to_usize(len)?;
					base + init_code_cost(len)
				} else {
					base
				}
			}
			GasCost::Create2 { len } => {
				let base = costs::create2_cost(len)?;
				if config.eip3860_max_initcode_size {
					let len = u256_to_usize(len)?;
					base + init_code_cost(len)
				} else {
					base
				}
			}
			GasCost::SLoad { target_is_cold } => costs::sload_cost(target_is_cold, config),

			GasCost::Zero => consts::G_ZERO,
			GasCost::Base => consts::G_BASE,
			GasCost::VeryLow => consts::G_VERYLOW,
			GasCost::Low => consts::G_LOW,
			GasCost::Invalid(opcode) => return Err(ExitException::InvalidOpcode(opcode).into()),

			GasCost::ExtCodeSize { target_is_cold } => {
				costs::address_access_cost(target_is_cold, config.gas_ext_code(), config)
			}
			GasCost::ExtCodeCopy {
				target_is_cold,
				len,
			} => costs::extcodecopy_cost(len, target_is_cold, config)?,
			GasCost::Balance { target_is_cold } => {
				costs::address_access_cost(target_is_cold, config.gas_balance(), config)
			}
			GasCost::BlockHash => consts::G_BLOCKHASH,
			GasCost::ExtCodeHash { target_is_cold } => {
				costs::address_access_cost(target_is_cold, config.gas_ext_code_hash(), config)
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
			} if !config.eip3529_decrease_clears_refund => costs::suicide_refund(already_removed),
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
		if self.len == U256::ZERO {
			return other;
		}

		if other.len == U256::ZERO {
			return self;
		}

		let self_end = self.offset.saturating_add(self.len);
		let other_end = other.offset.saturating_add(other.len);

		if self_end >= other_end { self } else { other }
	}

	/// Numeric value of the cost.
	pub fn cost(&self) -> Result<Option<u64>, ExitError> {
		let from = self.offset;
		let len = self.len;

		if len == U256::ZERO {
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

pub struct TransactionGas {
	used: u64,
	floor: u64,
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
		let initcode_cost = init_code_cost(data.len());

		TransactionCost::Create {
			zero_data_len,
			non_zero_data_len,
			access_list_address_len,
			access_list_storage_len,
			initcode_cost,
		}
	}

	pub fn cost(&self, config: &Config) -> TransactionGas {
		match self {
			TransactionCost::Call {
				zero_data_len,
				non_zero_data_len,
				access_list_address_len,
				access_list_storage_len,
			} => {
				let used = config.gas_transaction_call()
					+ *zero_data_len as u64 * config.gas_transaction_zero_data()
					+ *non_zero_data_len as u64 * config.gas_transaction_non_zero_data()
					+ *access_list_address_len as u64 * config.gas_access_list_address()
					+ *access_list_storage_len as u64 * config.gas_access_list_storage_key();

				let floor = config
					.gas_transaction_call()
					.saturating_add(
						(*zero_data_len as u64)
							.saturating_mul(config.gas_floor_transaction_zero_data()),
					)
					.saturating_add(
						(*non_zero_data_len as u64)
							.saturating_mul(config.gas_floor_transaction_non_zero_data()),
					);

				TransactionGas { used, floor }
			}
			TransactionCost::Create {
				zero_data_len,
				non_zero_data_len,
				access_list_address_len,
				access_list_storage_len,
				initcode_cost,
			} => {
				let mut used = config.gas_transaction_create()
					+ *zero_data_len as u64 * config.gas_transaction_zero_data()
					+ *non_zero_data_len as u64 * config.gas_transaction_non_zero_data()
					+ *access_list_address_len as u64 * config.gas_access_list_address()
					+ *access_list_storage_len as u64 * config.gas_access_list_storage_key();

				if config.max_initcode_size().is_some() {
					used += initcode_cost;
				}

				let floor = config
					.gas_transaction_call()
					.saturating_add(
						(*zero_data_len as u64)
							.saturating_mul(config.gas_floor_transaction_zero_data()),
					)
					.saturating_add(
						(*non_zero_data_len as u64)
							.saturating_mul(config.gas_floor_transaction_non_zero_data()),
					);

				TransactionGas { used, floor }
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

fn init_code_cost(len: usize) -> u64 {
	// As per EIP-3860:
	// > We define initcode_cost(initcode) to equal INITCODE_WORD_COST * ceil(len(initcode) / 32).
	// where INITCODE_WORD_COST is 2.
	2 * ((len as u64).div_ceil(32))
}
