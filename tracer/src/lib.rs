pub mod standard;

use evm::interpreter::{Machine, Opcode};

pub trait EvalTracer<S, H> {
	fn on_eval(&mut self, machine: &Machine<S>, handle: &H, opcode: Opcode, position: usize);
}
