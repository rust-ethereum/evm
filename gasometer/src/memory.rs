use evm_core::ExitError;
use crate::consts::*;

pub fn memory_gas(a: usize) -> Result<usize, ExitError> {
	G_MEMORY
		.checked_mul(a).ok_or(ExitError::OutOfGas)?
		.checked_add(
			a.checked_mul(a).ok_or(ExitError::OutOfGas)? / 512
		).ok_or(ExitError::OutOfGas)
}
