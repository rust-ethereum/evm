use alloc::{borrow::Cow, vec, vec::Vec};
use core::cmp::{max, min};
use evm::uint::U256;
use evm::{
	GasMutState,
	interpreter::{ExitException, ExitResult, ExitSucceed},
};

use crate::PurePrecompile;

fn modexp(base: &[u8], exponent: &[u8], modulus: &[u8]) -> Vec<u8> {
	aurora_engine_modexp::modexp(base, exponent, modulus)
}

/// Calculate the iteration count for the modexp precompile.
fn calculate_iteration_count<const MULTIPLIER: u64>(exp_length: u64, exp_highp: &U256) -> u64 {
	let mut iteration_count: u64 = 0;

	if exp_length <= 32 && exp_highp.is_zero() {
		iteration_count = 0;
	} else if exp_length <= 32 {
		iteration_count = exp_highp.bits() as u64 - 1;
	} else if exp_length > 32 {
		iteration_count = (MULTIPLIER.saturating_mul(exp_length - 32))
			.saturating_add(max(1, exp_highp.bits() as u64) - 1);
	}

	max(iteration_count, 1)
}

/// Calculate the gas cost for the modexp precompile with BYZANTIUM gas rules.
pub fn byzantium_gas_calc(base_len: u64, exp_len: u64, mod_len: u64, exp_highp: &U256) -> u64 {
	gas_calc::<0, 8, 20, _>(base_len, exp_len, mod_len, exp_highp, |max_len| -> U256 {
		// Output of this function is bounded by 2^128
		if max_len <= 64 {
			U256::from(max_len * max_len)
		} else if max_len <= 1_024 {
			U256::from(max_len * max_len / 4 + 96 * max_len - 3_072)
		} else {
			// Up-cast to avoid overflow
			let x = U256::from(max_len);
			let x_sq = x * x; // x < 2^64 => x*x < 2^128 < 2^256 (no overflow)
			x_sq / U256::from(16) + U256::from(480) * x - U256::from(199_680)
		}
	})
}

/// Calculate gas cost according to EIP 2565:
/// <https://eips.ethereum.org/EIPS/eip-2565>
fn berlin_gas_calc(base_len: u64, exp_len: u64, mod_len: u64, exp_highp: &U256) -> u64 {
	gas_calc::<200, 8, 3, _>(base_len, exp_len, mod_len, exp_highp, |max_len| -> U256 {
		let words = U256::from(max_len.div_ceil(8));
		words * words
	})
}

/// Calculate gas cost.
fn gas_calc<const MIN_PRICE: u64, const MULTIPLIER: u64, const GAS_DIVISOR: u64, F>(
	base_len: u64,
	exp_len: u64,
	mod_len: u64,
	exp_highp: &U256,
	calculate_multiplication_complexity: F,
) -> u64
where
	F: Fn(u64) -> U256,
{
	let multiplication_complexity = calculate_multiplication_complexity(max(base_len, mod_len));
	let iteration_count = calculate_iteration_count::<MULTIPLIER>(exp_len, exp_highp);
	let gas = (multiplication_complexity * U256::from(iteration_count)) / U256::from(GAS_DIVISOR);

	if gas > U256::from(u64::MAX) {
		u64::MAX
	} else {
		max(MIN_PRICE, gas.as_u64())
	}
}

/// Right-pads the given slice at `offset` with zeroes until `LEN`.
///
/// Returns the first `LEN` bytes if it does not need padding.
#[inline]
pub fn right_pad_with_offset<const LEN: usize>(data: &[u8], offset: usize) -> Cow<'_, [u8; LEN]> {
	right_pad(data.get(offset..).unwrap_or_default())
}

/// Right-pads the given slice with zeroes until `LEN`.
///
/// Returns the first `LEN` bytes if it does not need padding.
#[inline]
pub fn right_pad<const LEN: usize>(data: &[u8]) -> Cow<'_, [u8; LEN]> {
	if let Some(data) = data.get(..LEN) {
		Cow::Borrowed(data.try_into().unwrap())
	} else {
		let mut padded = [0; LEN];
		padded[..data.len()].copy_from_slice(data);
		Cow::Owned(padded)
	}
}

/// Right-pads the given slice with zeroes until `len`.
///
/// Returns the first `len` bytes if it does not need padding.
#[inline]
pub fn right_pad_vec(data: &[u8], len: usize) -> Cow<'_, [u8]> {
	if let Some(data) = data.get(..len) {
		Cow::Borrowed(data)
	} else {
		let mut padded = vec![0; len];
		padded[..data.len()].copy_from_slice(data);
		Cow::Owned(padded)
	}
}

