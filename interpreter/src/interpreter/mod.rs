mod etable;
mod valids;

use alloc::{boxed::Box, vec::Vec};

pub use self::etable::EtableInterpreter;
pub use self::valids::Valids;

use crate::{Capture, ExitError, ExitResult};

/// Control state.
#[derive(Eq, PartialEq, Debug)]
pub enum Control<Trap> {
	/// No action.
	NoAction,
	/// Continue the execution, increase the PC by N.
	Continue(usize),
	/// Exit the execution.
	Exit(ExitResult),
	/// Jump to the specified PC.
	Jump(usize),
	/// Trapping the execution with the possibility to resume.
	Trap(Box<Trap>),
}

pub trait Interpreter<H> {
	type State;
	type Trap;

	fn deconstruct(self) -> (Self::State, Vec<u8>);
	fn state(&self) -> &Self::State;
	fn state_mut(&mut self) -> &mut Self::State;
	fn run(&mut self, handle: &mut H) -> Capture<ExitResult, Self::Trap>;
}

pub trait FeedbackInterpreter<H, Feedback>: Interpreter<H> {
	fn feedback(&mut self, feedback: Feedback, handler: &mut H) -> Result<(), ExitError>;
}

pub trait StepInterpreter<H>: Interpreter<H> {
	fn step(&mut self, handle: &mut H) -> Result<(), Capture<ExitResult, Self::Trap>>;
}
