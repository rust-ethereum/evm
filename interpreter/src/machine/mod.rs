mod memory;
mod stack;

use alloc::{rc::Rc, vec::Vec};

pub use self::{memory::Memory, stack::Stack};

/// Core execution layer for EVM.
pub struct Machine<S> {
	/// Program data.
	pub(crate) data: Rc<Vec<u8>>,
	/// Program code.
	pub(crate) code: Rc<Vec<u8>>,
	/// Return value. Note the difference between `retbuf`.
	/// A `retval` holds what's returned by the current machine, with `RETURN` or `REVERT` opcode.
	/// A `retbuf` holds the buffer of returned value by sub-calls.
	pub retval: Vec<u8>,
	/// Memory.
	pub memory: Memory,
	/// Stack.
	pub stack: Stack,
	/// Extra state,
	pub state: S,
}

impl<S> Machine<S> {
	/// Create a new machine with given code and data.
	pub fn new(
		code: Rc<Vec<u8>>,
		data: Rc<Vec<u8>>,
		stack_limit: usize,
		memory_limit: usize,
		state: S,
	) -> Self {
		Self {
			data,
			code,
			retval: Vec::new(),
			memory: Memory::new(memory_limit),
			stack: Stack::new(stack_limit),
			state,
		}
	}

	/// Machine code.
	pub fn code(&self) -> &[u8] {
		&self.code
	}

	/// Whether the machine has empty code.
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.code.is_empty()
	}
}
