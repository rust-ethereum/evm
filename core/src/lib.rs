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
pub use crate::external::ExternalOperation;
pub use crate::memory::Memory;
pub use crate::opcode::Opcode;
pub use crate::stack::Stack;
pub use crate::valids::Valids;

use crate::eval::{eval, Control};
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::ops::Range;
use primitive_types::{H160, U256};

/// EIP-7702 delegation designator prefix
pub const EIP_7702_DELEGATION_PREFIX: &[u8] = &[0xef, 0x01, 0x00];

/// EIP-7702 delegation designator full length (prefix + address)
pub const EIP_7702_DELEGATION_SIZE: usize = 23; // 3 bytes prefix + 20 bytes address

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
	/// Reference of machine stack.
	pub fn stack(&self) -> &Stack {
		&self.stack
	}
	/// Mutable reference of machine stack.
	pub fn stack_mut(&mut self) -> &mut Stack {
		&mut self.stack
	}
	/// Reference of machine memory.
	pub fn memory(&self) -> &Memory {
		&self.memory
	}
	/// Mutable reference of machine memory.
	pub fn memory_mut(&mut self) -> &mut Memory {
		&mut self.memory
	}
	/// Return a reference of the program counter.
	pub fn position(&self) -> &Result<usize, ExitReason> {
		&self.position
	}

	/// Create a new machine with given code and data.
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
	pub fn inspect(&self) -> Option<(Opcode, &Stack)> {
		let position = match self.position {
			Ok(position) => position,
			Err(_) => return None,
		};
		self.code.get(position).map(|v| (Opcode(*v), &self.stack))
	}

	/// Copy and get the return value of the machine, if any.
	#[allow(clippy::slow_vector_initialization)]
	// Clippy complains about not using `no_std`. However, we need to support
	// `no_std` and we can't use that.
	pub fn return_value(&self) -> Vec<u8> {
		if self.return_range.start > U256::from(usize::MAX) {
			let mut ret = Vec::new();
			ret.resize(
				(self.return_range.end - self.return_range.start).as_usize(),
				0,
			);
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

		match self.code.get(position).map(|v| Opcode(*v)) {
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
					#[cfg(feature = "force-debug")]
					log::trace!(target: "evm", "OpCode Trap: {:?}", opcode);

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

/// Check if code is an EIP-7702 delegation designator
pub fn is_delegation_designator(code: &[u8]) -> bool {
	code.len() == EIP_7702_DELEGATION_SIZE && code.starts_with(EIP_7702_DELEGATION_PREFIX)
}

/// Extract the delegated address from EIP-7702 delegation designator
pub fn extract_delegation_address(code: &[u8]) -> Option<H160> {
	if is_delegation_designator(code) {
		let mut address_bytes = [0u8; 20];
		address_bytes.copy_from_slice(&code[3..23]);
		Some(H160::from(address_bytes))
	} else {
		None
	}
}

/// Create EIP-7702 delegation designator
pub fn create_delegation_designator(address: H160) -> Vec<u8> {
	let mut designator = Vec::with_capacity(EIP_7702_DELEGATION_SIZE);
	designator.extend_from_slice(EIP_7702_DELEGATION_PREFIX);
	designator.extend_from_slice(address.as_bytes());
	designator
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_delegation_designator_creation() {
		let address = H160::from_slice(&[1u8; 20]);
		let designator = create_delegation_designator(address);

		assert_eq!(designator.len(), EIP_7702_DELEGATION_SIZE);
		assert_eq!(&designator[0..3], EIP_7702_DELEGATION_PREFIX);
		assert_eq!(&designator[3..23], address.as_bytes());
	}

	#[test]
	fn test_delegation_designator_detection() {
		let address = H160::from_slice(&[1u8; 20]);
		let designator = create_delegation_designator(address);

		assert!(is_delegation_designator(&designator));
		assert_eq!(extract_delegation_address(&designator), Some(address));
	}

	#[test]
	fn test_non_delegation_code() {
		let regular_code = vec![0x60, 0x00]; // PUSH1 0
		assert!(!is_delegation_designator(&regular_code));
		assert_eq!(extract_delegation_address(&regular_code), None);
	}
}
