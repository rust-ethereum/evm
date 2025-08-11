//! EVM gasometer.

use evm_interpreter::{runtime::GasState, ExitError};
use primitive_types::U256;

pub trait GasMutState: GasState {
	fn record_gas(&mut self, gas: U256) -> Result<(), ExitError>;
}
