use alloc::{vec, vec::Vec};
use core::cmp::max;

use evm::{
	interpreter::error::{ExitException, ExitResult, ExitSucceed},
	GasMutState,
};
use num::{BigUint, FromPrimitive, Integer, One, ToPrimitive, Zero};

use crate::PurePrecompile;

pub struct Modexp;

const MIN_GAS_COST: u64 = 200;

// Calculate gas cost according to EIP 2565:
// https://eips.ethereum.org/EIPS/eip-2565
fn calculate_gas_cost(
	base_length: u64,
	mod_length: u64,
	exponent: &BigUint,
	exponent_bytes: &[u8],
	mod_is_even: bool,
) -> u64 {
	fn calculate_multiplication_complexity(base_length: u64, mod_length: u64) -> u64 {
		let max_length = max(base_length, mod_length);
		let mut words = max_length / 8;
		if max_length % 8 > 0 {
			words += 1;
		}

		// Note: can't overflow because we take words to be some u64 value / 8, which is
		// necessarily less than sqrt(u64::MAX).
		// Additionally, both base_length and mod_length are bounded to 1024, so this has
		// an upper bound of roughly (1024 / 8) squared
		words * words
	}

	fn calculate_iteration_count(exponent: &BigUint, exponent_bytes: &[u8]) -> u64 {
		let mut iteration_count: u64 = 0;
		let exp_length = exponent_bytes.len() as u64;

		if exp_length <= 32 && exponent.is_zero() {
			iteration_count = 0;
		} else if exp_length <= 32 {
			iteration_count = exponent.bits() - 1;
		} else if exp_length > 32 {
			// from the EIP spec:
			// (8 * (exp_length - 32)) + ((exponent & (2**256 - 1)).bit_length() - 1)
			//
			// Notes:
			// * exp_length is bounded to 1024 and is > 32
			// * exponent can be zero, so we subtract 1 after adding the other terms (whose sum
			//   must be > 0)
			// * the addition can't overflow because the terms are both capped at roughly
			//   8 * max size of exp_length (1024)
			// * the EIP spec is written in python, in which (exponent & (2**256 - 1)) takes the
			//   FIRST 32 bytes. However this `BigUint` `&` operator takes the LAST 32 bytes.
			//   We thus instead take the bytes manually.
			let exponent_head = BigUint::from_bytes_be(&exponent_bytes[..32]);

			iteration_count = (8 * (exp_length - 32)) + exponent_head.bits() - 1;
		}

		max(iteration_count, 1)
	}

	let multiplication_complexity = calculate_multiplication_complexity(base_length, mod_length);
	let iteration_count = calculate_iteration_count(exponent, exponent_bytes);
	max(
		MIN_GAS_COST,
		multiplication_complexity * iteration_count / 3,
	)
	.saturating_mul(if mod_is_even { 20 } else { 1 })
}

/// Copy bytes from input to target.
fn read_input(source: &[u8], target: &mut [u8], source_offset: &mut usize) {
	// We move the offset by the len of the target, regardless of what we
	// actually copy.
	let offset = *source_offset;
	*source_offset += target.len();

	// Out of bounds, nothing to copy.
	if source.len() <= offset {
		return;
	}

	// Find len to copy up to target len, but not out of bounds.
	let len = core::cmp::min(target.len(), source.len() - offset);
	target[..len].copy_from_slice(&source[offset..][..len]);
}

impl<G: GasMutState> PurePrecompile<G> for Modexp {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		let mut input_offset = 0;

		// Yellowpaper: whenever the input is too short, the missing bytes are
		// considered to be zero.
		let mut base_len_buf = [0u8; 32];
		read_input(input, &mut base_len_buf, &mut input_offset);
		let mut exp_len_buf = [0u8; 32];
		read_input(input, &mut exp_len_buf, &mut input_offset);
		let mut mod_len_buf = [0u8; 32];
		read_input(input, &mut mod_len_buf, &mut input_offset);

		// reasonable assumption: this must fit within the Ethereum EVM's max stack size
		let max_size_big = BigUint::from_u32(1024).expect("can't create BigUint");

		let base_len_big = BigUint::from_bytes_be(&base_len_buf);
		if base_len_big > max_size_big {
			try_some!(Err(ExitException::Other(
				"unreasonably large base length".into()
			)));
		}

		let exp_len_big = BigUint::from_bytes_be(&exp_len_buf);
		if exp_len_big > max_size_big {
			try_some!(Err(ExitException::Other(
				"unreasonably large exponent length".into()
			)));
		}

		let mod_len_big = BigUint::from_bytes_be(&mod_len_buf);
		if mod_len_big > max_size_big {
			try_some!(Err(ExitException::Other(
				"unreasonably large modulus length".into()
			)));
		}

		// bounds check handled above
		let base_len = base_len_big.to_usize().expect("base_len out of bounds");
		let exp_len = exp_len_big.to_usize().expect("exp_len out of bounds");
		let mod_len = mod_len_big.to_usize().expect("mod_len out of bounds");

		// if mod_len is 0 output must be empty
		if mod_len == 0 {
			return (ExitSucceed::Returned.into(), Vec::new());
		}

		// Gas formula allows arbitrary large exp_len when base and modulus are empty, so we need to handle empty base first.
		let r = if base_len == 0 && mod_len == 0 {
			try_some!(gasometer.record_gas(MIN_GAS_COST.into()));
			BigUint::zero()
		} else {
			// read the numbers themselves.
			let mut base_buf = vec![0u8; base_len];
			read_input(input, &mut base_buf, &mut input_offset);
			let base = BigUint::from_bytes_be(&base_buf);

			let mut exp_buf = vec![0u8; exp_len];
			read_input(input, &mut exp_buf, &mut input_offset);
			let exponent = BigUint::from_bytes_be(&exp_buf);

			let mut mod_buf = vec![0u8; mod_len];
			read_input(input, &mut mod_buf, &mut input_offset);
			let modulus = BigUint::from_bytes_be(&mod_buf);

			// do our gas accounting
			let gas_cost = calculate_gas_cost(
				base_len as u64,
				mod_len as u64,
				&exponent,
				&exp_buf,
				modulus.is_even(),
			);

			try_some!(gasometer.record_gas(gas_cost.into()));

			if modulus.is_zero() || modulus.is_one() {
				BigUint::zero()
			} else {
				base.modpow(&exponent, &modulus)
			}
		};

		// write output to given memory, left padded and same length as the modulus.
		let bytes = r.to_bytes_be();

		// always true except in the case of zero-length modulus, which leads to
		// output of length and value 1.
		#[allow(clippy::comparison_chain)]
		if bytes.len() == mod_len {
			(ExitSucceed::Returned.into(), bytes.to_vec())
		} else if bytes.len() < mod_len {
			let mut ret = Vec::with_capacity(mod_len);
			ret.extend(core::iter::repeat(0).take(mod_len - bytes.len()));
			ret.extend_from_slice(&bytes[..]);
			(ExitSucceed::Returned.into(), ret.to_vec())
		} else {
			return (ExitException::Other("failed".into()).into(), Vec::new());
		}
	}
}
