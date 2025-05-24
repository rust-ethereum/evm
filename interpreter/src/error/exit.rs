use alloc::borrow::Cow;
use core::fmt;

use crate::opcode::Opcode;

/// Exit result.
pub type ExitResult = Result<ExitSucceed, ExitError>;

/// Exit reason.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "scale",
	derive(scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitError {
	/// Machine returns a normal EVM error.
	Exception(ExitException),
	/// Machine encountered an explicit revert.
	Reverted,
	/// Machine encountered an error that is not supposed to be normal EVM
	/// errors, such as requiring too much memory to execute.
	Fatal(ExitFatal),
}

impl From<ExitError> for ExitResult {
	fn from(s: ExitError) -> Self {
		Err(s)
	}
}

#[cfg(feature = "std")]
impl std::error::Error for ExitError {}

impl fmt::Display for ExitError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Exception(_) => f.write_str("EVM exit exception"),
			Self::Reverted => f.write_str("EVM internal revert"),
			Self::Fatal(_) => f.write_str("EVM fatal error"),
		}
	}
}

/// Exit succeed reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "scale",
	derive(scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitSucceed {
	/// Machine encountered an explicit stop.
	Stopped,
	/// Machine encountered an explicit return.
	Returned,
	/// Machine encountered an explicit suicide.
	Suicided,
}

impl From<ExitSucceed> for ExitResult {
	fn from(s: ExitSucceed) -> Self {
		Ok(s)
	}
}

/// Exit error reason.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "scale",
	derive(scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitException {
	/// Trying to pop from an empty stack.
	#[cfg_attr(feature = "scale", codec(index = 0))]
	StackUnderflow,
	/// Trying to push into a stack over stack limit.
	#[cfg_attr(feature = "scale", codec(index = 1))]
	StackOverflow,
	/// Jump destination is invalid.
	#[cfg_attr(feature = "scale", codec(index = 2))]
	InvalidJump,
	/// An opcode accesses memory region, but the region is invalid.
	#[cfg_attr(feature = "scale", codec(index = 3))]
	InvalidRange,
	/// Encountered the designated invalid opcode.
	#[cfg_attr(feature = "scale", codec(index = 4))]
	DesignatedInvalid,
	/// Call stack is too deep (runtime).
	#[cfg_attr(feature = "scale", codec(index = 5))]
	CallTooDeep,
	/// Create opcode encountered collision (runtime).
	#[cfg_attr(feature = "scale", codec(index = 6))]
	CreateCollision,
	/// Create init code exceeds limit (runtime).
	#[cfg_attr(feature = "scale", codec(index = 7))]
	CreateContractLimit,

	/// Invalid opcode during execution or starting byte is 0xef ([EIP-3541](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-3541.md)).
	#[cfg_attr(feature = "scale", codec(index = 15))]
	InvalidOpcode(Opcode),

	/// An opcode accesses external information, but the request is off offset
	/// limit (runtime).
	#[cfg_attr(feature = "scale", codec(index = 8))]
	OutOfOffset,
	/// Execution runs out of gas (runtime).
	#[cfg_attr(feature = "scale", codec(index = 9))]
	OutOfGas,
	/// Not enough fund to start the execution (runtime).
	#[cfg_attr(feature = "scale", codec(index = 10))]
	OutOfFund,

	/// PC underflowed (unused).
	#[allow(clippy::upper_case_acronyms)]
	#[cfg_attr(feature = "scale", codec(index = 11))]
	PCUnderflow,

	/// Attempt to create an empty account (runtime, unused).
	#[cfg_attr(feature = "scale", codec(index = 12))]
	CreateEmpty,

	/// Nonce reached maximum value of 2^64-1
	/// https://eips.ethereum.org/EIPS/eip-2681
	#[cfg_attr(feature = "scale", codec(index = 14))]
	MaxNonce,

	/// Other normal errors.
	#[cfg_attr(feature = "scale", codec(index = 13))]
	Other(Cow<'static, str>),
}

impl From<ExitException> for ExitResult {
	fn from(s: ExitException) -> Self {
		Err(ExitError::Exception(s))
	}
}

impl From<ExitException> for ExitError {
	fn from(s: ExitException) -> Self {
		Self::Exception(s)
	}
}

/// Exit fatal reason.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "scale",
	derive(scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitFatal {
	/// The operation is not supported.
	NotSupported,
	/// The trap (interrupt) is unhandled.
	UnhandledInterrupt,
	/// The environment explicitly set call errors as fatal error.
	ExceptionAsFatal(ExitException),
	/// Already exited.
	AlreadyExited,
	/// Unfinished execution.
	Unfinished,

	/// Other fatal errors.
	Other(Cow<'static, str>),
}

impl From<ExitFatal> for ExitResult {
	fn from(s: ExitFatal) -> Self {
		Err(ExitError::Fatal(s))
	}
}

impl From<ExitFatal> for ExitError {
	fn from(s: ExitFatal) -> Self {
		Self::Fatal(s)
	}
}
