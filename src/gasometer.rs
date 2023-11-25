//! EVM gasometer.

use crate::{ExitError, Machine};
use primitive_types::U256;

/// A static gasometer, exposing functions for precompile cost recording or for
/// transactions.
pub trait StaticGasometer: Sized {
	fn record_cost(&mut self, cost: U256) -> Result<(), ExitError>;
	fn record_codedeposit(&mut self, len: usize) -> Result<(), ExitError>;
	fn gas(&self) -> U256;
}

/// A gasometer that is suitable for an interpreter machine.
pub trait Gasometer<S, H>: StaticGasometer {
	/// Record gas cost for a single opcode step.
	fn record_step(
		&mut self,
		machine: &Machine<S>,
		is_static: bool,
		backend: &H,
	) -> Result<(), ExitError>;
	/// Record gas cost, advancing as much as possible (possibly into the next
	/// branch). Returns the number of advances.
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
