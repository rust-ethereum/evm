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
	Revert(ExitRevert),
	Fatal(ExitFatal),
}

impl ExitReason {
	pub fn is_succeed(&self) -> bool {
		match self {
			Self::Succeed(_) => true,
			_ => false,
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitSucceed {
	Stopped,
	Returned,
	Suicided,
}

impl From<ExitSucceed> for ExitReason {
	fn from(s: ExitSucceed) -> Self {
		Self::Succeed(s)
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitRevert {
	Reverted,
}

impl From<ExitRevert> for ExitReason {
	fn from(s: ExitRevert) -> Self {
		Self::Revert(s)
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitError {
	StackUnderflow,
	StackOverflow,
	InvalidJump,
	InvalidRange,
	PCUnderflow,
	DesignatedInvalid,
	CallTooDeep,
	CreateCollision,
	CreateEmpty,
	CreateContractLimit,

	OutOfOffset,
	OutOfGas,
	OutOfFund,

	Other(&'static str),
}

impl From<ExitError> for ExitReason {
	fn from(s: ExitError) -> Self {
		Self::Error(s)
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitFatal {
	NotSupported,
	UnhandledInterrupt,
	CallErrorAsFatal(ExitError),

	Other(&'static str),
}

impl From<ExitFatal> for ExitReason {
	fn from(s: ExitFatal) -> Self {
		Self::Fatal(s)
	}
}
