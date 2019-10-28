use crate::ExternalOpcode;

pub type Trap = ExternalOpcode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capture<E, T> {
	Exit(E),
	Trap(T),
}

pub type ExitReason = Result<ExitSucceed, ExitError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitSucceed {
	Stopped,
	Returned,
	Suicided,

	Other(&'static str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitError {
	Reverted,
	StackUnderflow,
	StackOverflow,
	InvalidJump,
	InvalidReturnRange,
	PCUnderflow,
	DesignatedInvalid,

	OutOfGas,
	NotSupported,
	Other(&'static str),
}
