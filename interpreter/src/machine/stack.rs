use alloc::vec::Vec;

use primitive_types::H256;

use crate::error::{ExitError, ExitException};

/// EVM stack.
#[derive(Clone, Debug)]
pub struct Stack {
	data: Vec<H256>,
	limit: usize,
}

macro_rules! impl_perform_popn_pushn {
	(
		$name:ident,
		$pop_len:expr,
		$push_len:expr,
		($($peek_pop:expr),*),
		($($peek_push:expr),*),
		$pop_pushn_f:ident
	) => {
		/// Pop $pop_len values from the stack, and then push $push_len values
		/// into the stack.
		///
		/// If `f` returns error, then the stack will not be changed.
		#[allow(unused_parens)]
		pub fn $name<R, F>(&mut self, f: F) -> Result<R, ExitError> where
			F: FnOnce(
				$(impl_perform_popn_pushn!(INTERNAL_TYPE_RH256, $peek_pop)),*
			) -> Result<(($(impl_perform_popn_pushn!(INTERNAL_TYPE_H256, $peek_push)),*), R), ExitError>
		{
			match self.check_pop_push($pop_len, $push_len) {
				Ok(()) => (),
				Err(e) => return Err(e.into()),
			}

			let (p, ret) = match f($(self.unchecked_peek($peek_pop)),*) {
				Ok(p1) => p1,
				Err(e) => return Err(e.into()),
			};
			self.$pop_pushn_f($pop_len, p);

			Ok(ret)
		}
	};
	(INTERNAL_TYPE_RH256, $e:expr) => { &H256 };
	(INTERNAL_TYPE_H256, $e:expr) => { H256 };
}

impl Stack {
	/// Create a new stack with given limit.
	#[must_use]
	pub const fn new(limit: usize) -> Self {
		Self {
			data: Vec::new(),
			limit,
		}
	}

	/// Stack limit.
	#[inline]
	#[must_use]
	pub const fn limit(&self) -> usize {
		self.limit
	}

	/// Stack length.
	#[inline]
	#[must_use]
	pub fn len(&self) -> usize {
		self.data.len()
	}

	/// Whether the stack is empty.
	#[inline]
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}

	/// Stack data.
	#[inline]
	#[must_use]
	pub const fn data(&self) -> &Vec<H256> {
		&self.data
	}

	/// Clear the stack.
	pub fn clear(&mut self) {
		self.data.clear();
	}

	/// Pop a value from the stack.
	/// If the stack is already empty, returns the `StackUnderflow` error.
	#[inline]
	pub fn pop(&mut self) -> Result<H256, ExitException> {
		self.data.pop().ok_or(ExitException::StackUnderflow)
	}

	/// Push a new value into the stack.
	/// If it exceeds the stack limit, returns `StackOverflow` error and
	/// leaves the stack unchanged.
	#[inline]
	pub fn push(&mut self, value: H256) -> Result<(), ExitException> {
		if self.data.len() + 1 > self.limit {
			return Err(ExitException::StackOverflow);
		}
		self.data.push(value);
		Ok(())
	}

	/// Check whether it's possible to pop and push enough items in the stack.
	pub fn check_pop_push(&self, pop: usize, push: usize) -> Result<(), ExitException> {
		if self.data.len() < pop {
			return Err(ExitException::StackUnderflow);
		}
		if self.data.len() - pop + push + 1 > self.limit {
			return Err(ExitException::StackOverflow);
		}
		Ok(())
	}

	fn unchecked_peek(&self, no_from_top: usize) -> &H256 {
		&self.data[self.data.len() - no_from_top - 1]
	}

	fn unchecked_pop_push1(&mut self, pop: usize, p1: H256) {
		for _ in 0..pop {
			self.data.pop();
		}
		self.data.push(p1);
	}

	fn unchecked_pop_push0(&mut self, pop: usize, _p1: ()) {
		for _ in 0..pop {
			self.data.pop();
		}
	}

	/// Peek a value at given index for the stack, where the top of
	/// the stack is at index `0`. If the index is too large,
	/// `StackError::Underflow` is returned.
	#[inline]
	pub fn peek(&self, no_from_top: usize) -> Result<H256, ExitException> {
		if self.data.len() > no_from_top {
			Ok(self.data[self.data.len() - no_from_top - 1])
		} else {
			Err(ExitException::StackUnderflow)
		}
	}

	/// Set a value at given index for the stack, where the top of the
	/// stack is at index `0`. If the index is too large,
	/// `StackError::Underflow` is returned.
	#[inline]
	pub fn set(&mut self, no_from_top: usize, val: H256) -> Result<(), ExitException> {
		if self.data.len() > no_from_top {
			let len = self.data.len();
			self.data[len - no_from_top - 1] = val;
			Ok(())
		} else {
			Err(ExitException::StackUnderflow)
		}
	}

	impl_perform_popn_pushn!(perform_pop0_push1, 0, 1, (), (0), unchecked_pop_push1);
	impl_perform_popn_pushn!(perform_pop1_push0, 1, 0, (0), (), unchecked_pop_push0);
	impl_perform_popn_pushn!(perform_pop1_push1, 1, 1, (0), (0), unchecked_pop_push1);
	impl_perform_popn_pushn!(perform_pop2_push1, 2, 1, (0, 1), (0), unchecked_pop_push1);
	impl_perform_popn_pushn!(perform_pop3_push0, 3, 0, (0, 1, 2), (), unchecked_pop_push0);
	impl_perform_popn_pushn!(
		perform_pop4_push0,
		4,
		0,
		(0, 1, 2, 3),
		(),
		unchecked_pop_push0
	);
	impl_perform_popn_pushn!(
		perform_pop6_push0,
		6,
		0,
		(0, 1, 2, 3, 4, 5),
		(),
		unchecked_pop_push0
	);
	impl_perform_popn_pushn!(
		perform_pop7_push0,
		7,
		0,
		(0, 1, 2, 3, 4, 5, 6),
		(),
		unchecked_pop_push0
	);
}
