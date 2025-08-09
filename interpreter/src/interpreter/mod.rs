mod etable;
mod valids;

use alloc::vec::Vec;

pub use self::etable::EtableInterpreter;
use crate::{
	error::{Capture, ExitResult, ExitError, Trap},
};

pub trait Interpreter<H> {
	type State;
	type Trap: Trap<Self>;

	fn deconstruct(self) -> (Self::State, Vec<u8>);
	fn feedback(&mut self, trap: Self::Trap, feedback: <Self::Trap as Trap<Self>>::Feedback) -> Result<(), ExitError> {
		trap.feedback(feedback, self)
	}
	fn run(&mut self, handle: &mut H) -> Capture<ExitResult, Self::Trap>;
}

pub trait StepInterpreter<H>: Interpreter<H> {
	fn step(&mut self, handle: &mut H) -> Result<(), Capture<ExitResult, Self::Trap>>;
}
