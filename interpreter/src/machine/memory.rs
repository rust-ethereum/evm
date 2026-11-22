use alloc::{sync::Arc, vec, vec::Vec};
use core::{
	cmp::{max, min},
	fmt, mem,
	ops::{BitAnd, Not, Range},
};

#[allow(unused_imports)]
use crate::uint::{U256, U256Ext};
use crate::{
	error::{ExitException, ExitFatal},
	machine::CallInput,
};

/// A sequential memory. It uses Rust's `Vec` for internal
/// representation.
#[derive(Clone)]
pub struct Memory {
	/// Memory bytes. Empty while a callee shares them.
	data: Vec<u8>,
	/// Bytes shared with a callee, and kept out of `data` meanwhile.
	lent: Option<Arc<Vec<u8>>>,
	effective_len: U256,
	limit: usize,
}

impl fmt::Debug for Memory {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Memory")
			.field("data", &self.bytes())
			.field("effective_len", &self.effective_len)
			.field("limit", &self.limit)
			.finish()
	}
}

impl Memory {
	/// Create a new memory with the given limit.
	#[must_use]
	pub fn new(limit: usize) -> Self {
		Self {
			data: Vec::new(),
			lent: None,
			effective_len: U256::ZERO,
			limit,
		}
	}

	/// Memory limit.
	#[must_use]
	pub const fn limit(&self) -> usize {
		self.limit
	}

	/// Get the length of the current memory range.
	#[must_use]
	pub fn len(&self) -> usize {
		self.bytes().len()
	}

	/// Get the effective length.
	#[must_use]
	pub const fn effective_len(&self) -> U256 {
		self.effective_len
	}

	/// Return true if current effective memory range is zero.
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Return the full memory.
	///
	/// While a callee shares the bytes, they are held by the shares and this
	/// returns an empty vector. Only invoker code can reach a suspended caller.
	#[must_use]
	pub const fn data(&self) -> &Vec<u8> {
		&self.data
	}

	/// Memory bytes, the shared ones while a callee reads them.
	fn bytes(&self) -> &[u8] {
		match &self.lent {
			Some(lent) => &lent[..],
			None => &self.data,
		}
	}

	/// Let a callee read the range `offset..offset + len` without copying it.
	///
	/// The result shares the memory's buffer, so keeping it after the callee
	/// ends makes the next write to the memory copy the whole memory. The
	/// bytes up to `offset + len` are grown with zeros when the memory is
	/// shorter, so that the shared range equals what [`Memory::get`] returns
	/// for the same range.
	///
	/// The growth stays within what the caller paid only because the gasometer
	/// charges the memory expansion to `offset + len` before the opcode body
	/// runs. Without memory metering it can allocate more than a copy of `len`
	/// bytes would.
	pub(crate) fn share(&mut self, offset: usize, len: usize) -> CallInput {
		if len == 0 {
			return CallInput::default();
		}
		let end = match offset.checked_add(len) {
			Some(end) => end,
			None => return CallInput::from(self.get(offset, len)),
		};
		if let Some(lent) = &self.lent
			&& lent.len() >= end
		{
			return CallInput::from_shared(Arc::clone(lent), offset..end);
		}

		self.reclaim();
		if self.data.len() < end {
			self.data.resize(end, 0);
		}
		let lent = Arc::new(mem::take(&mut self.data));
		self.lent = Some(Arc::clone(&lent));

		CallInput::from_shared(lent, offset..end)
	}

	/// Take the shared bytes back into the memory.
	///
	/// Copies nothing when no share is left.
	pub(crate) fn reclaim(&mut self) {
		if let Some(lent) = self.lent.take() {
			self.data = Arc::try_unwrap(lent).unwrap_or_else(|lent| (*lent).clone());
		}
	}

	pub(crate) fn swap_and_clear(&mut self, other: &mut Vec<u8>) {
		self.reclaim();
		mem::swap(&mut self.data, other);
		self.data = Vec::new();
		self.effective_len = U256::ZERO;
	}

	/// Resize the memory, making it cover the memory region of `offset..(offset + len)`,
	/// with 32 bytes as the step. If the length is zero, this function
	/// does nothing.
	pub fn resize_offset(&mut self, offset: U256, len: U256) -> Result<(), ExitException> {
		if len == U256::ZERO {
			return Ok(());
		}

		offset
			.checked_add(len)
			.map_or(Err(ExitException::InvalidRange), |end| self.resize_end(end))
	}

