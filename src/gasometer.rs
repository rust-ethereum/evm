//! EVM gasometer.

use crate::{ExitError, Machine};
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

pub trait StaticGasometer: Sized {
	fn record_cost(&mut self, cost: U256) -> Result<(), ExitError>;
	fn record_codedeposit(&mut self, len: usize) -> Result<(), ExitError>;
	fn gas(&self) -> U256;
}

pub trait Gasometer<S, H>: StaticGasometer {
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
}
