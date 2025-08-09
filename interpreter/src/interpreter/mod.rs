mod etable;
mod valids;

use alloc::vec::Vec;

pub use self::etable::EtableInterpreter;
use crate::error::{Capture, ExitError, ExitResult};

pub trait Interpreter<H> {
	type State;
	type Trap;

	fn deconstruct(self) -> (Self::State, Vec<u8>);
	fn run(&mut self, handle: &mut H) -> Capture<ExitResult, Self::Trap>;
}

pub trait FeedbackInterpreter<H, Feedback>: Interpreter<H> {
	fn feedback(&mut self, feedback: Feedback, handler: &mut H) -> Result<(), ExitError>;
}

pub trait StepInterpreter<H>: Interpreter<H> {
	fn step(&mut self, handle: &mut H) -> Result<(), Capture<ExitResult, Self::Trap>>;
}
