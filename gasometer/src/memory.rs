use crate::consts;
use evm_core::ExitError;

pub fn memory_gas(a: usize) -> Result<u64, ExitError> {
	// NOTE: in that context usize->u64 `as_conversions` is save
	#[allow(clippy::as_conversions)]
	let a = a as u64;
	u64::from(consts::G_MEMORY)
		.checked_mul(a)
		.ok_or(ExitError::OutOfGas)?
		.checked_add(a.checked_mul(a).ok_or(ExitError::OutOfGas)? / 512)
		.ok_or(ExitError::OutOfGas)
}
