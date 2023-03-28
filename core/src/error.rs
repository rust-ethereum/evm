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
	derive(scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)
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
	derive(scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)
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
	derive(scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)
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
	derive(scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitError {
	/// Trying to pop from an empty stack.
	#[cfg_attr(feature = "with-codec", codec(index = 0))]
	StackUnderflow,
	/// Trying to push into a stack over stack limit.
	#[cfg_attr(feature = "with-codec", codec(index = 1))]
	StackOverflow,
	/// Jump destination is invalid.
	#[cfg_attr(feature = "with-codec", codec(index = 2))]
	InvalidJump,
	/// An opcode accesses memory region, but the region is invalid.
	#[cfg_attr(feature = "with-codec", codec(index = 3))]
	InvalidRange,
	/// Encountered the designated invalid opcode.
	#[cfg_attr(feature = "with-codec", codec(index = 4))]
	DesignatedInvalid,
	/// Call stack is too deep (runtime).
	#[cfg_attr(feature = "with-codec", codec(index = 5))]
	CallTooDeep,
	/// Create opcode encountered collision (runtime).
	#[cfg_attr(feature = "with-codec", codec(index = 6))]
	CreateCollision,
	/// Create init code exceeds limit (runtime).
	#[cfg_attr(feature = "with-codec", codec(index = 7))]
	CreateContractLimit,
	/// Invalid opcode during execution or starting byte is 0xef. See [EIP-3541](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-3541.md).
	#[cfg_attr(feature = "with-codec", codec(index = 15))]
	InvalidCode(Opcode),

	/// An opcode accesses external information, but the request is off offset
	/// limit (runtime).
	#[cfg_attr(feature = "with-codec", codec(index = 8))]
	OutOfOffset,
	/// Execution runs out of gas (runtime).
	#[cfg_attr(feature = "with-codec", codec(index = 9))]
	OutOfGas,
	/// Not enough fund to start the execution (runtime).
	#[cfg_attr(feature = "with-codec", codec(index = 10))]
	OutOfFund,

	/// PC underflowed (unused).
	#[allow(clippy::upper_case_acronyms)]
	#[cfg_attr(feature = "with-codec", codec(index = 11))]
	PCUnderflow,

	/// Attempt to create an empty account (runtime, unused).
	#[cfg_attr(feature = "with-codec", codec(index = 12))]
	CreateEmpty,

	/// Other normal errors.
	#[cfg_attr(feature = "with-codec", codec(index = 13))]
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
	derive(scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)
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