	/// Resize the memory, making it cover to `end`, with 32 bytes as the step.
	pub fn resize_end(&mut self, end: U256) -> Result<(), ExitException> {
		if end > self.effective_len {
			let new_end = next_multiple_of_32(end).ok_or(ExitException::InvalidRange)?;
			self.effective_len = new_end;
		}

		Ok(())
	}

	/// Resize to range. Used for return value.
	pub fn resize_to_range(&mut self, return_range: Range<U256>) {
		let ret = if return_range.start > U256::USIZE_MAX {
			// TODO: Potential wrapping and hugely inefficient resizing.
			vec![0; (return_range.end - return_range.start).low_usize()]
		} else if return_range.end > U256::USIZE_MAX {
			// `return_range.start` is less than `usize::MAX`.
			let mut ret = self.get(
				return_range.start.low_usize(),
				usize::MAX - return_range.start.low_usize(),
			);
			// TODO: Potential wrapping and hugely inefficient resizing.
			while ret.len() < (return_range.end - return_range.start).low_usize() {
				ret.push(0);
			}
			ret
		} else {
			self.get(
				// `return_range.start` is less than `usize::MAX`.
				return_range.start.low_usize(),
				// Both `return_range.start` and `return_range.end` is less than `usize::MAX`.
				// Therefore their substraction is less than `usize::MAX`.
				(return_range.end - return_range.start).low_usize(),
			)
		};
		// `ret` was read from the shared bytes, so the old buffer can be
		// dropped without taking it back.
		self.lent = None;
		self.data = ret;
		self.effective_len = return_range.end - return_range.start;
	}

	/// Get memory region at given offset.
	///
	/// ## Panics
	///
	/// Value of `size` is considered trusted. If they're too large,
	/// the program can run out of memory, or it can overflow.
	#[must_use]
	pub fn get(&self, offset: usize, size: usize) -> Vec<u8> {
		let bytes = self.bytes();
		let mut ret = vec![0; size];

		#[allow(clippy::needless_range_loop)]
		for index in 0..size {
			let position = if let Some(position) = offset.checked_add(index) {
				position
			} else {
				break;
			};

			if position >= bytes.len() {
				break;
			}

			ret[index] = bytes[position];
		}

		ret
	}

	/// Set memory region at given offset. The offset and value is considered
	/// untrusted.
	pub fn set(
		&mut self,
		offset: usize,
		value: &[u8],
		target_size: Option<usize>,
	) -> Result<(), ExitFatal> {
		self.reclaim();
		let target_size = target_size.unwrap_or(value.len());
		if target_size == 0 {
			return Ok(());
		}

		if offset
			.checked_add(target_size)
			.map(|pos| pos > self.limit)
			.unwrap_or(true)
		{
			return Err(ExitFatal::NotSupported);
		}

		if self.data.len() < offset + target_size {
			self.data.resize(offset + target_size, 0);
		}

		if target_size > value.len() {
			self.data[offset..((value.len()) + offset)].clone_from_slice(value);
			for index in (value.len())..target_size {
				self.data[offset + index] = 0;
			}
		} else {
			self.data[offset..(target_size + offset)].clone_from_slice(&value[..target_size]);
		}

		Ok(())
	}

	/// Copy `data` into the memory, of given `len`.
	pub fn copy_large(
		&mut self,
		memory_offset: U256,
		data_offset: U256,
		len: U256,
		data: &[u8],
	) -> Result<(), ExitFatal> {
		self.reclaim();
		// Needed to pass ethereum test defined in
		// https://github.com/ethereum/tests/commit/17f7e7a6c64bb878c1b6af9dc8371b46c133e46d
		// (regardless of other inputs, a zero-length copy is defined to be a no-op).
		// TODO: refactor `set` and `copy_large` (see
		// https://github.com/rust-blockchain/evm/pull/40#discussion_r677180794)
		if len.is_zero() {
			return Ok(());
		}

		let memory_offset = if memory_offset > U256::USIZE_MAX {
			return Err(ExitFatal::NotSupported);
		} else {
			// `memory_offset` is less than `usize::MAX`.
			memory_offset.low_usize()
		};

		let ulen = if len > U256::USIZE_MAX {
			return Err(ExitFatal::NotSupported);
		} else {
			// `len` is less than `usize::MAX`.
			len.low_usize()
		};

		let data: &[u8] = data_offset.checked_add(len).map_or(&[], |end| {
			if end > U256::USIZE_MAX {
				&[]
			} else {
				// `data_offset + len == end`, and `end` is less than `usize::MAX`.
				// `data_offset` therefore must be less than `usize::MAX`.
				let data_offset = data_offset.low_usize();
				// `end` is less than `usize::MAX`.
				let end = end.low_usize();

				if data_offset > data.len() {
					&[]
				} else {
					&data[data_offset..min(end, data.len())]
				}
			}
		});

		self.set(memory_offset, data, Some(ulen))
	}

