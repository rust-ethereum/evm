//! EVM gasometer.

use core::ops::{Add, AddAssign, Sub, SubAssign};
use evm_interpreter::{Capture, Control, Etable, ExitError, ExitResult, Machine, Opcode};
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

#[derive(Clone, Copy)]
pub enum GasometerMergeStrategy {
	Commit,
	Revert,
}

pub trait Gasometer<S, H>: Sized {
	type Gas: Gas;
	type Config;

	fn new(gas_limit: Self::Gas, machine: &Machine<S>, config: Self::Config) -> Self;
	fn record_stepn(
		&mut self,
		machine: &Machine<S>,
		backend: &H,
		is_static: bool,
	) -> Result<usize, ExitError>;
	fn record_codedeposit(&mut self, len: usize) -> Result<(), ExitError>;
	fn gas(&self) -> Self::Gas;
	fn merge(&mut self, other: Self, strategy: GasometerMergeStrategy);
}

pub fn run_with_gasometer<S, G, H, Tr, F>(
	machine: &mut Machine<S>,
	gasometer: &mut G,
	backend: &mut H,
	is_static: bool,
	etable: &Etable<S, (&mut G, &mut H), Tr, F>,
) -> Capture<ExitResult, Tr>
where
	G: Gasometer<S, H>,
	F: Fn(&mut Machine<S>, &mut (&mut G, &mut H), Opcode, usize) -> Control<Tr>,
{
	let mut handler = (gasometer, backend);

	loop {
		match handler.0.record_stepn(&machine, handler.1, is_static) {
			Ok(stepn) => match machine.stepn(stepn, &mut handler, etable) {
				Ok(()) => (),
				Err(c) => return c,
			},
			Err(e) => return Capture::Exit(Err(e)),
		}
	}
}
