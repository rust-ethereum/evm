//! Core layer for EVM.

#![deny(warnings)]
#![forbid(unsafe_code, unused_variables, unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod error;
mod eval;
mod external;
mod memory;
mod opcode;
mod stack;
mod utils;
mod valids;

pub use crate::error::{Capture, ExitError, ExitFatal, ExitReason, ExitRevert, ExitSucceed, Trap};
pub use crate::eval::{Control, Efn, Etable};
pub use crate::external::ExternalOperation;
pub use crate::memory::Memory;
pub use crate::opcode::Opcode;
pub use crate::stack::Stack;
pub use crate::valids::Valids;

use alloc::rc::Rc;
use alloc::vec::Vec;

pub trait State: 'static + Sized {
	/// Etable of the state.
	fn etable() -> &'static Etable<Self>;
}

static CORE_ETABLE: Etable<()> = Etable::<()>::core();

impl State for () {
	#[inline]
	fn etable() -> &'static Etable<Self> {
		&CORE_ETABLE
	}
}

/// Core execution layer for EVM.
pub struct Machine<S = ()> {
	/// Program data.
	data: Rc<Vec<u8>>,
	/// Program code.
	code: Rc<Vec<u8>>,
	/// Program counter.
	position: usize,
	/// Code validity maps.
	valids: Valids,
	/// Memory.
	pub memory: Memory,
	/// Stack.
	pub stack: Stack,
	/// Extra state,
	pub state: S,
}

impl<S: State> Machine<S> {
	/// Return a reference of the program counter.
	pub fn position(&self) -> usize {
		self.position
	}

	/// Create a new machine with given code and data.
	pub fn new(
		code: Rc<Vec<u8>>,
		data: Rc<Vec<u8>>,
		stack_limit: usize,
		memory_limit: usize,
		state: S,
	) -> Self {
		let valids = Valids::new(&code[..]);

		Self {
			data,
			code,
			position: 0,
			valids,
			memory: Memory::new(memory_limit),
			stack: Stack::new(stack_limit),
			state,
		}
	}

	/// Explicit exit of the machine. Further step will return error.
	pub fn exit(&mut self) {
		self.position = self.code.len();
	}

	/// Inspect the machine's next opcode and current stack.
	pub fn inspect(&self) -> Option<(Opcode, &Stack)> {
		self.code
			.get(self.position)
			.map(|v| (Opcode(*v), &self.stack))
	}

	/// Loop stepping the machine, until it stops.
	pub fn run(&mut self) -> Capture<ExitReason, Trap> {
		loop {
			match self.step() {
				Ok(()) => (),
				Err(res) => return res,
			}
		}
	}

	#[inline]
	/// Step the machine, executing one opcode. It then returns.
	pub fn step(&mut self) -> Result<(), Capture<ExitReason, Trap>> {
		let position = self.position;
		if position >= self.code.len() {
			return Err(Capture::Exit(ExitSucceed::Stopped.into()));
		}

		let opcode = Opcode(self.code[position]);
		let control = S::etable()[opcode.as_usize()](self, opcode, self.position);

		match control {
			Control::Continue(p) => {
				self.position = position + p;
				Ok(())
			}
			Control::Exit(e) => {
				self.position = self.code.len();
				Err(Capture::Exit(e))
			}
			Control::Jump(p) => {
				self.position = p;
				Ok(())
			}
			Control::Trap(opcode) => {
				#[cfg(feature = "force-debug")]
				log::trace!(target: "evm", "OpCode Trap: {:?}", opcode);

				self.position = position + 1;
				Err(Capture::Trap(opcode))
			}
		}
	}
}
