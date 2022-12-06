//! Core layer for EVM.

#![deny(warnings)]
#![forbid(unsafe_code, unused_variables, unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate core;

mod error;
mod eval;
mod memory;
mod opcode;
mod stack;
mod utils;
mod valids;

pub use crate::error::{Capture, ExitError, ExitFatal, ExitReason, ExitRevert, ExitSucceed, Trap};
pub use crate::memory::Memory;
pub use crate::opcode::Opcode;
pub use crate::stack::Stack;
pub use crate::valids::Valids;

use crate::eval::{eval, Control};
use alloc::rc::Rc;
use core::ops::Range;
use elrond_wasm::{api::ManagedTypeApi, types::ManagedVec};
use primitive_types::U256;

/// Core execution layer for EVM.
pub struct Machine<M: ManagedTypeApi> {
	/// Program data.
	data: Rc<ManagedVec<M, u8>>,
	/// Program code.
	code: Rc<ManagedVec<M, u8>>,
	/// Program counter.
	position: Result<usize, ExitReason>,
	/// Return value.
	return_range: Range<U256>,
	/// Code validity maps.
	valids: Valids<M>,
	/// Memory.
	memory: Memory<M>,
	/// Stack.
	stack: Stack<M>,
}

impl<M: ManagedTypeApi> Machine<M> {
	/// Reference of machine stack.
	pub fn stack(&self) -> &Stack<M> {
		&self.stack
	}
	/// Mutable reference of machine stack.
	pub fn stack_mut(&mut self) -> &mut Stack<M> {
		&mut self.stack
	}
	/// Reference of machine memory.
	pub fn memory(&self) -> &Memory<M> {
		&self.memory
	}
	/// Mutable reference of machine memory.
	pub fn memory_mut(&mut self) -> &mut Memory<M> {
		&mut self.memory
	}
	/// Return a reference of the program counter.
	pub fn position(&self) -> &Result<usize, ExitReason> {
		&self.position
	}

	/// Create a new machine with given code and data.
	pub fn new(
		code: Rc<ManagedVec<M, u8>>,
		data: Rc<ManagedVec<M, u8>>,
		stack_limit: usize,
		memory_limit: usize,
	) -> Self {
		let valids = Valids::new(&code);

		Self {
			data,
			code,
			position: Ok(0),
			return_range: U256::zero()..U256::zero(),
			valids,
			memory: Memory::new(memory_limit),
			stack: Stack::new(stack_limit),
		}
	}

	/// Explicit exit of the machine. Further step will return error.
	pub fn exit(&mut self, reason: ExitReason) {
		self.position = Err(reason);
	}

	/// Inspect the machine's next opcode and current stack.
	pub fn inspect(&self) -> Option<(Opcode, &Stack<M>)> {
		let position = match self.position {
			Ok(position) => position,
			Err(_) => return None,
		};
		let value = self.code.get(position);
		Some((Opcode(value), &self.stack))
	}

	/// Copy and get the return value of the machine, if any.
	pub fn return_value(&self) -> ManagedVec<M, u8> {
		if self.return_range.start > U256::from(usize::MAX) {
			let mut ret = ManagedVec::new();
			let size = (self.return_range.end - self.return_range.start).as_usize();
			for i in 0..size {
				ret.push(0);
			}
			ret
		} else if self.return_range.end > U256::from(usize::MAX) {
			let mut ret = self.memory.get(
				self.return_range.start.as_usize(),
				usize::MAX - self.return_range.start.as_usize(),
			);
			while ret.len() < (self.return_range.end - self.return_range.start).as_usize() {
				ret.push(0);
			}
			ret
		} else {
			self.memory.get(
				self.return_range.start.as_usize(),
				(self.return_range.end - self.return_range.start).as_usize(),
			)
		}
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
		let position = *self
			.position
			.as_ref()
			.map_err(|reason| Capture::Exit(reason.clone()))?;

		let v = self.code.get(position);

		match Some(Opcode(v)) {
			Some(opcode) => match eval(self, opcode, position) {
				Control::Continue(p) => {
					self.position = Ok(position + p);
					Ok(())
				}
				Control::Exit(e) => {
					self.position = Err(e.clone());
					Err(Capture::Exit(e))
				}
				Control::Jump(p) => {
					self.position = Ok(p);
					Ok(())
				}
				Control::Trap(opcode) => {
					self.position = Ok(position + 1);
					Err(Capture::Trap(opcode))
				}
			},
			None => {
				self.position = Err(ExitSucceed::Stopped.into());
				Err(Capture::Exit(ExitSucceed::Stopped.into()))
			}
		}
	}
}