/// Left-pads the given slice with zeroes until `LEN`.
///
/// Returns the first `LEN` bytes if it does not need padding.
#[inline]
pub fn left_pad<const LEN: usize>(data: &[u8]) -> Cow<'_, [u8; LEN]> {
	if let Some(data) = data.get(..LEN) {
		Cow::Borrowed(data.try_into().unwrap())
	} else {
		let mut padded = [0; LEN];
		padded[LEN - data.len()..].copy_from_slice(data);
		Cow::Owned(padded)
	}
}

/// Left-pads the given slice with zeroes until `len`.
///
/// Returns the first `len` bytes if it does not need padding.
#[inline]
pub fn left_pad_vec(data: &[u8], len: usize) -> Cow<'_, [u8]> {
	if let Some(data) = data.get(..len) {
		Cow::Borrowed(data)
	} else {
		let mut padded = vec![0; len];
		padded[len - data.len()..].copy_from_slice(data);
		Cow::Owned(padded)
	}
}

fn execute<G: GasMutState>(
	input: &[u8],
	gasometer: &mut G,
	gas_calc: fn(u64, u64, u64, &U256) -> u64,
) -> (ExitResult, Vec<u8>) {
	// The format of input is:
	// <length_of_BASE> <length_of_EXPONENT> <length_of_MODULUS> <BASE> <EXPONENT> <MODULUS>
	// Where every length is a 32-byte left-padded integer representing the number of bytes
	// to be taken up by the next value.
	const HEADER_LENGTH: usize = 96;

	// Extract the header
	let base_len = U256::from_big_endian(&right_pad_with_offset::<32>(input, 0).into_owned());
	let exp_len = U256::from_big_endian(&right_pad_with_offset::<32>(input, 32).into_owned());
	let mod_len = U256::from_big_endian(&right_pad_with_offset::<32>(input, 64).into_owned());

	// Cast base and modulus to usize, it does not make sense to handle larger values
	let base_len = try_some!(usize::try_from(base_len).map_err(|_| ExitException::OutOfGas));
	let mod_len = try_some!(usize::try_from(mod_len).map_err(|_| ExitException::OutOfGas));
	// cast exp len to the max size, it will fail later in gas calculation if it is too large.
	let exp_len = usize::try_from(exp_len).unwrap_or(usize::MAX);

	// Used to extract ADJUSTED_EXPONENT_LENGTH.
	let exp_highp_len = min(exp_len, 32);

	// Throw away the header data as we already extracted lengths.
	let input = input.get(HEADER_LENGTH..).unwrap_or_default();

	let exp_highp = {
		// Get right padded bytes so if data.len is less then exp_len we will get right padded zeroes.
		let right_padded_highp = right_pad_with_offset::<32>(input, base_len);
		// If exp_len is less then 32 bytes get only exp_len bytes and do left padding.
		let out = left_pad::<32>(&right_padded_highp[..exp_highp_len]);
		U256::from_big_endian(&out.into_owned())
	};

	// Check if we have enough gas.
	let gas_cost = gas_calc(base_len as u64, exp_len as u64, mod_len as u64, &exp_highp);
	try_some!(gasometer.record_gas(U256::from(gas_cost)));

	if base_len == 0 && mod_len == 0 {
		return (ExitSucceed::Returned.into(), Vec::new());
	}

	// Padding is needed if the input does not contain all 3 values.
	let input_len = base_len.saturating_add(exp_len).saturating_add(mod_len);
	let input = right_pad_vec(input, input_len);
	let (base, input) = input.split_at(base_len);
	let (exponent, modulus) = input.split_at(exp_len);
	debug_assert_eq!(modulus.len(), mod_len);

	// Call the modexp.
	let output = modexp(base, exponent, modulus);

	(
		ExitSucceed::Returned.into(),
		left_pad_vec(&output, mod_len).into_owned(),
	)
}

pub struct ModexpByzantium;

impl<G: GasMutState> PurePrecompile<G> for ModexpByzantium {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		execute(input, gasometer, byzantium_gas_calc)
	}
}

pub struct ModexpBerlin;

impl<G: GasMutState> PurePrecompile<G> for ModexpBerlin {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		execute(input, gasometer, berlin_gas_calc)
	}
}
