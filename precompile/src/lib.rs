//! Standard EVM precompiles.

// #![deny(warnings)]
// #![forbid(unsafe_code, unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

macro_rules! try_some {
	($e:expr) => {
		match $e {
			Ok(v) => v,
			Err(err) => return (Err(err.into()), Vec::new()),
		}
	};
}

extern crate alloc;

mod blake2;
mod bn128;
mod modexp;
mod simple;

use alloc::vec::Vec;

use evm::{
	GasMutState,
	interpreter::{ExitError, ExitException, ExitResult, runtime::RuntimeState},
	standard::{Config, PrecompileSet},
};
use primitive_types::H160;

pub use crate::{
	blake2::Blake2F,
	bn128::{Bn128AddIstanbul, Bn128MulIstanbul, Bn128PairingIstanbul},
	modexp::Modexp,
	simple::{ECRecover, Identity, Ripemd160, Sha256},
};

pub trait PurePrecompile<G> {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>);
}

pub struct StandardPrecompileSet;

impl<G: AsRef<RuntimeState> + AsRef<Config> + GasMutState, H> PrecompileSet<G, H>
	for StandardPrecompileSet
{
	fn execute(
		&self,
		code_address: H160,
		input: &[u8],
		gasometer: &mut G,
		_handler: &mut H,
	) -> Option<(ExitResult, Vec<u8>)> {
		// TODO: selectively disable precompiles based on config.

		if code_address == address(1) {
			Some(ECRecover.execute(input, gasometer))
		} else if code_address == address(2) {
			Some(Sha256.execute(input, gasometer))
		} else if code_address == address(3) {
			Some(Ripemd160.execute(input, gasometer))
		} else if code_address == address(4) {
			Some(Identity.execute(input, gasometer))
		} else if code_address == address(5) {
			Some(Modexp.execute(input, gasometer))
		} else if code_address == address(6) {
			Some(Bn128AddIstanbul.execute(input, gasometer))
		} else if code_address == address(7) {
			Some(Bn128MulIstanbul.execute(input, gasometer))
		} else if code_address == address(8) {
			Some(Bn128PairingIstanbul.execute(input, gasometer))
		} else if code_address == address(9) {
			Some(Blake2F.execute(input, gasometer))
		} else {
			None
		}
	}
}

fn linear_cost(len: u64, base: u64, word: u64) -> Result<u64, ExitError> {
	let cost = base
		.checked_add(
			word.checked_mul(len.saturating_add(31) / 32)
				.ok_or(ExitException::OutOfGas)?,
		)
		.ok_or(ExitException::OutOfGas)?;

	Ok(cost)
}

const fn address(last: u8) -> H160 {
	H160([
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, last,
	])
}
