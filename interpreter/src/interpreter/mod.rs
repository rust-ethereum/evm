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

/// An interpreter.
pub trait Interpreter<H> {
	/// Interpreter state.
	type State;
	/// Interpreter trap.
	type Trap;

	/// Deconstruct the interpreter.
	fn deconstruct(self) -> (Self::State, Vec<u8>);
	/// Get a reference to the internal state.
	fn state(&self) -> &Self::State;
	/// Get a mutable reference to the internal state.
	fn state_mut(&mut self) -> &mut Self::State;
	/// Run the interpreter.
	fn run(&mut self, handle: &mut H) -> Capture<ExitResult, Self::Trap>;
}

/// Trap feedback for an interpreter.
pub trait FeedbackInterpreter<H, Feedback>: Interpreter<H> {
	/// Feedback to the interpreter.
	fn feedback(&mut self, feedback: Feedback, handler: &mut H) -> Result<(), ExitError>;
}

/// An interpreter that allows single stepping.
pub trait StepInterpreter<H>: Interpreter<H> {
	/// Step the interpreter.
	fn step(&mut self, handle: &mut H) -> Result<(), Capture<ExitResult, Self::Trap>>;
}
