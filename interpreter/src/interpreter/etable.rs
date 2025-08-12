use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use crate::{
	etable::Etable,
	runtime::RuntimeState,
	trap::{CallFeedback, CreateFeedback},
	Capture, Control, ExitError, ExitException, ExitResult, ExitSucceed, FeedbackInterpreter,
	Interpreter, Machine, Opcode, Stack, StepInterpreter, Valids,
};

pub struct EtableInterpreter<'etable, S, ES> {
	valids: Valids,
	position: usize,
	machine: Machine<S>,
	etable: &'etable ES,
}

impl<'etable, S, ES> AsRef<Machine<S>> for EtableInterpreter<'etable, S, ES> {
	fn as_ref(&self) -> &Machine<S> {
		&self.machine
	}
}

impl<'etable, S, ES> AsMut<Machine<S>> for EtableInterpreter<'etable, S, ES> {
	fn as_mut(&mut self) -> &mut Machine<S> {
		&mut self.machine
	}
}

impl<'etable, S, ES> Deref for EtableInterpreter<'etable, S, ES> {
	type Target = Machine<S>;

	fn deref(&self) -> &Machine<S> {
		&self.machine
	}
}

impl<'etable, S, ES> DerefMut for EtableInterpreter<'etable, S, ES> {
	fn deref_mut(&mut self) -> &mut Machine<S> {
		&mut self.machine
	}
}

impl<'etable, S, ES> EtableInterpreter<'etable, S, ES> {
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

	pub fn advance(&mut self) {
		if self.position == self.code.len() {
			return;
		}

		self.position += 1;
	}
}

impl<'etable, S, H, ES: Etable<H, State = S>> Interpreter<H>
	for EtableInterpreter<'etable, S, ES>
{
	type State = S;
	type Trap = ES::Trap;

	fn deconstruct(self) -> (ES::State, Vec<u8>) {
		(self.machine.state, self.machine.retval)
	}

	fn state(&self) -> &Self::State {
		&self.machine.state
	}

	fn state_mut(&mut self) -> &mut Self::State {
		&mut self.machine.state
	}

	fn run(&mut self, handle: &mut H) -> Capture<ExitResult, Self::Trap> {
		loop {
			match self.step(handle) {
				Ok(()) => (),
				Err(res) => return res,
			}
		}
	}
}

impl<'etable, S, H, ES: Etable<H, State = S>> FeedbackInterpreter<H, CallFeedback>
	for EtableInterpreter<'etable, S, ES>
where
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
{
	fn feedback(&mut self, feedback: CallFeedback, _handler: &mut H) -> Result<(), ExitError> {
		match feedback.to_machine(self) {
			Ok(()) => {
				self.advance();
				Ok(())
			}
			Err(err) => Err(err),
		}
	}
}

impl<'etable, S, H, ES: Etable<H, State = S>> FeedbackInterpreter<H, CreateFeedback>
	for EtableInterpreter<'etable, S, ES>
where
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
{
	fn feedback(&mut self, feedback: CreateFeedback, _handler: &mut H) -> Result<(), ExitError> {
		match feedback.to_machine(self) {
			Ok(()) => {
				self.advance();
				Ok(())
			}
			Err(err) => Err(err),
		}
	}
}

impl<'etable, S, H, ES: Etable<H, State = S>> StepInterpreter<H>
	for EtableInterpreter<'etable, S, ES>
{
	#[inline]
	fn step(&mut self, handle: &mut H) -> Result<(), Capture<ExitResult, ES::Trap>> {
		let position = self.position;
		if position >= self.code.len() {
			return Err(Capture::Exit(ExitSucceed::Stopped.into()));
		}

		let control = self.etable.eval(&mut self.machine, handle, self.position);

		match control {
			Control::NoAction => (),
			Control::Continue(p) => {
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

		Ok(())
	}
}
