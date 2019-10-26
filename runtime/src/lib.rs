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
					$self.machine.exit(error.into());
					$self.status = Err(error.into());
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
