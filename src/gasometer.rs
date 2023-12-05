//! EVM gasometer.

use crate::{ExitError, GasState};
use primitive_types::U256;

/// A static gasometer, exposing functions for precompile cost recording or for
/// transactions.
pub trait GasometerState: GasState {
	fn record_cost(&mut self, cost: U256) -> Result<(), ExitError>;
	fn record_codedeposit(&mut self, len: usize) -> Result<(), ExitError>;
}
