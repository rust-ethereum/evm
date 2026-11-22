//! Input of a call, shared with the callee.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
use core::ops::{Deref, Range};

/// Input of a call, either shared with the caller's memory or owned.
///
/// A `CallInput` that [`Memory::share`](crate::Memory::share) returned reads
/// the bytes of the caller's memory in place instead of copying them. It must
/// not outlive the call it is the input of: while it is alive, the caller's
/// next write to its memory copies the whole buffer (copy-on-write), because
/// the buffer is then no longer uniquely owned.
///
/// The type is deliberately not `Clone`. A clone kept after the callee ends
/// would make that copy happen. Call [`CallInput::to_vec`] instead to keep the
/// bytes themselves.
///
/// ```compile_fail
/// fn needs_clone<T: Clone>() {}
/// needs_clone::<evm_core::CallInput>();
/// ```
pub struct CallInput {
	data: Option<Arc<Vec<u8>>>,
	range: Range<usize>,
}

impl CallInput {
	/// Share `range` of `data`.
	pub(crate) fn from_shared(data: Arc<Vec<u8>>, range: Range<usize>) -> Self {
		Self {
			data: Some(data),
			range,
		}
	}

	/// Get the length of the input in bytes.
	pub fn len(&self) -> usize {
		self.range.len()
	}

	/// Return true if the input is empty.
	pub fn is_empty(&self) -> bool {
		self.range.is_empty()
	}

	/// Copy the input into a new `Vec`.
	pub fn to_vec(&self) -> Vec<u8> {
		self[..].to_vec()
	}
}

impl Deref for CallInput {
	type Target = [u8];

	fn deref(&self) -> &[u8] {
		match &self.data {
			Some(data) => &data[self.range.clone()],
			None => &[],
		}
	}
}

impl AsRef<[u8]> for CallInput {
	fn as_ref(&self) -> &[u8] {
		self
	}
}

impl fmt::Debug for CallInput {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(&**self, f)
	}
}

impl Default for CallInput {
	fn default() -> Self {
		Self {
			data: None,
			range: 0..0,
		}
	}
}

impl From<Vec<u8>> for CallInput {
	fn from(data: Vec<u8>) -> Self {
		if data.is_empty() {
			return Self::default();
		}

		let range = 0..data.len();
		Self {
			data: Some(Arc::new(data)),
			range,
		}
	}
}
