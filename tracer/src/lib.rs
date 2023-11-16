mod standard;

use evm::{Machine, Opcode};

pub trait EvalTracer<S, H> {
	fn on_eval(&mut self, machine: &Machine<S>, handle: &H, opcode: Opcode, position: usize);
}
