//! Runtime layer for EVM.

#![deny(warnings)]
#![forbid(unsafe_code, unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "tracing")]
pub mod tracing;

#[cfg(feature = "tracing")]
macro_rules! event {
	($x:expr) => {
		use crate::tracing::Event::*;
		crate::tracing::with(|listener| listener.event($x));
	};
}

#[cfg(not(feature = "tracing"))]
macro_rules! event {
	($x:expr) => {};
}

mod context;
mod eval;
mod handler;
mod interrupt;

pub use evm_core::*;

pub use crate::context::{CallScheme, Context, CreateScheme};
pub use crate::handler::{Handler, Transfer};
pub use crate::interrupt::{Resolve, ResolveCall, ResolveCreate};

use alloc::rc::Rc;
use alloc::vec::Vec;
use primitive_types::{H160, U256};

macro_rules! step {
	( $self:expr, $handler:expr, $return:tt $($err:path)?; $($ok:path)? ) => ({
		if let Some((opcode, stack)) = $self.machine.inspect() {
			event!(Step {
				context: &$self.context,
				opcode,
				position: $self.machine.position(),
				stack,
				memory: $self.machine.memory()
			});

			match $handler.pre_validate(&$self.context, opcode, stack) {
				Ok(()) => (),
				Err(e) => {
					$self.machine.exit(e.clone().into());
					$self.status = Err(e.into());
				},
			}
		}

		match &$self.status {
			Ok(()) => (),
			Err(e) => {
				#[allow(unused_parens)]
				$return $($err)*(Capture::Exit(e.clone()))
			},
		}

		let result = $self.machine.step();

		event!(StepResult {
			result: &result,
			return_value: &$self.machine.return_value(),
		});

		match result {
			Ok(()) => $($ok)?(()),
			Err(Capture::Exit(e)) => {
				$self.status = Err(e.clone());
				#[allow(unused_parens)]
				$return $($err)*(Capture::Exit(e))
			},
			Err(Capture::Trap(opcode)) => {
				match eval::eval($self, opcode, $handler) {
					eval::Control::Continue => $($ok)?(()),
					eval::Control::CallInterrupt(interrupt) => {
						let resolve = ResolveCall::new($self);
						#[allow(unused_parens)]
						$return $($err)*(Capture::Trap(Resolve::Call(interrupt, resolve)))
					},
					eval::Control::CreateInterrupt(interrupt) => {
						let resolve = ResolveCreate::new($self);
						#[allow(unused_parens)]
						$return $($err)*(Capture::Trap(Resolve::Create(interrupt, resolve)))
					},
					eval::Control::Exit(exit) => {
						$self.machine.exit(exit.clone().into());
						$self.status = Err(exit.clone());
						#[allow(unused_parens)]
						$return $($err)*(Capture::Exit(exit))
					},
				}
			},
		}
	});
}

/// EVM runtime.
///
/// The runtime wraps an EVM `Machine` with support of return data and context.
pub struct Runtime {
	machine: Machine,
	status: Result<(), ExitReason>,
	return_data_buffer: Vec<u8>,
	return_data_len: U256,
	return_data_offset: U256,
	context: Context,
}

impl Runtime {
	/// Create a new runtime with given code and data.
	pub fn new(
		code: Rc<Vec<u8>>,
		data: Rc<Vec<u8>>,
		context: Context,
		stack_limit: usize,
		memory_limit: usize,
	) -> Self {
		Self {
			machine: Machine::new(code, data, stack_limit, memory_limit),
			status: Ok(()),
			return_data_buffer: Vec::new(),
			return_data_len: U256::zero(),
			return_data_offset: U256::zero(),
			context,
		}
	}

	/// Get a reference to the machine.
	pub fn machine(&self) -> &Machine {
		&self.machine
	}

	/// Get a reference to the execution context.
	pub fn context(&self) -> &Context {
		&self.context
	}

	/// Step the runtime.
	pub fn step<'a, H: Handler>(
		&'a mut self,
		handler: &mut H,
	) -> Result<(), Capture<ExitReason, Resolve<'a, H>>> {
		step!(self, handler, return Err; Ok)
	}

	/// Loop stepping the runtime until it stops.
	pub fn run<'a, H: Handler>(
		&'a mut self,
		handler: &mut H,
	) -> Capture<ExitReason, Resolve<'a, H>> {
		loop {
			step!(self, handler, return;)
		}
	}

	pub fn finish_create(
		&mut self,
		reason: ExitReason,
		address: Option<H160>,
		return_data: Vec<u8>,
	) -> Result<(), ExitReason> {
		eval::finish_create(self, reason, address, return_data)
	}

	pub fn finish_call(
		&mut self,
		reason: ExitReason,
		return_data: Vec<u8>,
	) -> Result<(), ExitReason> {
		eval::finish_call(
			self,
			self.return_data_len,
			self.return_data_offset,
			reason,
			return_data,
		)
	}
}

