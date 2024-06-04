use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use crate::{
	error::{Capture, ExitError, ExitException, ExitFatal, ExitResult, ExitSucceed},
	etable::{Control, EtableSet},
	interpreter::{valids::Valids, Interpreter, RunInterpreter, StepInterpreter},
	machine::{Machine, Stack},
	opcode::Opcode,
};

pub struct EtableInterpreter<'etable, ES: EtableSet> {
	valids: Valids,
	position: usize,
	machine: Machine<ES::State>,
	etable: &'etable ES,
}

impl<'etable, ES: EtableSet> Deref for EtableInterpreter<'etable, ES> {
	type Target = Machine<ES::State>;

	fn deref(&self) -> &Machine<ES::State> {
		&self.machine
	}
}

impl<'etable, ES: EtableSet> DerefMut for EtableInterpreter<'etable, ES> {
	fn deref_mut(&mut self) -> &mut Machine<ES::State> {
		&mut self.machine
	}
}

impl<'etable, ES> EtableInterpreter<'etable, ES>
where
	ES: EtableSet,
{
	/// Return a reference of the program counter.
	pub const fn position(&self) -> usize {
		self.position
	}

	pub fn new(machine: Machine<ES::State>, etable: &'etable ES) -> Self {
		let valids = Valids::new(&machine.code[..]);

		Self {
			machine,
			valids,
			position: 0,
			etable,
		}
	}

	pub fn deconstruct(self) -> Machine<ES::State> {
		self.machine
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

	/// Perform any operation. If the operation fails, then set the machine
	/// status to already exited.
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

	/// Pick the next opcode.
	pub fn peek_opcode(&self) -> Option<Opcode> {
		self.code.get(self.position).map(|opcode| Opcode(*opcode))
	}
}

impl<'etable, ES: EtableSet> Interpreter for EtableInterpreter<'etable, ES> {
	type State = ES::State;

	fn machine(&self) -> &Machine<ES::State> {
		&self.machine
	}

	fn machine_mut(&mut self) -> &mut Machine<ES::State> {
		&mut self.machine
	}

	fn deconstruct(self) -> (ES::State, Vec<u8>) {
		(self.machine.state, self.machine.retval)
	}

	fn advance(&mut self) {
		if self.position == self.code.len() {
			return;
		}

		self.position += 1;
	}
}

impl<'etable, H, Tr, ES> RunInterpreter<H, Tr> for EtableInterpreter<'etable, ES>
where
	ES: EtableSet<Handle = H, Trap = Tr>,
{
	fn run(&mut self, handle: &mut H) -> Capture<ExitResult, Tr> {
		loop {
			match self.step(handle) {
				Ok(()) => (),
				Err(res) => return res,
			}
		}
	}
}

impl<'etable, H, Tr, ES> StepInterpreter<H, Tr> for EtableInterpreter<'etable, ES>
where
	ES: EtableSet<Handle = H, Trap = Tr>,
{
	#[inline]
	fn step(&mut self, handle: &mut H) -> Result<(), Capture<ExitResult, Tr>> {
		if self.is_empty() {
			return Err(Capture::Exit(ExitSucceed::Stopped.into()));
		}

		let position = self.position;
		if position >= self.code.len() {
			return Err(Capture::Exit(ExitFatal::AlreadyExited.into()));
		}

		let opcode = Opcode(self.code[position]);
		let control = self
			.etable
			.eval(&mut self.machine, handle, opcode, self.position);

		match control {
			Control::Continue => {
				self.position += 1;
			}
			Control::ContinueN(p) => {
				self.position = position + p;
			}
			Control::Exit(e) => {
				self.position = self.code.len();
				return Err(Capture::Exit(e));
			}
			Control::Jump(p) => {
				if self.valids.is_valid(p) {
					self.position = p;
				} else {
					self.position = self.code.len();
					return Err(Capture::Exit(ExitException::InvalidJump.into()));
				}
			}
			Control::Trap(opcode) => return Err(Capture::Trap(opcode)),
		};

		if self.position >= self.code.len() {
			return Err(Capture::Exit(ExitSucceed::Stopped.into()));
		}

		Ok(())
	}
}
