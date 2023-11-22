//! Standard EVM precompiles.

// #![deny(warnings)]
// #![forbid(unsafe_code, unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

macro_rules! try_some {
	($e:expr) => {
		match $e {
			Ok(v) => v,
			Err(err) => return Some((Err(err.into()), Vec::new())),
		}
	};
}

extern crate alloc;

mod blake2;
mod bn128;
mod modexp;
mod simple;

pub use crate::blake2::Blake2F;
pub use crate::bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
pub use crate::modexp::Modexp;
pub use crate::simple::{ECRecover, Identity, Ripemd160, Sha256};

use alloc::vec::Vec;
use evm::standard::{Config, PrecompileSet};
use evm::{ExitError, ExitException, ExitResult, RuntimeState, StaticGasometer};

use primitive_types::H160;

pub trait PurePrecompileSet<G> {
	fn execute(
		&self,
		input: &[u8],
		state: &RuntimeState,
		gasometer: &mut G,
	) -> Option<(ExitResult, Vec<u8>)>;
}

pub struct StandardPrecompileSet<'config> {
	_config: &'config Config,
}

impl<'config> StandardPrecompileSet<'config> {
	pub fn new(config: &'config Config) -> Self {
		Self { _config: config }
	}
}

impl<'config, S: AsRef<RuntimeState>, G: StaticGasometer, H> PrecompileSet<S, G, H>
	for StandardPrecompileSet<'config>
{
	fn execute(
		&self,
		input: &[u8],
		state: &mut S,
		gasometer: &mut G,
		_handler: &mut H,
	) -> Option<(ExitResult, Vec<u8>)> {
		// TODO: selectively disable precompiles based on config.

		if let Some(v) = ECRecover.execute(input, state.as_ref(), gasometer) {
			return Some(v);
		}

		if let Some(v) = Sha256.execute(input, state.as_ref(), gasometer) {
			return Some(v);
		}

		if let Some(v) = Ripemd160.execute(input, state.as_ref(), gasometer) {
			return Some(v);
		}

		if let Some(v) = Identity.execute(input, state.as_ref(), gasometer) {
			return Some(v);
		}

		if let Some(v) = Modexp.execute(input, state.as_ref(), gasometer) {
			return Some(v);
		}

		if let Some(v) = Bn128Add.execute(input, state.as_ref(), gasometer) {
			return Some(v);
		}

		if let Some(v) = Bn128Mul.execute(input, state.as_ref(), gasometer) {
			return Some(v);
		}

		if let Some(v) = Bn128Pairing.execute(input, state.as_ref(), gasometer) {
			return Some(v);
		}

		if let Some(v) = Blake2F.execute(input, state.as_ref(), gasometer) {
			return Some(v);
		}

		None
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
