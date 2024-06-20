mod standard;

use evm::interpreter::{machine::Machine, opcode::Opcode};

pub trait EvalTracer<S, H> {
	fn on_eval(&mut self, machine: &Machine<S>, handle: &H, opcode: Opcode, position: usize);
}
