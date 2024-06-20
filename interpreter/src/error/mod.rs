mod exit;
mod trap;

pub use self::{exit::*, trap::*};

/// Capture represents the result of execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capture<E, T> {
	/// The machine has exited. It cannot be executed again.
	Exit(E),
	/// The machine has trapped. It is waiting for external information, and can
	/// be executed again.
	Trap(T),
}

impl<E, T> Capture<E, T> {
	pub fn exit(self) -> Option<E> {
		match self {
			Self::Exit(e) => Some(e),
			Self::Trap(_) => None,
		}
	}

	pub fn trap(self) -> Option<T> {
		match self {
			Self::Exit(_) => None,
			Self::Trap(t) => Some(t),
		}
	}
}
