//! Call input.

use alloc::{sync::Arc, vec::Vec};
use core::{
	fmt,
	ops::{Deref, Range},
};

/// Input of a call.
///
/// The input either owns its bytes or reads a range of a memory buffer that it
/// shares with the memory it came from. A `CallInput` is not `Clone`, because a
/// retained clone would make the next write to that memory copy the whole
/// memory. Call [`CallInput::to_vec`] to keep the bytes.
///
/// ```
/// let input = evm_interpreter::CallInput::from(vec![1u8, 2, 3]);
/// assert_eq!(&input[..], &[1u8, 2, 3][..]);
/// assert_eq!(input.to_vec(), vec![1u8, 2, 3]);
/// assert_eq!(format!("{input:?}"), "[1, 2, 3]");
/// ```
///
/// A `CallInput` cannot be cloned:
///
/// ```compile_fail
/// let input = evm_interpreter::CallInput::default();
/// let kept = input.clone();
/// ```
pub struct CallInput {
	/// Bytes that the input reads, `None` for an empty input.
	input: Option<Arc<Vec<u8>>>,
	/// Range of `input` that the input reads.
	range: Range<usize>,
}

impl CallInput {
	/// Create a call input that reads the range `range` of the buffer `input`.
	pub(crate) fn from_shared(input: Arc<Vec<u8>>, range: Range<usize>) -> Self {
		Self {
			input: Some(input),
			range,
		}
	}

	/// Length of the input in bytes.
	#[must_use]
	pub fn len(&self) -> usize {
		self.range.end - self.range.start
	}

	/// Whether the input is empty.
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Copy the input bytes into a new vector.
	#[must_use]
	pub fn to_vec(&self) -> Vec<u8> {
		self.deref().to_vec()
	}
}

impl Deref for CallInput {
	type Target = [u8];

	fn deref(&self) -> &[u8] {
		match &self.input {
			Some(input) => &input[self.range.clone()],
			None => &[],
		}
	}
}

impl AsRef<[u8]> for CallInput {
	fn as_ref(&self) -> &[u8] {
		self.deref()
	}
}

impl fmt::Debug for CallInput {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self.deref(), f)
	}
}

impl Default for CallInput {
	fn default() -> Self {
		Self {
			input: None,
			range: 0..0,
		}
	}
}

impl From<Vec<u8>> for CallInput {
	fn from(input: Vec<u8>) -> Self {
		if input.is_empty() {
			return Self::default();
		}
		let range = 0..input.len();
		Self {
			input: Some(Arc::new(input)),
			range,
		}
	}
}