/// Runtime configuration.
#[derive(Clone, Debug)]
pub struct Config {
	/// Gas paid for extcode.
	pub gas_ext_code: u64,
	/// Gas paid for extcodehash.
	pub gas_ext_code_hash: u64,
	/// Gas paid for sstore set.
	pub gas_sstore_set: u64,
	/// Gas paid for sstore reset.
	pub gas_sstore_reset: u64,
	/// Gas paid for sstore refund.
	pub refund_sstore_clears: i64,
	/// EIP-3529
	pub max_refund_quotient: u64,
	/// Gas paid for BALANCE opcode.
	pub gas_balance: u64,
	/// Gas paid for SLOAD opcode.
	pub gas_sload: u64,
	/// Gas paid for cold SLOAD opcode.
	pub gas_sload_cold: u64,
	/// Gas paid for SUICIDE opcode.
	pub gas_suicide: u64,
	/// Gas paid for SUICIDE opcode when it hits a new account.
	pub gas_suicide_new_account: u64,
	/// Gas paid for CALL opcode.
	pub gas_call: u64,
	/// Gas paid for EXP opcode for every byte.
	pub gas_expbyte: u64,
	/// Gas paid for a contract creation transaction.
	pub gas_transaction_create: u64,
	/// Gas paid for a message call transaction.
	pub gas_transaction_call: u64,
	/// Gas paid for zero data in a transaction.
	pub gas_transaction_zero_data: u64,
	/// Gas paid for non-zero data in a transaction.
	pub gas_transaction_non_zero_data: u64,
	/// Gas paid per address in transaction access list (see EIP-2930).
	pub gas_access_list_address: u64,
	/// Gas paid per storage key in transaction access list (see EIP-2930).
	pub gas_access_list_storage_key: u64,
	/// Gas paid for accessing cold account.
	pub gas_account_access_cold: u64,
	/// Gas paid for accessing ready storage.
	pub gas_storage_read_warm: u64,
	/// EIP-1283.
	pub sstore_gas_metering: bool,
	/// EIP-1706.
	pub sstore_revert_under_stipend: bool,
	/// EIP-2929
	pub increase_state_access_gas: bool,
	/// EIP-3529
	pub decrease_clears_refund: bool,
	/// EIP-3541
	pub disallow_executable_format: bool,
	/// EIP-3651
	pub warm_coinbase_address: bool,
	/// Whether to throw out of gas error when
	/// CALL/CALLCODE/DELEGATECALL requires more than maximum amount
	/// of gas.
	pub err_on_call_with_more_gas: bool,
	/// Take l64 for callcreate after gas.
	pub call_l64_after_gas: bool,
	/// Whether empty account is considered exists.
	pub empty_considered_exists: bool,
	/// Whether create transactions and create opcode increases nonce by one.
	pub create_increase_nonce: bool,
	/// Stack limit.
	pub stack_limit: usize,
	/// Memory limit.
	pub memory_limit: usize,
	/// Call limit.
	pub call_stack_limit: usize,
	/// Create contract limit.
	pub create_contract_limit: Option<usize>,
	/// EIP-3860, maximum size limit of init_code.
	pub max_initcode_size: Option<usize>,
	/// Call stipend.
	pub call_stipend: u64,
	/// Has delegate call.
	pub has_delegate_call: bool,
	/// Has create2.
	pub has_create2: bool,
	/// Has revert.
	pub has_revert: bool,
	/// Has return data.
	pub has_return_data: bool,
	/// Has bitwise shifting.
	pub has_bitwise_shifting: bool,
	/// Has chain ID.
	pub has_chain_id: bool,
	/// Has self balance.
	pub has_self_balance: bool,
	/// Has ext code hash.
	pub has_ext_code_hash: bool,
	/// Has ext block fee. See [EIP-3198](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-3198.md)
	pub has_base_fee: bool,
	/// Has PUSH0 opcode. See [EIP-3855](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-3855.md)
	pub has_push0: bool,
	/// Whether the gasometer is running in estimate mode.
	pub estimate: bool,
}

