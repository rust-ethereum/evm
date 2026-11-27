mod call_input;
mod memory;
mod stack;

use alloc::{rc::Rc, vec::Vec};

pub use self::{call_input::CallInput, memory::Memory, stack::Stack};

/// Core execution layer for EVM.
pub struct Machine<S> {
	/// Program data.
	pub(crate) data: CallInput,
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
	/// Whether the input of a call is shared with the callee.
	share_call_input: bool,
	/// Input range of the pending call, if any.
	pub(crate) call_input_range: Option<(usize, usize)>,
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
		Self::new_with_input(
			code,
			CallInput::from(Rc::try_unwrap(data).unwrap_or_else(|data| (*data).clone())),
			stack_limit,
			memory_limit,
			state,
		)
	}

	/// Create a new machine with given code and call input.
	pub fn new_with_input(
		code: Rc<Vec<u8>>,
		data: CallInput,
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
			share_call_input: false,
			call_input_range: None,
		}
	}

	/// Share the input of a call with the callee instead of copying it.
	///
	/// The callee then reads the input from this machine's memory through
	/// [`Machine::shared_call_input`]. Sharing is off by default.
	///
	/// While sharing is on, a call trap that the standard opcode function
	/// builds carries an empty `CallTrap::input`, and this machine's
	/// [`Memory::data`] is empty until the call's feedback arrives.
	///
	/// The shared input grows this machine's memory with zeros up to the end
	/// of the input range, and that growth does not apply the memory limit:
	/// only the memory gas bounds it. Without memory metering, a call can
	/// therefore allocate up to `in_offset + in_len` bytes, where a copy of
	/// the input allocated `in_len` bytes.
	pub fn enable_call_input_sharing(&mut self) {
		self.share_call_input = true;
	}

	/// Whether the machine shares call input with the callee.
	pub(crate) const fn shares_call_input(&self) -> bool {
		self.share_call_input
	}

	/// Input of the pending call, shared from this machine's memory.
	///
	/// Returns `None` if the machine does not share call input, or if it has no
	/// pending call. Each call returns another share of the same range: the
	/// bytes are read from the memory when this method is called, not when the
	/// opcode that recorded the range trapped.
	///
	/// The result shares the memory's buffer, so keeping it after the callee
	/// ends makes the next write to the memory copy the whole memory.
	pub fn shared_call_input(&mut self) -> Option<CallInput> {
		if !self.share_call_input {
			return None;
		}
		let (offset, len) = self.call_input_range?;
		Some(self.memory.share(offset, len))
	}

	/// Forget the input range of the pending call.
	pub(crate) fn clear_call_input_range(&mut self) {
		if self.call_input_range.is_some() {
			self.call_input_range = None;
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

#[cfg(test)]
mod tests {
	use alloc::{rc::Rc, vec, vec::Vec};

	use super::{CallInput, Machine};

	#[test]
	fn machine_new_does_not_copy_a_unique_rc() {
		let data = vec![1u8, 2, 3, 4];
		let ptr = data.as_ptr();

		let machine = Machine::new(Rc::new(Vec::new()), Rc::new(data), 1024, 10000, ());

		assert_eq!(machine.data.as_ptr(), ptr);
		assert_eq!(&machine.data[..], &[1, 2, 3, 4]);
	}

	#[test]
	fn call_input_from_vec_copies_nothing() {
		let data = vec![1u8, 2, 3, 4];
		let ptr = data.as_ptr();

		let input = CallInput::from(data);

		assert_eq!(input.as_ptr(), ptr);
		assert_eq!(&input[..], &[1, 2, 3, 4]);
	}
}
