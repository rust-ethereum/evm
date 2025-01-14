use crate::{ExitError, ExitFatal};
use alloc::vec::Vec;
use core::cmp::{max, min};
use core::ops::{BitAnd, Not};
use primitive_types::U256;

/// A sequencial memory. It uses Rust's `Vec` for internal
/// representation.
#[derive(Clone, Debug)]
pub struct Memory {
	data: Vec<u8>,
	effective_len: U256,
	limit: usize,
}

impl Memory {
	/// Create a new memory with the given limit.
	pub fn new(limit: usize) -> Self {
		Self {
			data: Vec::new(),
			effective_len: U256::zero(),
			limit,
		}
	}

	/// Memory limit.
	pub fn limit(&self) -> usize {
		self.limit
	}

	/// Get the length of the current memory range.
	pub fn len(&self) -> usize {
		self.data.len()
	}

	/// Get the effective length.
	pub fn effective_len(&self) -> U256 {
		self.effective_len
	}

	/// Return true if current effective memory range is zero.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Return the full memory.
	pub fn data(&self) -> &Vec<u8> {
		&self.data
	}

	/// Resize the memory, making it cover the memory region of `offset..(offset + len)`,
	/// with 32 bytes as the step. If the length is zero, this function does nothing.
	pub fn resize_offset(&mut self, offset: U256, len: U256) -> Result<(), ExitError> {
		if len == U256::zero() {
			return Ok(());
		}

		if let Some(end) = offset.checked_add(len) {
			self.resize_end(end)
		} else {
			Err(ExitError::InvalidRange)
		}
	}

	/// Resize the memory, making it cover to `end`, with 32 bytes as the step.
	pub fn resize_end(&mut self, end: U256) -> Result<(), ExitError> {
		if end > self.effective_len {
			let new_end = next_multiple_of_32(end).ok_or(ExitError::InvalidRange)?;
			self.effective_len = new_end;
		}

		Ok(())
	}

	/// Get memory region at given offset.
	///
	/// ## Panics
	///
	/// Value of `size` is considered trusted. If they're too large,
	/// the program can run out of memory, or it can overflow.
	#[allow(clippy::slow_vector_initialization)]
	// Clippy complains about not using `vec!`. However, we need to support
	// `no_std` and we can't use that.
	pub fn get(&self, offset: usize, size: usize) -> Vec<u8> {
		let mut ret = Vec::new();
		ret.resize(size, 0);

		#[allow(clippy::needless_range_loop)]
		for index in 0..size {
			let position = offset + index;
			if position >= self.data.len() {
				break;
			}

			ret[index] = self.data[position];
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
		// Needed to pass ethereum test defined in
		// https://github.com/ethereum/tests/commit/17f7e7a6c64bb878c1b6af9dc8371b46c133e46d
		// (regardless of other inputs, a zero-length copy is defined to be a no-op).
		// TODO: refactor `set` and `copy_large` (see
		// https://github.com/rust-blockchain/evm/pull/40#discussion_r677180794)
		if len.is_zero() {
			return Ok(());
		}

		let memory_offset = if memory_offset > U256::from(usize::MAX) {
			return Err(ExitFatal::NotSupported);
		} else {
			memory_offset.as_usize()
		};

		let ulen = if len > U256::from(usize::MAX) {
			return Err(ExitFatal::NotSupported);
		} else {
			len.as_usize()
		};

		let data = if let Some(end) = data_offset.checked_add(len) {
			if end > U256::from(usize::MAX) {
				&[]
			} else {
				let data_offset = data_offset.as_usize();
				let end = end.as_usize();

				if data_offset > data.len() {
					&[]
				} else {
					&data[data_offset..min(end, data.len())]
				}
			}
		} else {
			&[]
		};

		self.set(memory_offset, data, Some(ulen))
	}

	/// Copies part of the memory inside another part of itself.
	pub fn copy(&mut self, dst: usize, src: usize, len: usize) {
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
	x.checked_add(r.into())
}

#[cfg(test)]
mod tests {
	use super::{next_multiple_of_32, Memory, U256};

	#[test]
	fn test_next_multiple_of_32() {
		// next_multiple_of_32 returns x when it is a multiple of 32
		for i in 0..32 {
			let x = U256::from(i * 32);
			assert_eq!(Some(x), next_multiple_of_32(x));
		}

		// next_multiple_of_32 rounds up to the nearest multiple of 32 when `x % 32 != 0`
		for x in 0..1024 {
			if x % 32 == 0 {
				continue;
			}
			let next_multiple = x + 32 - (x % 32);
			assert_eq!(
				Some(U256::from(next_multiple)),
				next_multiple_of_32(x.into())
			);
		}

		// next_multiple_of_32 returns None when the next multiple of 32 is too big
		let last_multiple_of_32 = U256::MAX & !U256::from(31);
		for i in 0..63 {
			let x = U256::MAX - U256::from(i);
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
		// An extra element is added due to rezising.
		assert_eq!(
			memory.data(),
			&[0u8, 0u8, 0u8, 4u8, 0u8, 3u8, 4u8, 0u8].to_vec()
		);
	}
}
