//! Core layer for EVM.

// #![deny(warnings)]
// #![forbid(unsafe_code, unused_variables, unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod error;
mod eval;
mod memory;
mod opcode;
mod runtime;
mod stack;
mod utils;
mod valids;

pub use crate::error::{ExitError, ExitException, ExitFatal, ExitResult, ExitSucceed};
pub use crate::eval::{Control, Efn, Etable};
pub use crate::memory::Memory;
pub use crate::opcode::Opcode;
pub use crate::runtime::{
	CallScheme, Context, CreateScheme, Handler, RuntimeBackend, RuntimeCallTrapData,
	RuntimeCreateTrapData, RuntimeEnvironmentalBackend, RuntimeFullBackend,
	RuntimeGasometerBackend, RuntimeState, RuntimeTrap, RuntimeTrapData, Transfer,
};
pub use crate::stack::Stack;
pub use crate::valids::Valids;

use alloc::rc::Rc;
use alloc::vec::Vec;
use core::convert::Infallible;

pub type StandardMachine = Machine<RuntimeState>;
pub type StandardControl = Control<StandardTrapData>;
pub type StandardEfn<H> = Efn<RuntimeState, H, StandardTrapData>;
pub type StandardEtable<H> = Etable<RuntimeState, H, StandardTrap>;
pub type StandardTrapData = RuntimeTrapData;
pub type StandardTrap = RuntimeTrap<RuntimeState>;

/// Trap which indicates that an `ExternalOpcode` has to be handled.
pub trait Trap<S> {
	type Data;

	fn from_data(data: Self::Data, machine: Machine<S>) -> Self;
}

impl<S> Trap<S> for Infallible {
	type Data = Infallible;

	fn from_data(data: Infallible, _machine: Machine<S>) -> Self {
		match data {}
	}
}

/// Capture represents the result of execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capture<E, T> {
	/// The machine has exited. It cannot be executed again.
	Exit(E),
	/// The machine has trapped. It is waiting for external information, and can
	/// be executed again.
	Trap(T),
}

impl<E, T> Capture<E, T> {
	pub fn exit(self) -> Option<E> {
		if let Self::Exit(e) = self {
			Some(e)
		} else {
			None
		}
	}

	pub fn trap(self) -> Option<T> {
		if let Self::Trap(t) = self {
			Some(t)
		} else {
			None
		}
	}
}

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

	/// Loop stepping the machine, until it stops.
	pub fn run<H, Tr, F>(
		mut self,
		handle: &mut H,
		etable: &Etable<S, H, Tr, F>,
	) -> Capture<(Self, ExitResult), Tr>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr::Data>,
		Tr: Trap<S>,
	{
		loop {
			match self.step(handle, etable) {
				Ok(s) => {
					self = s;
				}
				Err(res) => return res,
			}
		}
	}

	/// Step the machine N times.
	pub fn stepn<H, Tr, F>(
		mut self,
		n: usize,
		handle: &mut H,
		etable: &Etable<S, H, Tr, F>,
	) -> Result<Self, Capture<(Self, ExitResult), Tr>>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr::Data>,
		Tr: Trap<S>,
	{
		for _ in 0..n {
			match self.step(handle, etable) {
				Ok(s) => {
					self = s;
				}
				Err(res) => return Err(res),
			}
		}

		Ok(self)
	}

	#[inline]
	/// Step the machine, executing one opcode. It then returns.
	pub fn step<H, Tr, F>(
		mut self,
		handle: &mut H,
		etable: &Etable<S, H, Tr, F>,
	) -> Result<Self, Capture<(Self, ExitResult), Tr>>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr::Data>,
		Tr: Trap<S>,
	{
		let position = self.position;
		if position >= self.code.len() {
			return Err(Capture::Exit((self, ExitSucceed::Stopped.into())));
		}

		let opcode = Opcode(self.code[position]);
		let position = self.position;
		let control = etable[opcode.as_usize()](&mut self, handle, opcode, position);

		match control {
			Control::Continue => {
				self.position += 1;
				Ok(self)
			}
			Control::ContinueN(p) => {
				self.position = position + p;
				Ok(self)
			}
			Control::Exit(e) => {
				self.position = self.code.len();
				Err(Capture::Exit((self, e)))
			}
			Control::Jump(p) => {
				self.position = p;
				Ok(self)
			}
			Control::Trap(data) => {
				self.position = position + 1;
				Err(Capture::Trap(Tr::from_data(data, self)))
			}
		}
	}

	/// Pick the next opcode.
	pub fn peek_opcode(&self) -> Option<Opcode> {
		self.code.get(self.position).map(|opcode| Opcode(*opcode))
	}
}
