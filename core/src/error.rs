use crate::ExternalOpcode;

pub type Trap = ExternalOpcode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capture<E, T> {
	Exit(E),
	Trap(T),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitReason {
	Succeed(ExitSucceed),
	Error(ExitError),
}

impl ExitReason {
	pub fn is_succeed(&self) -> bool {
		match self {
			ExitReason::Succeed(_) => true,
			ExitReason::Error(_) => false,
		}
	}

	pub fn is_error(&self) -> bool {
		match self {
			ExitReason::Succeed(_) => false,
			ExitReason::Error(_) => true,
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitSucceed {
	Stopped,
	Returned,
	Suicided,

	Other(&'static str),
}

impl From<ExitSucceed> for ExitReason {
	fn from(exit: ExitSucceed) -> ExitReason {
		ExitReason::Succeed(exit)
	}
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

impl From<ExitError> for ExitReason {
	fn from(exit: ExitError) -> ExitReason {
		ExitReason::Error(exit)
	}
}
