//! Core layer for EVM.

// #![deny(warnings)]
// #![forbid(unsafe_code, unused_variables, unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod call_create;
mod error;
mod eval;
mod memory;
mod opcode;
mod runtime;
mod stack;
pub mod utils;
mod valids;

pub use crate::error::{Capture, ExitError, ExitException, ExitFatal, ExitResult, ExitSucceed};
pub use crate::eval::{Control, Efn, Etable};
pub use crate::memory::Memory;
pub use crate::opcode::Opcode;
pub use crate::runtime::{
	CallCreateTrap, Context, Log, RuntimeBackend, RuntimeBaseBackend, RuntimeEnvironment,
	RuntimeState, TransactionContext, Transfer,
};
pub use crate::stack::Stack;
pub use crate::valids::Valids;

use alloc::rc::Rc;
use alloc::vec::Vec;

/// Core execution layer for EVM.
pub struct Machine<S> {
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

impl<S> Machine<S> {
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

	pub fn perform<R, F: FnOnce(&mut Self) -> Result<R, ExitError>>(
		&mut self,
		f: F,
	) -> Result<R, ExitError> {
		match f(self) {
			Ok(r) => Ok(r),
			Err(e) => {
				self.exit();
				Err(e)
			}
		}
	}

	/// Explicit exit of the machine. Further step will return error.
	pub fn exit(&mut self) {
		self.position = self.code.len();
	}

	/// Return value of the machine.
	pub fn into_retbuf(self) -> Vec<u8> {
		self.memory.into_data()
	}

	/// Inspect the machine's next opcode and current stack.
	pub fn inspect(&self) -> Option<(Opcode, &Stack)> {
		self.code
			.get(self.position)
			.map(|v| (Opcode(*v), &self.stack))
	}

	/// Loop stepping the machine, until it stops.
	pub fn run<H, Tr, F>(
		&mut self,
		handle: &mut H,
		etable: &Etable<S, H, Tr, F>,
	) -> Capture<ExitResult, Tr>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		loop {
			match self.step(handle, etable) {
				Ok(()) => (),
				Err(res) => return res,
			}
		}
	}

	/// Step the machine N times.
	pub fn stepn<H, Tr, F>(
		&mut self,
		n: usize,
		handle: &mut H,
		etable: &Etable<S, H, Tr, F>,
	) -> Result<(), Capture<ExitResult, Tr>>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		for _ in 0..n {
			match self.step(handle, etable) {
				Ok(()) => (),
				Err(res) => return Err(res),
			}
		}

		Ok(())
	}

	#[inline]
	/// Step the machine, executing one opcode. It then returns.
	pub fn step<H, Tr, F>(
		&mut self,
		handle: &mut H,
		etable: &Etable<S, H, Tr, F>,
	) -> Result<(), Capture<ExitResult, Tr>>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		let position = self.position;
		if position >= self.code.len() {
			return Err(Capture::Exit(ExitFatal::AlreadyExited.into()));
		}

		let opcode = Opcode(self.code[position]);
		let control = etable[opcode.as_usize()](self, handle, opcode, self.position);

		let mut ret = match control {
			Control::Continue => {
				self.position += 1;
				Ok(())
			}
			Control::ContinueN(p) => {
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
			Control::Trap(opcode) => Err(Capture::Trap(opcode)),
		};

		if position >= self.code.len() {
			ret = Err(Capture::Exit(ExitSucceed::Stopped.into()));
		}

		ret
	}

	/// Pick the next opcode.
	pub fn peek_opcode(&self) -> Option<Opcode> {
		self.code.get(self.position).map(|opcode| Opcode(*opcode))
	}

	pub fn advance(&mut self) {
		if self.position == self.code.len() {
			return;
		}

		self.position += 1;
	}
}
