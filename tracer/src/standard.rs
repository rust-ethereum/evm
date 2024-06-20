use evm::{
	interpreter::opcode::Opcode,
	standard::{Machine, State},
};

pub trait EvalTracer<H> {
	fn on_eval(&mut self, machine: &Machine, handle: &H, opcode: Opcode, position: usize);
}

impl<'config, H, T: EvalTracer<H>> crate::EvalTracer<State<'config>, H> for T {
	fn on_eval(&mut self, machine: &Machine, handle: &H, opcode: Opcode, position: usize) {
		EvalTracer::<H>::on_eval(self, machine, handle, opcode, position)
	}
}
