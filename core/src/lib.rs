#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;
extern crate alloc;

mod memory;
mod stack;
mod valids;
mod opcode;
mod error;
mod eval;
mod utils;

pub use crate::memory::Memory;
pub use crate::stack::Stack;
pub use crate::valids::Valids;
pub use crate::opcode::{Opcode, ExternalOpcode};
pub use crate::error::{Trap, Capture, ExitReason, ExitSucceed, ExitError};

use core::ops::Range;
use alloc::rc::Rc;
use primitive_types::U256;
use crate::eval::{eval, Control};

/// Core execution layer for EVM.
pub struct Machine {
	/// Program data.
	data: Rc<Vec<u8>>,
	/// Program code.
	code: Rc<Vec<u8>>,
	/// Program counter.
	position: Result<usize, ExitReason>,
	/// Return value.
	return_range: Range<U256>,
	/// Code validity maps.
	valids: Valids,
	/// Memory.
	memory: Memory,
	/// Stack.
	stack: Stack,
}

impl Machine {
	pub fn stack(&self) -> &Stack { &self.stack }
	pub fn stack_mut(&mut self) -> &mut Stack { &mut self.stack }
	pub fn memory(&self) -> &Memory { &self.memory }
	pub fn memory_mut(&mut self) -> &mut Memory { &mut self.memory }

	pub fn new(
		code: Rc<Vec<u8>>,
		data: Rc<Vec<u8>>,
		stack_limit: usize,
		memory_limit: usize
	) -> Self {
		let valids = Valids::new(&code[..]);

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

	pub fn exit(&mut self, reason: ExitReason) {
		self.position = Err(reason);
	}

	pub fn inspect(&self) -> Option<(Result<Opcode, ExternalOpcode>, &Stack)> {
		let position = match self.position {
			Ok(position) => position,
			Err(_) => return None,
		};
		self.code.get(position).map(|v| (Opcode::parse(*v), &self.stack))
	}

	pub fn return_value(&self) -> Vec<u8> {
		self.memory.get(
			self.return_range.start.as_usize(),
			self.return_range.end.as_usize() - self.return_range.start.as_usize(),
		)
	}

	pub fn run(&mut self) -> Capture<ExitReason, Trap> {
		loop {
			match self.step() {
				Ok(()) => (),
				Err(res) => return res,
			}
		}
	}

	pub fn step(&mut self) -> Result<(), Capture<ExitReason, Trap>> {
		let position = self.position.map_err(|reason| Capture::Exit(reason))?;

		match self.code.get(position).map(|v| Opcode::parse(*v)) {
			Some(Ok(opcode)) => {
				match eval(self, opcode, position) {
					Control::Continue(p) => {
						self.position = Ok(position + p);
						Ok(())
					},
					Control::Exit(e) => {
						self.position = Err(e);
						Err(Capture::Exit(e))
					},
					Control::Jump(p) => {
						if self.valids.is_valid(p) {
							self.position = Ok(p);
							Ok(())
						} else {
							self.position = Err(ExitError::InvalidJump.into());
							Err(Capture::Exit(ExitError::InvalidJump.into()))
						}
					},
				}
			},
			Some(Err(external)) => {
				self.position = Ok(position + 1);
				Err(Capture::Trap(external))
			},
			None => {
				self.position = Err(ExitSucceed::Stopped.into());
				Err(Capture::Exit(ExitSucceed::Stopped.into()))
			},
		}
	}
}
