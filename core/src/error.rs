use crate::ExternalOpcode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Trap {
	Exit(ExitReason),
	External(ExternalOpcode),
}

impl From<ExitReason> for Trap {
	fn from(reason: ExitReason) -> Trap {
	Trap::Exit(reason)
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitReason {
	Succeed(ExitSucceed),
	Error(ExitError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitSucceed {
	Stopped,
	Returned,

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
