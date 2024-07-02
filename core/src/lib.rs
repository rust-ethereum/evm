//! Core layer for EVM.

#![deny(warnings)]
#![forbid(unsafe_code, unused_variables, unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
pub mod prelude {
	pub use alloc::{borrow::Cow, rc::Rc, vec, vec::Vec};
}
#[cfg(feature = "std")]
pub mod prelude {
	pub use std::{borrow::Cow, rc::Rc, vec::Vec};
}

mod error;
mod eval;
mod external;
mod memory;
mod opcode;
mod stack;
pub mod utils;
mod valids;

pub use crate::error::{Capture, ExitError, ExitFatal, ExitReason, ExitRevert, ExitSucceed, Trap};
pub use crate::external::ExternalOperation;
pub use crate::memory::Memory;
pub use crate::opcode::Opcode;
pub use crate::stack::Stack;
pub use crate::valids::Valids;

use crate::eval::{eval, Control};
use crate::prelude::*;
use crate::utils::USIZE_MAX;
use core::ops::Range;
use primitive_types::{H160, U256};

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

/// EVM interpreter handler.
pub trait InterpreterHandler {
	fn before_eval(&mut self);

	fn after_eval(&mut self);

	fn before_bytecode(
		&mut self,
		opcode: Opcode,
		pc: usize,
		machine: &Machine,
		address: &H160,
	) -> Result<(), ExitError>;

	// Only invoked if #[cfg(feature = "tracing")]
	fn after_bytecode(&mut self, result: &Result<(), Capture<ExitReason, Trap>>, machine: &Machine);
}

impl Machine {
	/// Reference of machine stack.
	#[must_use]
	pub const fn stack(&self) -> &Stack {
		&self.stack
	}
	/// Mutable reference of machine stack.
	pub fn stack_mut(&mut self) -> &mut Stack {
		&mut self.stack
	}
	/// Reference of machine memory.
	#[must_use]
	pub const fn memory(&self) -> &Memory {
		&self.memory
	}
	/// Mutable reference of machine memory.
	pub fn memory_mut(&mut self) -> &mut Memory {
		&mut self.memory
	}
	/// Return a reference of the program counter.
	pub const fn position(&self) -> &Result<usize, ExitReason> {
		&self.position
	}

	/// Create a new machine with given code and data.
	#[must_use]
	pub fn new(
		code: Rc<Vec<u8>>,
		data: Rc<Vec<u8>>,
		stack_limit: usize,
		memory_limit: usize,
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

	/// Explicit exit of the machine. Further step will return error.
	pub fn exit(&mut self, reason: ExitReason) {
		self.position = Err(reason);
	}

	/// Inspect the machine's next opcode and current stack.
	#[must_use]
	pub fn inspect(&self) -> Option<(Opcode, &Stack)> {
		let Ok(position) = self.position else {
			return None;
		};
		self.code.get(position).map(|v| (Opcode(*v), &self.stack))
	}

	/// Copy and get the return value of the machine, if any.
	#[must_use]
	pub fn return_value(&self) -> Vec<u8> {
		if self.return_range.start > USIZE_MAX {
			vec![0; (self.return_range.end - self.return_range.start).as_usize()]
		} else if self.return_range.end > USIZE_MAX {
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
		let mut handler = SimpleInterpreterHandler::default();
		let address = H160::default();
		loop {
			match self.step(&mut handler, &address) {
				Ok(()) => (),
				Err(res) => return res,
			}
		}
	}

	#[inline]
	/// Step the machine, executing until exit or trap.
	pub fn step<H: InterpreterHandler>(
		&mut self,
		handler: &mut H,
		address: &H160,
	) -> Result<(), Capture<ExitReason, Trap>> {
		let position = *self
			.position
			.as_ref()
			.map_err(|reason| Capture::Exit(reason.clone()))?;
		match eval(self, position, handler, address) {
			Control::Exit(e) => {
				self.position = Err(e.clone());
				Err(Capture::Exit(e))
			}
			Control::Trap(opcode) => Err(Capture::Trap(opcode)),
			Control::Continue(_) | Control::Jump(_) => Ok(()),
		}
	}
}

pub struct SimpleInterpreterHandler {
	pub executed: u64,
	pub profile: [u64; 256],
	pub address: H160,
}

impl SimpleInterpreterHandler {
	#[must_use]
	pub const fn new(address: H160) -> Self {
		Self {
			executed: 0,
			profile: [0; 256],
			address,
		}
	}
}

impl Default for SimpleInterpreterHandler {
	fn default() -> Self {
		Self {
			executed: 0,
			profile: [0; 256],
			address: H160::default(),
		}
	}
}

impl InterpreterHandler for SimpleInterpreterHandler {
	fn before_eval(&mut self) {}

	fn after_eval(&mut self) {}

	#[inline]
	fn before_bytecode(
		&mut self,
		opcode: Opcode,
		_pc: usize,
		_machine: &Machine,
		_address: &H160,
	) -> Result<(), ExitError> {
		self.executed += 1;
		self.profile[opcode.as_usize()] += 1;
		Ok(())
	}

	#[inline]
	fn after_bytecode(
		&mut self,
		_result: &Result<(), Capture<ExitReason, Trap>>,
		_machine: &Machine,
	) {
	}
}
