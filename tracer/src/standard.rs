use evm::standard::Machine;
use evm::{Opcode, RuntimeState};

pub trait EvalTracer<H> {
	fn on_eval(&mut self, machine: &Machine, handle: &H, opcode: Opcode, position: usize);
}

impl<H, T: EvalTracer<H>> crate::EvalTracer<RuntimeState, H> for T {
	fn on_eval(&mut self, machine: &Machine, handle: &H, opcode: Opcode, position: usize) {
		EvalTracer::<H>::on_eval(self, machine, handle, opcode, position)
	}
}
