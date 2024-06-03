mod etable;
mod valids;

use alloc::vec::Vec;

pub use self::etable::EtableInterpreter;
use crate::{
	error::{Capture, ExitResult},
	machine::Machine,
};

pub trait Interpreter {
	type State;

	fn machine(&self) -> &Machine<Self::State>;
	fn machine_mut(&mut self) -> &mut Machine<Self::State>;

	fn deconstruct(self) -> (Self::State, Vec<u8>);
	fn advance(&mut self);
}

pub trait RunInterpreter<H, Tr>: Interpreter {
	fn run(&mut self, handle: &mut H) -> Capture<ExitResult, Tr>;
}

pub trait StepInterpreter<H, Tr>: Interpreter {
	fn step(&mut self, handle: &mut H) -> Result<(), Capture<ExitResult, Tr>>;
}