impl Config {
	/// Frontier hard fork configuration.
	pub const fn frontier() -> Config {
		Config {
			gas_ext_code: 20,
			gas_ext_code_hash: 20,
			gas_balance: 20,
			gas_sload: 50,
			gas_sload_cold: 0,
			gas_sstore_set: 20000,
			gas_sstore_reset: 5000,
			refund_sstore_clears: 15000,
			max_refund_quotient: 2,
			gas_suicide: 0,
			gas_suicide_new_account: 0,
			gas_call: 40,
			gas_expbyte: 10,
			gas_transaction_create: 21000,
			gas_transaction_call: 21000,
			gas_transaction_zero_data: 4,
			gas_transaction_non_zero_data: 68,
			gas_access_list_address: 0,
			gas_access_list_storage_key: 0,
			gas_account_access_cold: 0,
			gas_storage_read_warm: 0,
			sstore_gas_metering: false,
			sstore_revert_under_stipend: false,
			increase_state_access_gas: false,
			decrease_clears_refund: false,
			disallow_executable_format: false,
			warm_coinbase_address: false,
			err_on_call_with_more_gas: true,
			empty_considered_exists: true,
			create_increase_nonce: false,
			call_l64_after_gas: false,
			stack_limit: 1024,
			memory_limit: usize::MAX,
			call_stack_limit: 1024,
			create_contract_limit: None,
			max_initcode_size: None,
			call_stipend: 2300,
			has_delegate_call: false,
			has_create2: false,
			has_revert: false,
			has_return_data: false,
			has_bitwise_shifting: false,
			has_chain_id: false,
			has_self_balance: false,
			has_ext_code_hash: false,
			has_base_fee: false,
			has_push0: false,
			estimate: false,
		}
	}

	/// Istanbul hard fork configuration.
	pub const fn istanbul() -> Config {
		Config {
			gas_ext_code: 700,
			gas_ext_code_hash: 700,
			gas_balance: 700,
			gas_sload: 800,
			gas_sload_cold: 0,
			gas_sstore_set: 20000,
			gas_sstore_reset: 5000,
			refund_sstore_clears: 15000,
			max_refund_quotient: 2,
			gas_suicide: 5000,
			gas_suicide_new_account: 25000,
			gas_call: 700,
			gas_expbyte: 50,
			gas_transaction_create: 53000,
			gas_transaction_call: 21000,
			gas_transaction_zero_data: 4,
			gas_transaction_non_zero_data: 16,
			gas_access_list_address: 0,
			gas_access_list_storage_key: 0,
			gas_account_access_cold: 0,
			gas_storage_read_warm: 0,
			sstore_gas_metering: true,
			sstore_revert_under_stipend: true,
			increase_state_access_gas: false,
			decrease_clears_refund: false,
			disallow_executable_format: false,
			warm_coinbase_address: false,
			err_on_call_with_more_gas: false,
			empty_considered_exists: false,
			create_increase_nonce: true,
			call_l64_after_gas: true,
			stack_limit: 1024,
			memory_limit: usize::MAX,
			call_stack_limit: 1024,
			create_contract_limit: Some(0x6000),
			max_initcode_size: None,
			call_stipend: 2300,
			has_delegate_call: true,
			has_create2: true,
			has_revert: true,
			has_return_data: true,
			has_bitwise_shifting: true,
			has_chain_id: true,
			has_self_balance: true,
			has_ext_code_hash: true,
			has_base_fee: false,
			has_push0: false,
			estimate: false,
		}
	}

	/// Berlin hard fork configuration.
	pub const fn berlin() -> Config {
		Self::config_with_derived_values(DerivedConfigInputs::berlin())
	}

	/// london hard fork configuration.
	pub const fn london() -> Config {
		Self::config_with_derived_values(DerivedConfigInputs::london())
	}

	/// The Merge (Paris) hard fork configuration.
	pub const fn merge() -> Config {
		Self::config_with_derived_values(DerivedConfigInputs::merge())
	}

	/// Shanghai hard fork configuration.
	pub const fn shanghai() -> Config {
		Self::config_with_derived_values(DerivedConfigInputs::shanghai())
	}

