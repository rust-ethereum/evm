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

pub use crate::blake2::Blake2F;
pub use crate::bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
pub use crate::modexp::Modexp;
pub use crate::simple::{ECRecover, Identity, Ripemd160, Sha256};

use alloc::vec::Vec;
use evm::standard::{CodeResolver, Config, Precompile, ResolvedCode};
use evm::{ExitError, ExitException, ExitResult, RuntimeBackend, RuntimeState, StaticGasometer};

use primitive_types::H160;

pub trait PurePrecompile<G> {
	fn execute(
		&self,
		input: &[u8],
		state: &RuntimeState,
		gasometer: &mut G,
	) -> (ExitResult, Vec<u8>);
}

pub enum StandardPrecompile {
	ECRecover,
	Sha256,
	Ripemd160,
	Identity,
	Modexp,
	Bn128Add,
	Bn128Mul,
	Bn128Pairing,
	Blake2F,
}

impl<S: AsRef<RuntimeState>, G: StaticGasometer, H> Precompile<S, G, H> for StandardPrecompile {
	fn execute(
		&self,
		input: &[u8],
		state: &mut S,
		gasometer: &mut G,
		_handler: &mut H,
	) -> (ExitResult, Vec<u8>) {
		match self {
			Self::ECRecover => ECRecover.execute(input, state.as_ref(), gasometer),
			Self::Sha256 => Sha256.execute(input, state.as_ref(), gasometer),
			Self::Ripemd160 => Ripemd160.execute(input, state.as_ref(), gasometer),
			Self::Identity => Identity.execute(input, state.as_ref(), gasometer),
			Self::Modexp => Modexp.execute(input, state.as_ref(), gasometer),
			Self::Bn128Add => Bn128Add.execute(input, state.as_ref(), gasometer),
			Self::Bn128Mul => Bn128Mul.execute(input, state.as_ref(), gasometer),
			Self::Bn128Pairing => Bn128Pairing.execute(input, state.as_ref(), gasometer),
			Self::Blake2F => Blake2F.execute(input, state.as_ref(), gasometer),
		}
	}
}

pub struct StandardResolver<'config> {
	_config: &'config Config,
}

impl<'config> StandardResolver<'config> {
	pub fn new(config: &'config Config) -> Self {
		Self { _config: config }
	}
}

impl<'config, S: AsRef<RuntimeState>, G: StaticGasometer, H: RuntimeBackend> CodeResolver<S, G, H>
	for StandardResolver<'config>
{
	type Precompile = StandardPrecompile;

	fn resolve(
		&self,
		addr: H160,
		_gasometer: &mut G,
		handler: &mut H,
	) -> Result<ResolvedCode<Self::Precompile>, ExitError> {
		// TODO: selectively disable precompiles based on config.

		Ok(if addr == address(1) {
			ResolvedCode::Precompile(StandardPrecompile::ECRecover)
		} else if addr == address(2) {
			ResolvedCode::Precompile(StandardPrecompile::Sha256)
		} else if addr == address(3) {
			ResolvedCode::Precompile(StandardPrecompile::Ripemd160)
		} else if addr == address(4) {
			ResolvedCode::Precompile(StandardPrecompile::Identity)
		} else if addr == address(5) {
			ResolvedCode::Precompile(StandardPrecompile::Modexp)
		} else if addr == address(6) {
			ResolvedCode::Precompile(StandardPrecompile::Bn128Add)
		} else if addr == address(7) {
			ResolvedCode::Precompile(StandardPrecompile::Bn128Mul)
		} else if addr == address(8) {
			ResolvedCode::Precompile(StandardPrecompile::Bn128Pairing)
		} else if addr == address(9) {
			ResolvedCode::Precompile(StandardPrecompile::Blake2F)
		} else {
			ResolvedCode::Normal(handler.code(addr))
		})
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
