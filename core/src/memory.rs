use crate::ExitError;

/// A sequencial memory. It uses Rust's `Vec` for internal
/// representation.
#[derive(Clone, Debug)]
pub struct Memory {
	data: Vec<u8>,
	limit: usize,
}

impl Memory {
	pub fn new(limit: usize) -> Self {
		Self {
			data: Vec::new(),
			limit,
		}
	}

	/// Get the length of the current effective memory range.
	pub fn len(&self) -> usize {
		self.data.len()
	}

	/// Return true if current effective memory range is zero.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Resize the current memory range to given length, aligned to next 32.
	pub fn resize(&mut self, mut size: usize) -> Result<(), ExitError> {
		if self.data.len() >= size {
			return Ok(())
		}

		while size % 32 != 0 {
			size += 1;
		}

		if size > self.limit {
			return Err(ExitError::NotSupported)
		}

		self.data.resize(size, 0);
		Ok(())
	}

	/// Get memory region at given offset.
	///
	/// ## Panics
	///
	/// Value of `size` is considered trusted. If they're too large,
	/// the program can run out of memory, or it can overflow.
	pub fn get(&self, offset: usize, size: usize) -> Vec<u8> {
		let mut ret = Vec::new();
		ret.resize(size, 0);

		for index in 0..size {
			let position = offset + index;
			if position >= self.data.len() {
				break
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
		target_size: Option<usize>
	) -> Result<(), ExitError> {
		let target_size = target_size.unwrap_or(value.len());

		if offset.checked_add(target_size)
			.map(|pos| pos > self.limit).unwrap_or(true)
		{
			return Err(ExitError::NotSupported)
		}

		self.resize(offset + value.len())?;

		for index in 0..value.len() {
			if self.data.len() > offset + index {
				self.data[offset + index] = value[index];
			}
		}

		Ok(())
	}
}
