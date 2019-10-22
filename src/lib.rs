mod eval;
mod context;
mod interrupt;
mod handler;

pub use evm_core::*;
pub use evm_gasometer::*;

pub use crate::context::{CreateScheme, CallScheme, Context};
pub use crate::interrupt::{Resolve, ResolveCall, ResolveCreate};
pub use crate::handler::Handler;

pub struct Runtime<'context> {
	machine: Machine,
	status: Result<(), ExitReason>,
	return_data_buffer: Vec<u8>,
	context: &'context Context,
}

impl<'context> Runtime<'context> {
	pub fn step<H: Handler>(
		mut self,
		handler: &mut H,
	) -> Result<Self, Capture<(Self, ExitReason), Resolve<'context, H>>> {
		if let Some((opcode, stack)) = self.machine.inspect() {
			match handler.pre_validate(opcode, stack) {
				Ok(()) => (),
				Err(error) => {
					self.machine.exit(error.into());
					self.status = Err(error.into());
				},
			}
		}

		match self.status.clone() {
			Ok(()) => (),
			Err(exit) => return Err(Capture::Exit((self, exit))),
		}

		match self.machine.step() {
			Ok(()) => Ok(self),
			Err(Capture::Exit(exit)) => {
				self.status = Err(exit);
				Err(Capture::Exit((self, exit)))
			},
			Err(Capture::Trap(opcode)) => {
				match eval::eval(&mut self, opcode, handler) {
					eval::Control::Continue => Ok(self),
					eval::Control::CallInterrupt(interrupt) => {
						let resolve = ResolveCall::new(self, interrupt);
						Err(Capture::Trap(Resolve::Call(resolve)))
					},
					eval::Control::CreateInterrupt(interrupt) => {
						let resolve = ResolveCreate::new(self, interrupt);
						Err(Capture::Trap(Resolve::Create(resolve)))
					},
					eval::Control::Exit(exit) => {
						self.machine.exit(exit.into());
						self.status = Err(exit);
						Err(Capture::Exit((self, exit)))
					},
				}
			},
		}
	}

	pub fn run<H: Handler>(
		self,
		handler: &mut H,
	) -> Capture<(Self, ExitReason), Resolve<'context, H>> {
		let mut current = self;

		loop {
			match current.step(handler) {
				Ok(value) => {
					current = value
				},
				Err(capture) => return capture,
			}
		}
	}
}
