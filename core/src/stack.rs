use crate::ExitError;
use alloc::vec::Vec;
use primitive_types::{H256, U256};

/// EVM stack.
#[derive(Clone, Debug)]
pub struct Stack {
	data: Vec<U256>,
	limit: usize,
}

impl Stack {
	/// Create a new stack with given limit.
	pub fn new(limit: usize) -> Self {
		Self {
			data: Vec::new(),
			limit,
		}
	}

	#[inline]
	/// Stack limit.
	pub fn limit(&self) -> usize {
		self.limit
	}

	#[inline]
	/// Stack length.
	pub fn len(&self) -> usize {
		self.data.len()
	}

	#[inline]
	/// Whether the stack is empty.
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}

	#[inline]
	/// Stack data.
	pub fn data(&self) -> &Vec<U256> {
		&self.data
	}

	#[inline]
	/// Pop a value from the stack. If the stack is already empty, returns the
	/// `StackUnderflow` error.
	pub fn pop(&mut self) -> Result<U256, ExitError> {
		self.data.pop().ok_or(ExitError::StackUnderflow)
	}

	#[inline]
	pub fn pop_h256(&mut self) -> Result<H256, ExitError> {
		self.pop().map(|it| {
			let mut res = H256([0; 32]);
			it.to_big_endian(&mut res.0);
			res
		})
	}

	#[inline]
	/// Push a new value into the stack. If it will exceed the stack limit,
	/// returns `StackOverflow` error and leaves the stack unchanged.
	pub fn push(&mut self, value: U256) -> Result<(), ExitError> {
		if self.data.len() + 1 > self.limit {
			return Err(ExitError::StackOverflow);
		}
		self.data.push(value);
		Ok(())
	}

	#[inline]
	/// Peek a value at given index for the stack, where the top of
	/// the stack is at index `0`. If the index is too large,
	/// `StackError::Underflow` is returned.
	pub fn peek(&self, no_from_top: usize) -> Result<U256, ExitError> {
		if self.data.len() > no_from_top {
			Ok(self.data[self.data.len() - no_from_top - 1])
		} else {
			Err(ExitError::StackUnderflow)
		}
	}

	#[inline]
	/// Peek a value at given index for the stack, where the top of
	/// the stack is at index `0`. If the index is too large,
	/// `StackError::Underflow` is returned.
	pub fn peek_h256(&self, no_from_top: usize) -> Result<H256, ExitError> {
		self.peek(no_from_top).map(|it| {
			let mut res = H256([0; 32]);
			it.to_big_endian(&mut res.0);
			res
		})
	}

	#[inline]
	/// Set a value at given index for the stack, where the top of the
	/// stack is at index `0`. If the index is too large,
	/// `StackError::Underflow` is returned.
	pub fn set(&mut self, no_from_top: usize, val: U256) -> Result<(), ExitError> {
		if self.data.len() > no_from_top {
			let len = self.data.len();
			self.data[len - no_from_top - 1] = val;
			Ok(())
		} else {
			Err(ExitError::StackUnderflow)
		}
	}
}
