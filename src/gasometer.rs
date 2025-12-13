//! EVM gasometer.

use evm_interpreter::uint::U256;
use evm_interpreter::{ExitError, runtime::GasState};

/// Mutable [GasState]. This simply allows recording an arbitrary gas.
pub trait GasMutState: GasState {
	/// Record an arbitrary gas into the current gasometer.
	fn record_gas(&mut self, gas: U256) -> Result<(), ExitError>;
}
