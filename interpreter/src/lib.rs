//! Core layer for EVM.

// #![deny(warnings)]
// #![forbid(unsafe_code, unused_variables, unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod call_create;
mod error;
mod etable;
pub mod eval;
mod interpreter;
mod memory;
mod opcode;
mod runtime;
mod stack;
mod trap;
pub mod utils;
mod valids;

pub use crate::error::{Capture, ExitError, ExitException, ExitFatal, ExitResult, ExitSucceed};
pub use crate::etable::{Control, Efn, Etable, EtableSet};
pub use crate::interpreter::{EtableInterpreter, Interpreter, StepInterpreter};
pub use crate::memory::Memory;
pub use crate::opcode::Opcode;
pub use crate::runtime::{
	Context, GasState, Log, RuntimeBackend, RuntimeBaseBackend, RuntimeEnvironment, RuntimeState,
	TransactionContext, Transfer,
};
pub use crate::stack::Stack;
pub use crate::trap::{TrapConstruct, TrapConsume};
pub use crate::valids::Valids;

use alloc::rc::Rc;
use alloc::vec::Vec;

/// Core execution layer for EVM.
pub struct Machine<S> {
	/// Program data.
	data: Rc<Vec<u8>>,
	/// Program code.
	code: Rc<Vec<u8>>,
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
	/// Machine code.
	pub fn code(&self) -> &[u8] {
		&self.code
	}

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

	/// Whether the machine has empty code.
	pub fn is_empty(&self) -> bool {
		self.code.is_empty()
	}
}