	/// Copies part of the memory inside another part of itself.
	pub fn copy(&mut self, dst: usize, src: usize, len: usize) {
		self.reclaim();
		let resize_offset = max(dst, src);
		if self.data.len() < resize_offset + len {
			self.data.resize(resize_offset + len, 0);
		}
		self.data.copy_within(src..src + len, dst);
	}
}

/// Rounds up `x` to the closest multiple of 32. If `x % 32 == 0` then `x` is returned.
#[inline]
fn next_multiple_of_32(x: U256) -> Option<U256> {
	let r = x.low_u32().bitand(31).not().wrapping_add(1).bitand(31);
	x.checked_add(U256::from_u32(r))
}

#[cfg(test)]
mod tests {
	use alloc::{format, vec};

	use super::{CallInput, Memory, U256, U256Ext, next_multiple_of_32};

	#[test]
	fn test_next_multiple_of_32() {
		// next_multiple_of_32 returns x when it is a multiple of 32
		for i in 0..32 {
			let x = U256::from_usize(i * 32);
			assert_eq!(Some(x), next_multiple_of_32(x));
		}

		// next_multiple_of_32 rounds up to the nearest multiple of 32 when `x % 32 != 0`
		for x in 0..1024 {
			if x % 32 == 0 {
				continue;
			}
			let next_multiple = x + 32 - (x % 32);
			assert_eq!(
				Some(U256::from_usize(next_multiple)),
				next_multiple_of_32(U256::from_usize(x))
			);
		}

		// next_multiple_of_32 returns None when the next multiple of 32 is too big
		let last_multiple_of_32 = U256::MAX & !U256::from_usize(31);
		for i in 0..63 {
			let x = U256::MAX - U256::from_usize(i);
			if x > last_multiple_of_32 {
				assert_eq!(None, next_multiple_of_32(x));
			} else {
				assert_eq!(Some(last_multiple_of_32), next_multiple_of_32(x));
			}
		}
	}

	#[test]
	fn test_memory_copy_works() {
		// Create a new instance of memory
		let mut memory = Memory::new(100usize);

		// Set the [0,0,0,1,2,3,4] array as memory data.
		//
		// We insert the [1,2,3,4] array on index 3,
		// that's why we have the zero padding at the beginning.
		memory.set(3usize, &[1u8, 2u8, 3u8, 4u8], None).unwrap();
		assert_eq!(memory.data(), &[0u8, 0u8, 0u8, 1u8, 2u8, 3u8, 4u8].to_vec());

		// Copy 1 byte into index 0.
		// As the length is 1, we only copy the byte present on index 3.
		memory.copy(0usize, 3usize, 1usize);

		// Now the new memory data results in [1,0,0,1,2,3,4]
		assert_eq!(memory.data(), &[1u8, 0u8, 0u8, 1u8, 2u8, 3u8, 4u8].to_vec());
	}

	#[test]
	fn test_memory_copy_resize() {
		// Create a new instance of memory
		let mut memory = Memory::new(100usize);

		// Set the [0,0,0,1,2,3,4] array as memory data.
		//
		// We insert the [1,2,3,4] array on index 3,
		// that's why we have the zero padding at the beginning.
		memory.set(3usize, &[1u8, 2u8, 3u8, 4u8], None).unwrap();
		assert_eq!(memory.data(), &[0u8, 0u8, 0u8, 1u8, 2u8, 3u8, 4u8].to_vec());

		// Copy 2 bytes into index 3.
		// As the length is 2, we copy the bytes present on indexes 6 and 7,
		// which are [4,0].
		memory.copy(3usize, 6usize, 2usize);

		// Now the new memory data results in [0, 0, 0, 4, 0, 3, 4, 0].
		// An extra element is added due to resizing.
		assert_eq!(
			memory.data(),
			&[0u8, 0u8, 0u8, 4u8, 0u8, 3u8, 4u8, 0u8].to_vec()
		);
	}

