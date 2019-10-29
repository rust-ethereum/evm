#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod eval;
mod context;
mod interrupt;
mod handler;

pub use evm_core::*;

pub use crate::context::{CreateScheme, CallScheme, Context};
pub use crate::interrupt::{Resolve, ResolveCall, ResolveCreate};
pub use crate::handler::Handler;

use alloc::vec::Vec;
use alloc::rc::Rc;

macro_rules! step {
	( $self:expr, $handler:expr, $return:tt $($err:path)?; $($ok:path)? ) => ({
		if let Some((opcode, stack)) = $self.machine.inspect() {
			match $handler.pre_validate(&$self.context, opcode, stack) {
				Ok(()) => (),
				Err(error) => {
					$self.machine.exit(Err(error));
					$self.status = Err(Err(error));
				},
			}
		}

		match $self.status.clone() {
			Ok(()) => (),
			Err(exit) => {
				#[allow(unused_parens)]
				$return $($err)*(Capture::Exit(exit))
			},
		}

		match $self.machine.step() {
			Ok(()) => $($ok)?(()),
			Err(Capture::Exit(exit)) => {
				$self.status = Err(exit);
				#[allow(unused_parens)]
				$return $($err)*(Capture::Exit(exit))
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
						$self.machine.exit(exit.into());
						$self.status = Err(exit);
						#[allow(unused_parens)]
						$return $($err)*(Capture::Exit(exit))
					},
				}
			},
		}
	});
}

pub struct Runtime {
	machine: Machine,
	status: Result<(), ExitReason>,
	return_data_buffer: Vec<u8>,
	context: Context,
}

impl Runtime {
	pub fn new(
		code: Rc<Vec<u8>>,
		data: Rc<Vec<u8>>,
		stack_limit: usize,
		memory_limit: usize,
		context: Context,
	) -> Self {
		Self {
			machine: Machine::new(code, data, stack_limit, memory_limit),
			status: Ok(()),
			return_data_buffer: Vec::new(),
			context,
		}
	}

	pub fn machine(&self) -> &Machine {
		&self.machine
	}

	pub fn step<'a, H: Handler>(
		&'a mut self,
		handler: &mut H,
	) -> Result<(), Capture<ExitReason, Resolve<'a, H>>> {
		step!(self, handler, return Err; Ok)
	}

	pub fn run<'a, H: Handler>(
		&'a mut self,
		handler: &mut H,
	) -> Capture<ExitReason, Resolve<'a, H>> {
		loop {
			step!(self, handler, return;)
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
	/// Gas paid for a message call transaction.
	pub gas_transaction_call: usize,
	/// Gas paid for zero data in a transaction.
	pub gas_transaction_zero_data: usize,
	/// Gas paid for non-zero data in a transaction.
	pub gas_transaction_non_zero_data: usize,
	/// EIP-1283.
	pub has_reduced_sstore_gas_metering: bool,
	/// Whether to throw out of gas error when
	/// CALL/CALLCODE/DELEGATECALL requires more than maximum amount
	/// of gas.
	pub err_on_call_with_more_gas: bool,
	/// Whether empty account is considered exists.
	pub empty_considered_exists: bool,
	/// Whether create transactions and create opcode increases nonce by one.
	pub create_increase_nonce: bool,
	/// Stack limit.
	pub stack_limit: usize,
	/// Memory limit.
	pub memory_limit: usize,
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
			gas_transaction_create: 21000,
			gas_transaction_call: 21000,
			gas_transaction_zero_data: 4,
			gas_transaction_non_zero_data: 68,
			has_reduced_sstore_gas_metering: false,
			err_on_call_with_more_gas: true,
			empty_considered_exists: true,
			create_increase_nonce: false,
			stack_limit: 1024,
			memory_limit: usize::max_value(),
		}
	}

	pub const fn istanbul() -> Config {
		Config {
			gas_extcode: 700,
			gas_balance: 400,
			gas_sload: 800,
			gas_suicide: 5000,
			gas_suicide_new_account: 25000,
			gas_call: 700,
			gas_expbyte: 50,
			gas_transaction_create: 53000,
			gas_transaction_call: 21000,
			gas_transaction_zero_data: 4,
			gas_transaction_non_zero_data: 16,
			has_reduced_sstore_gas_metering: true,
			err_on_call_with_more_gas: false,
			empty_considered_exists: false,
			create_increase_nonce: true,
			stack_limit: 1024,
			memory_limit: usize::max_value(),
		}
	}
}
