mod etable;

pub use self::etable::EtableInterpreter;

use crate::{Capture, ExitResult, Machine};
use alloc::vec::Vec;

pub trait Interpreter<S, H, Tr> {
	fn machine(&self) -> &Machine<S>;
	fn machine_mut(&mut self) -> &mut Machine<S>;

	fn deconstruct(self) -> (S, Vec<u8>);
	fn run(&mut self, handle: &mut H) -> Capture<ExitResult, Tr>;
	fn advance(&mut self);
}

pub trait StepInterpreter<S, H, Tr>: Interpreter<S, H, Tr> {
	fn step(&mut self, handle: &mut H) -> Result<(), Capture<ExitResult, Tr>>;
}