	/// A memory whose first bytes are `bytes`.
	fn memory_with(bytes: &[u8]) -> Memory {
		let mut memory = Memory::new(1024);
		memory.set(0, bytes, None).unwrap();
		memory
	}

	#[test]
	fn test_share_written_range() {
		let mut memory = memory_with(&[1, 2, 3, 4]);

		let input = memory.share(1, 2);

		assert_eq!(input.len(), 2);
		assert_eq!(&input[..], &[2, 3]);
		assert_eq!(input.to_vec(), vec![2, 3]);
	}

	#[test]
	fn test_share_past_end() {
		// A range that ends past the physical end is zero padded, and equals
		// what `get` returns for the same range.
		let mut memory = memory_with(&[1, 2, 3, 4]);
		let expected = memory.get(2, 8);
		let input = memory.share(2, 8);
		assert_eq!(&input[..], &expected[..]);
		assert_eq!(&input[..], &[3, 4, 0, 0, 0, 0, 0, 0]);
		assert_eq!(memory.len(), 10);

		// A range wholly past the physical end is all zeros.
		let mut memory = memory_with(&[1, 2, 3, 4]);
		let expected = memory.get(8, 4);
		let input = memory.share(8, 4);
		assert_eq!(&input[..], &expected[..]);
		assert_eq!(&input[..], &[0, 0, 0, 0]);
	}

	#[test]
	fn test_share_empty() {
		let mut memory = memory_with(&[1, 2, 3, 4]);
		let len = memory.len();
		let ptr = memory.data().as_ptr();

		let input = memory.share(100, 0);

		assert!(input.is_empty());
		assert_eq!(input.len(), 0);
		assert!(input.to_vec().is_empty());
		assert_eq!(memory.len(), len);
		assert_eq!(memory.data().as_ptr(), ptr);
		assert_eq!(memory.data(), &vec![1, 2, 3, 4]);
	}

	#[test]
	fn test_share_overflow() {
		let mut memory = memory_with(&[1, 2, 3, 4]);
		let expected = memory.get(usize::MAX, 4);
		let len = memory.len();
		let ptr = memory.data().as_ptr();

		let input = memory.share(usize::MAX, 4);

		assert_eq!(&input[..], &expected[..]);
		assert_eq!(&input[..], &[0, 0, 0, 0]);
		assert_eq!(memory.len(), len);
		assert_eq!(memory.data().as_ptr(), ptr);
	}

	#[test]
	fn test_share_twice() {
		let mut memory = memory_with(&[1, 2, 3, 4]);

		let first = memory.share(0, 4);
		let second = memory.share(0, 4);

		assert_eq!(&first[..], &[1, 2, 3, 4]);
		assert_eq!(&second[..], &[1, 2, 3, 4]);
		assert!(memory.data().is_empty());

		// A wider range takes the bytes back and lends them again.
		let expected = memory.get(0, 8);
		let third = memory.share(0, 8);
		assert_eq!(&first[..], &[1, 2, 3, 4]);
		assert_eq!(&second[..], &[1, 2, 3, 4]);
		assert_eq!(&third[..], &expected[..]);
		assert_eq!(memory.len(), 8);
	}

	#[test]
	fn test_share_reads_lent() {
		let mut memory = memory_with(&[1, 2, 3, 4]);

		let input = memory.share(1, 3);

		assert_eq!(input.len(), 3);
		assert_eq!(memory.len(), 4);
		assert!(!memory.is_empty());
		assert_eq!(memory.get(0, 4), vec![1, 2, 3, 4]);
		assert_eq!(memory.get(1, 3), vec![2, 3, 4]);
		assert_eq!(memory.effective_len(), U256::ZERO);
		assert!(memory.data().is_empty());
	}

