//! EVM gasometer.

use crate::{
	Capture, Control, Etable, ExitError, ExitResult, Machine, MergeStrategy, Opcode, RuntimeState,
};
use core::ops::{Add, AddAssign, Sub, SubAssign};
use primitive_types::U256;

pub trait Gas:
	Copy
	+ Into<U256>
	+ Add<Self, Output = Self>
	+ AddAssign<Self>
	+ Sub<Self, Output = Self>
	+ SubAssign<Self>
{
}

impl Gas for u64 {}
impl Gas for U256 {}

pub trait Gasometer<S, H>: Sized {
	fn record_step(
		&mut self,
		machine: &Machine<S>,
		is_static: bool,
		backend: &H,
	) -> Result<(), ExitError>;
	fn record_stepn(
		&mut self,
		machine: &Machine<S>,
		is_static: bool,
		backend: &H,
	) -> Result<usize, ExitError> {
		self.record_step(machine, is_static, backend)?;
		Ok(1)
	}
	fn record_codedeposit(&mut self, len: usize) -> Result<(), ExitError>;
	fn gas(&self) -> U256;
	fn submeter(
		&mut self,
		gas_limit: U256,
		call_has_value: bool,
		code: &[u8],
	) -> Result<Self, ExitError>;
	fn merge(&mut self, other: Self, strategy: MergeStrategy);
}

pub struct GasedMachine<S, G> {
	pub machine: Machine<S>,
	pub gasometer: G,
	pub is_static: bool,
}

impl<S: AsMut<RuntimeState>, G> GasedMachine<S, G> {
	pub fn step<H, Tr, F>(
		&mut self,
		handler: &mut H,
		etable: &Etable<S, H, Tr, F>,
	) -> Result<(), Capture<ExitResult, Tr>>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
		G: Gasometer<S, H>,
	{
		match self
			.gasometer
			.record_step(&self.machine, self.is_static, handler)
		{
			Ok(()) => {
				self.machine.state.as_mut().gas = self.gasometer.gas().into();
				self.machine.step(handler, etable)
			}
			Err(e) => return Err(Capture::Exit(Err(e))),
		}
	}

	pub fn run<H, Tr, F>(
		&mut self,
		handler: &mut H,
		etable: &Etable<S, H, Tr, F>,
	) -> Capture<ExitResult, Tr>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
		G: Gasometer<S, H>,
	{
		loop {
			match self
				.gasometer
				.record_stepn(&self.machine, self.is_static, handler)
			{
				Ok(stepn) => {
					self.machine.state.as_mut().gas = self.gasometer.gas().into();
					match self.machine.stepn(stepn, handler, etable) {
						Ok(()) => (),
						Err(c) => return c,
					}
				}
				Err(e) => return Capture::Exit(Err(e)),
			}
		}
	}
}
