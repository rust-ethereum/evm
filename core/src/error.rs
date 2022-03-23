use crate::Opcode;
use alloc::borrow::Cow;

/// Trap which indicates that an `ExternalOpcode` has to be handled.
pub type Trap = Opcode;

/// Capture represents the result of execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capture<E, T> {
	/// The machine has exited. It cannot be executed again.
	Exit(E),
	/// The machine has trapped. It is waiting for external information, and can
	/// be executed again.
	Trap(T),
}

/// Exit reason.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "with-codec",
	derive(codec::Encode, codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitReason {
	/// Machine has succeeded.
	Succeed(ExitSucceed),
	/// Machine returns a normal EVM error.
	Error(ExitError),
	/// Machine encountered an explicit revert.
	Revert(ExitRevert),
	/// Machine encountered an error that is not supposed to be normal EVM
	/// errors, such as requiring too much memory to execute.
	Fatal(ExitFatal),
}

impl ExitReason {
	/// Whether the exit is succeeded.
	pub fn is_succeed(&self) -> bool {
		matches!(self, Self::Succeed(_))
	}

	/// Whether the exit is error.
	pub fn is_error(&self) -> bool {
		matches!(self, Self::Error(_))
	}

	/// Whether the exit is revert.
	pub fn is_revert(&self) -> bool {
		matches!(self, Self::Revert(_))
	}

	/// Whether the exit is fatal.
	pub fn is_fatal(&self) -> bool {
		matches!(self, Self::Fatal(_))
	}
}

/// Exit succeed reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "with-codec",
	derive(codec::Encode, codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitSucceed {
	/// Machine encountered an explicit stop.
	Stopped,
	/// Machine encountered an explicit return.
	Returned,
	/// Machine encountered an explicit suicide.
	Suicided,
}

impl From<ExitSucceed> for ExitReason {
	fn from(s: ExitSucceed) -> Self {
		Self::Succeed(s)
	}
}

/// Exit revert reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "with-codec",
	derive(codec::Encode, codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitRevert {
	/// Machine encountered an explicit revert.
	Reverted,
}

impl From<ExitRevert> for ExitReason {
	fn from(s: ExitRevert) -> Self {
		Self::Revert(s)
	}
}

/// Exit error reason.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "with-codec",
	derive(codec::Encode, codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitError {
	/// Trying to pop from an empty stack.
	StackUnderflow,
	/// Trying to push into a stack over stack limit.
	StackOverflow,
	/// Jump destination is invalid.
	InvalidJump,
	/// An opcode accesses memory region, but the region is invalid.
	InvalidRange,
	/// Encountered the designated invalid opcode.
	DesignatedInvalid,
	/// Call stack is too deep (runtime).
	CallTooDeep,
	/// Create opcode encountered collision (runtime).
	CreateCollision,
	/// Create init code exceeds limit (runtime).
	CreateContractLimit,
	/// Starting byte must not begin with 0xef. See [EIP-3541](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-3541.md).
	InvalidCode,

	/// An opcode accesses external information, but the request is off offset
	/// limit (runtime).
	OutOfOffset,
	/// Execution runs out of gas (runtime).
	OutOfGas,
	/// Not enough fund to start the execution (runtime).
	OutOfFund,

	/// PC underflowed (unused).
	#[allow(clippy::upper_case_acronyms)]
	PCUnderflow,

	/// Attempt to create an empty account (runtime, unused).
	CreateEmpty,

	/// Other normal errors.
	Other(Cow<'static, str>),
}

impl From<ExitError> for ExitReason {
	fn from(s: ExitError) -> Self {
		Self::Error(s)
	}
}

/// Exit fatal reason.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "with-codec",
	derive(codec::Encode, codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitFatal {
	/// The operation is not supported.
	NotSupported,
	/// The trap (interrupt) is unhandled.
	UnhandledInterrupt,
	/// The environment explicitly set call errors as fatal error.
	CallErrorAsFatal(ExitError),

	/// Other fatal errors.
	Other(Cow<'static, str>),
}

impl From<ExitFatal> for ExitReason {
	fn from(s: ExitFatal) -> Self {
		Self::Fatal(s)
	}
}