	const fn config_with_derived_values(inputs: DerivedConfigInputs) -> Config {
		let DerivedConfigInputs {
			gas_storage_read_warm,
			gas_sload_cold,
			gas_access_list_storage_key,
			decrease_clears_refund,
			has_base_fee,
			has_push0,
			disallow_executable_format,
			warm_coinbase_address,
			max_initcode_size,
		} = inputs;

		// See https://eips.ethereum.org/EIPS/eip-2929
		let gas_sload = gas_storage_read_warm;
		let gas_sstore_reset = 5000 - gas_sload_cold;

		// See https://eips.ethereum.org/EIPS/eip-3529
		let refund_sstore_clears = if decrease_clears_refund {
			(gas_sstore_reset + gas_access_list_storage_key) as i64
		} else {
			15000
		};
		let max_refund_quotient = if decrease_clears_refund { 5 } else { 2 };

		Config {
			gas_ext_code: 0,
			gas_ext_code_hash: 0,
			gas_balance: 0,
			gas_sload,
			gas_sload_cold,
			gas_sstore_set: 20000,
			gas_sstore_reset,
			refund_sstore_clears,
			max_refund_quotient,
			gas_suicide: 5000,
			gas_suicide_new_account: 25000,
			gas_call: 0,
			gas_expbyte: 50,
			gas_transaction_create: 53000,
			gas_transaction_call: 21000,
			gas_transaction_zero_data: 4,
			gas_transaction_non_zero_data: 16,
			gas_access_list_address: 2400,
			gas_access_list_storage_key,
			gas_account_access_cold: 2600,
			gas_storage_read_warm,
			sstore_gas_metering: true,
			sstore_revert_under_stipend: true,
			increase_state_access_gas: true,
			decrease_clears_refund,
			disallow_executable_format,
			warm_coinbase_address,
			err_on_call_with_more_gas: false,
			empty_considered_exists: false,
			create_increase_nonce: true,
			call_l64_after_gas: true,
			stack_limit: 1024,
			memory_limit: usize::MAX,
			call_stack_limit: 1024,
			create_contract_limit: Some(0x6000),
			max_initcode_size,
			call_stipend: 2300,
			has_delegate_call: true,
			has_create2: true,
			has_revert: true,
			has_return_data: true,
			has_bitwise_shifting: true,
			has_chain_id: true,
			has_self_balance: true,
			has_ext_code_hash: true,
			has_base_fee,
			has_push0,
			estimate: false,
		}
	}
}

/// Independent inputs that are used to derive other config values.
/// See `Config::config_with_derived_values` implementation for details.
struct DerivedConfigInputs {
	gas_storage_read_warm: u64,
	gas_sload_cold: u64,
	gas_access_list_storage_key: u64,
	decrease_clears_refund: bool,
	has_base_fee: bool,
	has_push0: bool,
	disallow_executable_format: bool,
	warm_coinbase_address: bool,
	max_initcode_size: Option<usize>,
}

impl DerivedConfigInputs {
	const fn berlin() -> Self {
		Self {
			gas_storage_read_warm: 100,
			gas_sload_cold: 2100,
			gas_access_list_storage_key: 1900,
			decrease_clears_refund: false,
			has_base_fee: false,
			has_push0: false,
			disallow_executable_format: false,
			warm_coinbase_address: false,
			max_initcode_size: None,
		}
	}

	const fn london() -> Self {
		Self {
			gas_storage_read_warm: 100,
			gas_sload_cold: 2100,
			gas_access_list_storage_key: 1900,
			decrease_clears_refund: true,
			has_base_fee: true,
			has_push0: false,
			disallow_executable_format: true,
			warm_coinbase_address: false,
			max_initcode_size: None,
		}
	}

	const fn merge() -> Self {
		Self {
			gas_storage_read_warm: 100,
			gas_sload_cold: 2100,
			gas_access_list_storage_key: 1900,
			decrease_clears_refund: true,
			has_base_fee: true,
			has_push0: false,
			disallow_executable_format: true,
			warm_coinbase_address: false,
			max_initcode_size: None,
		}
	}

	const fn shanghai() -> Self {
		Self {
			gas_storage_read_warm: 100,
			gas_sload_cold: 2100,
			gas_access_list_storage_key: 1900,
			decrease_clears_refund: true,
			has_base_fee: true,
			has_push0: true,
			disallow_executable_format: true,
			warm_coinbase_address: true,
			// 2 * 24576 as per EIP-3860
			max_initcode_size: Some(0xC000),
		}
	}
}
