use alloc::{vec, vec::Vec};
use core::{
	cmp::min,
	mem,
	ops::{BitAnd, Not, Range},
};

use primitive_types::U256;

use crate::error::{ExitException, ExitFatal};

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
	#[must_use]
	pub fn new(limit: usize) -> Self {
		Self {
			data: Vec::new(),
			effective_len: U256::zero(),
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
		self.data.len()
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
	#[must_use]
	pub const fn data(&self) -> &Vec<u8> {
		&self.data
	}

	pub(crate) fn swap_and_clear(&mut self, other: &mut Vec<u8>) {
		mem::swap(&mut self.data, other);
		self.data = Vec::new();
		self.effective_len = U256::zero();
	}

	/// Resize the memory, making it cover the memory region of `offset..(offset
	/// + len)`, with 32 bytes as the step. If the length is zero, this function
	/// does nothing.
	pub fn resize_offset(&mut self, offset: U256, len: U256) -> Result<(), ExitException> {
		if len == U256::zero() {
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
		let ret = if return_range.start > U256::from(usize::MAX) {
			vec![0; (return_range.end - return_range.start).as_usize()]
		} else if return_range.end > U256::from(usize::MAX) {
			let mut ret = self.get(
				return_range.start.as_usize(),
				usize::MAX - return_range.start.as_usize(),
			);
			while ret.len() < (return_range.end - return_range.start).as_usize() {
				ret.push(0);
			}
			ret
		} else {
			self.get(
				return_range.start.as_usize(),
				(return_range.end - return_range.start).as_usize(),
			)
		};
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
		let mut ret = vec![0; size];

		#[allow(clippy::needless_range_loop)]
		for index in 0..size {
			let position = if let Some(position) = offset.checked_add(index) {
				position
			} else {
				break;
			};

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

		let data: &[u8] = data_offset.checked_add(len).map_or(&[], |end| {
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
		});

		self.set(memory_offset, data, Some(ulen))
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
	use super::{next_multiple_of_32, U256};

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
}
