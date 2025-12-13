//! Standard EVM precompiles.

#![deny(warnings)]
#![forbid(unsafe_code, unused_variables)]
#![warn(missing_docs)]
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
mod kzg;
mod modexp;
mod simple;

use alloc::vec::Vec;

use crate::{
	blake2::Blake2F,
	bn128::{
		Bn128AddByzantium, Bn128AddIstanbul, Bn128MulByzantium, Bn128MulIstanbul,
		Bn128PairingByzantium, Bn128PairingIstanbul,
	},
	kzg::KZGPointEvaluation,
	modexp::{ModexpBerlin, ModexpByzantium},
	simple::{ECRecover, Identity, Ripemd160, Sha256},
};
use evm::uint::H160;
use evm::{
	GasMutState,
	interpreter::{
		ExitError, ExitException, ExitResult,
		runtime::{RuntimeBackend, RuntimeState, TouchKind},
	},
	standard::{Config, PrecompileSet},
};

trait PurePrecompile<G> {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>);
}

/// The standard precompile set on Ethereum mainnet.
pub struct StandardPrecompileSet;

impl<G: AsRef<RuntimeState> + AsRef<Config> + GasMutState, H: RuntimeBackend> PrecompileSet<G, H>
	for StandardPrecompileSet
{
	fn on_transaction(&self, state: &mut G, handler: &mut H) {
		handler.mark_hot(address(1), TouchKind::Access);
		handler.mark_hot(address(2), TouchKind::Access);
		handler.mark_hot(address(3), TouchKind::Access);
		handler.mark_hot(address(4), TouchKind::Access);

		if AsRef::<Config>::as_ref(state).eip198_modexp_precompile {
			handler.mark_hot(address(5), TouchKind::Access);
		}

		if AsRef::<Config>::as_ref(state).eip196_ec_add_mul_precompile {
			handler.mark_hot(address(6), TouchKind::Access);
			handler.mark_hot(address(7), TouchKind::Access);
		}

		if AsRef::<Config>::as_ref(state).eip197_ec_pairing_precompile {
			handler.mark_hot(address(8), TouchKind::Access);
		}

		if AsRef::<Config>::as_ref(state).eip152_blake_2f_precompile {
			handler.mark_hot(address(9), TouchKind::Access);
		}

		if AsRef::<Config>::as_ref(state).eip4844_shard_blob {
			handler.mark_hot(address(10), TouchKind::Access);
		}
	}

	fn execute(
		&self,
		code_address: H160,
		input: &[u8],
		gasometer: &mut G,
		_handler: &mut H,
	) -> Option<(ExitResult, Vec<u8>)> {
		if code_address == address(1) {
			Some(ECRecover.execute(input, gasometer))
		} else if code_address == address(2) {
			Some(Sha256.execute(input, gasometer))
		} else if code_address == address(3) {
			Some(Ripemd160.execute(input, gasometer))
		} else if code_address == address(4) {
			Some(Identity.execute(input, gasometer))
		} else if AsRef::<Config>::as_ref(gasometer).eip198_modexp_precompile
			&& code_address == address(5)
		{
			if AsRef::<Config>::as_ref(gasometer).eip2565_lower_modexp {
				Some(ModexpBerlin.execute(input, gasometer))
			} else {
				Some(ModexpByzantium.execute(input, gasometer))
			}
		} else if AsRef::<Config>::as_ref(gasometer).eip196_ec_add_mul_precompile
			&& code_address == address(6)
		{
			if AsRef::<Config>::as_ref(gasometer).eip1108_ec_add_mul_pairing_decrease {
				Some(Bn128AddIstanbul.execute(input, gasometer))
			} else {
				Some(Bn128AddByzantium.execute(input, gasometer))
			}
		} else if AsRef::<Config>::as_ref(gasometer).eip196_ec_add_mul_precompile
			&& code_address == address(7)
		{
			if AsRef::<Config>::as_ref(gasometer).eip1108_ec_add_mul_pairing_decrease {
				Some(Bn128MulIstanbul.execute(input, gasometer))
			} else {
				Some(Bn128MulByzantium.execute(input, gasometer))
			}
		} else if AsRef::<Config>::as_ref(gasometer).eip197_ec_pairing_precompile
			&& code_address == address(8)
		{
			if AsRef::<Config>::as_ref(gasometer).eip1108_ec_add_mul_pairing_decrease {
				Some(Bn128PairingIstanbul.execute(input, gasometer))
			} else {
				Some(Bn128PairingByzantium.execute(input, gasometer))
			}
		} else if AsRef::<Config>::as_ref(gasometer).eip152_blake_2f_precompile
			&& code_address == address(9)
		{
			Some(Blake2F.execute(input, gasometer))
		} else if AsRef::<Config>::as_ref(gasometer).eip4844_shard_blob
			&& code_address == address(10)
		{
			Some(KZGPointEvaluation.execute(input, gasometer))
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