	#[test]
	fn test_reclaim_copies_nothing() {
		let mut memory = memory_with(&[1, 2, 3, 4]);
		let ptr = memory.data().as_ptr();

		{
			let input = memory.share(0, 4);
			assert_eq!(&input[..], &[1, 2, 3, 4]);
		}
		memory.reclaim();

		assert_eq!(memory.data().as_ptr(), ptr);
		assert_eq!(memory.data(), &vec![1, 2, 3, 4]);

		memory.set(0, &[5, 6, 7, 8], None).unwrap();

		assert_eq!(memory.data().as_ptr(), ptr);
		assert_eq!(memory.data(), &vec![5, 6, 7, 8]);
	}

	#[test]
	fn test_write_while_shared() {
		let mut memory = memory_with(&[1, 2, 3, 4]);

		let input = memory.share(0, 4);
		memory.set(0, &[9, 9, 9, 9], None).unwrap();

		assert_eq!(&input[..], &[1, 2, 3, 4]);
		assert_eq!(memory.data(), &vec![9, 9, 9, 9]);
		assert_eq!(memory.get(0, 4), vec![9, 9, 9, 9]);
	}

	#[test]
	fn test_copy_while_shared() {
		let mut memory = memory_with(&[1, 2, 3, 4]);

		let input = memory.share(0, 4);
		memory.copy(0, 2, 2);

		assert_eq!(memory.data(), &vec![3, 4, 3, 4]);
		assert_eq!(memory.get(0, 4), vec![3, 4, 3, 4]);
		assert_eq!(&input[..], &[1, 2, 3, 4]);
	}

	#[test]
	fn test_copy_large_while_shared() {
		let mut memory = memory_with(&[1, 2, 3, 4]);

		let input = memory.share(0, 4);
		memory
			.copy_large(
				U256::from_usize(4),
				U256::ZERO,
				U256::from_usize(2),
				&[9, 9],
			)
			.unwrap();

		assert_eq!(memory.data(), &vec![1, 2, 3, 4, 9, 9]);
		assert_eq!(memory.get(4, 2), vec![9, 9]);
		assert_eq!(&input[..], &[1, 2, 3, 4]);
	}

	#[test]
	fn test_resize_to_range_while_shared() {
		let mut memory = memory_with(&[1, 2, 3, 4]);

		let input = memory.share(0, 4);
		memory.resize_to_range(U256::ONE..U256::from_usize(3));

		assert_eq!(memory.data(), &vec![2, 3]);
		assert_eq!(memory.len(), 2);
		assert_eq!(memory.effective_len(), U256::from_usize(2));
		assert_eq!(&input[..], &[1, 2, 3, 4]);
	}

	#[test]
	fn test_swap_and_clear_while_shared() {
		let mut memory = memory_with(&[1, 2, 3, 4]);

		let input = memory.share(0, 4);
		let mut other = vec![7, 7];
		memory.swap_and_clear(&mut other);

		assert_eq!(other, vec![1, 2, 3, 4]);
		assert!(memory.data().is_empty());
		assert_eq!(memory.len(), 0);
		assert_eq!(memory.effective_len(), U256::ZERO);
		assert_eq!(&input[..], &[1, 2, 3, 4]);
	}

	#[test]
	fn test_clone_while_shared() {
		let mut memory = memory_with(&[1, 2, 3, 4]);

		let input = memory.share(0, 4);
		let mut clone = memory.clone();

		assert_eq!(clone.len(), 4);
		assert_eq!(clone.get(0, 4), vec![1, 2, 3, 4]);

		clone.set(0, &[7, 7, 7, 7], None).unwrap();

		assert_eq!(clone.data(), &vec![7, 7, 7, 7]);
		assert_eq!(&input[..], &[1, 2, 3, 4]);
		assert_eq!(memory.get(0, 4), vec![1, 2, 3, 4]);
	}

	#[test]
	fn test_debug_format() {
		let mut memory = Memory::new(100);
		assert_eq!(
			format!("{memory:?}"),
			format!(
				"Memory {{ data: [], effective_len: {:?}, limit: 100 }}",
				U256::ZERO
			)
		);

		memory.set(0, &[1, 2], None).unwrap();
		let expected = format!(
			"Memory {{ data: [1, 2], effective_len: {:?}, limit: 100 }}",
			U256::ZERO
		);
		assert_eq!(format!("{memory:?}"), expected);

		let input = memory.share(0, 2);
		assert_eq!(format!("{memory:?}"), expected);
		assert_eq!(format!("{input:?}"), "[1, 2]");
		assert_eq!(format!("{:?}", CallInput::default()), "[]");
	}
}
