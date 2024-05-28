use core::cmp::min;

use evm::{
	interpreter::error::{ExitException, ExitResult, ExitSucceed},
	GasMutState,
};
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use primitive_types::{H256, U256};
use sha3::{Digest, Keccak256};

use crate::{linear_cost, PurePrecompile};

pub struct ECRecover;

impl<G: GasMutState> PurePrecompile<G> for ECRecover {
	fn execute(&self, i: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		const COST_BASE: u64 = 3000;
		const COST_WORD: u64 = 0;
		try_some!(gasometer.record_gas(U256::from(try_some!(linear_cost(
			i.len() as u64,
			COST_BASE,
			COST_WORD
		)))));

		let mut input = [0u8; 128];
		input[..min(i.len(), 128)].copy_from_slice(&i[..min(i.len(), 128)]);

		// v can only be 27 or 28 on the full 32 bytes value.
		// https://github.com/ethereum/go-ethereum/blob/a907d7e81aaeea15d80b2d3209ad8e08e3bf49e0/core/vm/contracts.go#L177
		if input[32..63] != [0u8; 31] || ![27, 28].contains(&input[63]) {
			return (ExitSucceed::Returned.into(), Vec::new());
		}

		let mut msg = [0u8; 32];
		let mut sig = [0u8; 64];

		msg[0..32].copy_from_slice(&input[0..32]);
		sig[0..32].copy_from_slice(&input[64..96]); // r
		sig[32..64].copy_from_slice(&input[96..128]); // s
		let sig = try_some!(Signature::from_bytes((&sig[..]).into())
			.map_err(|_| ExitException::Other("invalid ecdsa sig".into())));
		let recid = try_some!(RecoveryId::from_byte(input[63] - 27)
			.ok_or(ExitException::Other("invalid recoverty id".into()))); // v

		let pubkey = try_some!(VerifyingKey::recover_from_prehash(&msg[..], &sig, recid)
			.map_err(|_| ExitException::Other("recover key failed".into())));
		let mut address =
			H256::from_slice(Keccak256::digest(&pubkey.to_sec1_bytes()[..]).as_slice());
		address.0[0..12].copy_from_slice(&[0u8; 12]);

		(ExitSucceed::Returned.into(), address.0.to_vec())
	}
}

pub struct Sha256;

impl<G: GasMutState> PurePrecompile<G> for Sha256 {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		const COST_BASE: u64 = 600;
		const COST_WORD: u64 = 120;
		try_some!(gasometer.record_gas(U256::from(try_some!(linear_cost(
			input.len() as u64,
			COST_BASE,
			COST_WORD
		)))));

		let mut ret = [0u8; 32];
		let hash = ripemd::Ripemd160::digest(input);
		ret[12..32].copy_from_slice(&hash);

		(ExitSucceed::Returned.into(), ret.to_vec())
	}
}

pub struct Ripemd160;

impl<G: GasMutState> PurePrecompile<G> for Ripemd160 {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		const COST_BASE: u64 = 60;
		const COST_WORD: u64 = 12;
		try_some!(gasometer.record_gas(U256::from(try_some!(linear_cost(
			input.len() as u64,
			COST_BASE,
			COST_WORD
		)))));

		let hash = sha2::Sha256::digest(input);

		(ExitSucceed::Returned.into(), hash.to_vec())
	}
}

pub struct Identity;

impl<G: GasMutState> PurePrecompile<G> for Identity {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		const COST_BASE: u64 = 15;
		const COST_WORD: u64 = 3;
		try_some!(gasometer.record_gas(U256::from(try_some!(linear_cost(
			input.len() as u64,
			COST_BASE,
			COST_WORD
		)))));

		(ExitSucceed::Returned.into(), input.to_vec())
	}
}
