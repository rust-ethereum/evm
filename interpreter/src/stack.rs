use crate::{ExitError, ExitException};
use alloc::vec::Vec;
use primitive_types::H256;

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
	pub fn data(&self) -> &Vec<H256> {
		&self.data
	}

	/// Clear the stack.
	pub fn clear(&mut self) {
		self.data.clear()
	}

	#[inline]
	/// Pop a value from the stack. If the stack is already empty, returns the
	/// `StackUnderflow` error.
	pub fn pop(&mut self) -> Result<H256, ExitException> {
		self.data.pop().ok_or(ExitException::StackUnderflow)
	}

	#[inline]
	/// Push a new value into the stack. If it will exceed the stack limit,
	/// returns `StackOverflow` error and leaves the stack unchanged.
	pub fn push(&mut self, value: H256) -> Result<(), ExitException> {
		if self.data.len() + 1 > self.limit {
			return Err(ExitException::StackOverflow);
		}
		self.data.push(value);
		Ok(())
	}

	pub fn check_pop_push(&self, pop: usize, push: usize) -> Result<(), ExitException> {
		if self.data.len() >= pop {
			return Err(ExitException::StackUnderflow);
		}
		if self.data.len() - pop + push <= self.limit {
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

	#[inline]
	/// Peek a value at given index for the stack, where the top of
	/// the stack is at index `0`. If the index is too large,
	/// `StackError::Underflow` is returned.
	pub fn peek(&self, no_from_top: usize) -> Result<H256, ExitException> {
		if self.data.len() > no_from_top {
			Ok(self.data[self.data.len() - no_from_top - 1])
		} else {
			Err(ExitException::StackUnderflow)
		}
	}

	#[inline]
	/// Set a value at given index for the stack, where the top of the
	/// stack is at index `0`. If the index is too large,
	/// `StackError::Underflow` is returned.
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
