mod etable;
mod valids;

use alloc::{boxed::Box, vec::Vec};

pub use self::etable::EtableInterpreter;
pub use self::valids::Valids;

use crate::{Capture, ExitError, ExitResult, Machine};

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
	/// Get a mutable reference to the machine, if this interpreter runs one.
	///
	/// An invoker uses it to share call input from the machine. An interpreter
	/// that returns `Some` lets a call stack whose invoker shares call input
	/// switch sharing on. The default returns `None`, which keeps the
	/// behaviour of an interpreter that does not expose a machine.
	///
	/// [`Interpreter::deconstruct`] must drop the machine, so that no share of
	/// a caller's memory outlives its callee. An interpreter that returns
	/// `Some` but does not run its machine through `EtableInterpreter::step`
	/// should know that the input range of a pending call is cleared only by
	/// the trap constructors, by both feedbacks and by that step.
	fn machine_mut(&mut self) -> Option<&mut Machine<Self::State>> {
		None
	}
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
