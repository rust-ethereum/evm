use alloc::boxed::Box;

mod exit;

pub use self::exit::{ExitError, ExitException, ExitFatal, ExitResult, ExitSucceed};

/// Capture represents the result of execution.
#[derive(Debug, Eq, PartialEq)]
pub enum Capture<E, T> {
	/// The machine has exited. It cannot be executed again.
	Exit(E),
	/// The machine has trapped. It is waiting for external information, and can
	/// be executed again.
	Trap(Box<T>),
}

impl<E, T> Capture<E, T> {
	/// Exit value if it is [Capture::Exit].
	pub fn exit(self) -> Option<E> {
		match self {
			Self::Exit(e) => Some(e),
			Self::Trap(_) => None,
		}
	}

	/// Trap value if it is [Capture::Trap].
	pub fn trap(self) -> Option<Box<T>> {
		match self {
			Self::Exit(_) => None,
			Self::Trap(t) => Some(t),
		}
	}
}
