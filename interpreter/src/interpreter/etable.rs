use crate::{
	Capture, Control, EtableSet, ExitError, ExitException, ExitFatal, ExitResult, ExitSucceed,
	Interpreter, Machine, Opcode, Stack, StepInterpreter, Valids,
};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub struct EtableInterpreter<'etable, S, H, Tr, ES> {
	valids: Valids,
	position: usize,
	machine: Machine<S>,
	etable: &'etable ES,
	_marker: PhantomData<(H, Tr)>,
}

impl<'etable, S, H, Tr, ES> Deref for EtableInterpreter<'etable, S, H, Tr, ES> {
	type Target = Machine<S>;

	fn deref(&self) -> &Machine<S> {
		&self.machine
	}
}

impl<'etable, S, H, Tr, ES> DerefMut for EtableInterpreter<'etable, S, H, Tr, ES> {
	fn deref_mut(&mut self) -> &mut Machine<S> {
		&mut self.machine
	}
}

impl<'etable, S, H, Tr, ES> EtableInterpreter<'etable, S, H, Tr, ES>
where
	ES: EtableSet<S, H, Tr>,
{
	/// Return a reference of the program counter.
	pub const fn position(&self) -> usize {
		self.position
	}

	pub fn new(machine: Machine<S>, etable: &'etable ES) -> Self {
		let valids = Valids::new(&machine.code[..]);

		Self {
			machine,
			valids,
			position: 0,
			etable,
			_marker: PhantomData,
		}
	}

	pub fn deconstruct(self) -> Machine<S> {
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

impl<'etable, S, H, Tr, ES> Interpreter<S, H, Tr> for EtableInterpreter<'etable, S, H, Tr, ES>
where
	ES: EtableSet<S, H, Tr>,
{
	fn machine(&self) -> &Machine<S> {
		&self.machine
	}

	fn machine_mut(&mut self) -> &mut Machine<S> {
		&mut self.machine
	}

	fn deconstruct(self) -> (S, Vec<u8>) {
		(self.machine.state, self.machine.retval)
	}

	fn run(&mut self, handle: &mut H) -> Capture<ExitResult, Tr> {
		loop {
			match self.step(handle) {
				Ok(()) => (),
				Err(res) => return res,
			}
		}
	}

	fn advance(&mut self) {
		if self.position == self.code.len() {
			return;
		}

		self.position += 1;
	}
}

impl<'etable, S, H, Tr, ES> StepInterpreter<S, H, Tr> for EtableInterpreter<'etable, S, H, Tr, ES>
where
	ES: EtableSet<S, H, Tr>,
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
