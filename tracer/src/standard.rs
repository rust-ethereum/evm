use evm::{
	interpreter::opcode::Opcode,
	standard::{Machine, State},
};

pub trait EvalTracer<H> {
	fn on_eval(&mut self, machine: &Machine, handle: &H, opcode: Opcode, position: usize);
}

impl<'config, H, T: EvalTracer<H>> crate::EvalTracer<State<'config>, H> for T {
	fn on_eval(&mut self, machine: &Machine, handle: &H, opcode: Opcode, position: usize) {
		self.on_eval(machine, handle, opcode, position)
	}
}
