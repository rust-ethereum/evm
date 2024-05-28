use core::cmp::Ordering;
use core::ops::{Div, Rem};
use primitive_types::U256;

/// EIP-4844 constants
/// Gas consumption of a single data blob (== blob byte size).
pub const GAS_PER_BLOB: u64 = 1 << 17;
/// Target number of the blob per block.
pub const TARGET_BLOB_NUMBER_PER_BLOCK: u64 = 3;
/// Max number of blobs per block
pub const MAX_BLOB_NUMBER_PER_BLOCK: u64 = 2 * TARGET_BLOB_NUMBER_PER_BLOCK;
/// Maximum consumable blob gas for data blobs per block.
pub const MAX_BLOB_GAS_PER_BLOCK: u64 = MAX_BLOB_NUMBER_PER_BLOCK * GAS_PER_BLOB;
/// Target consumable blob gas for data blobs per block (for 1559-like pricing).
pub const TARGET_BLOB_GAS_PER_BLOCK: u64 = TARGET_BLOB_NUMBER_PER_BLOCK * GAS_PER_BLOB;
/// Minimum gas price for data blobs.
pub const MIN_BLOB_GASPRICE: u64 = 1;
/// Controls the maximum rate of change for blob gas price.
pub const BLOB_GASPRICE_UPDATE_FRACTION: u64 = 3338477;
/// First version of the blob.
pub const VERSIONED_HASH_VERSION_KZG: u8 = 0x01;

/// Precalculated `usize::MAX` for `U256`
pub const USIZE_MAX: U256 = U256([usize::MAX as u64, 0, 0, 0]);

/// Calculates the `excess_blob_gas` from the parent header's `blob_gas_used` and `excess_blob_gas`.
///
/// See also [the EIP-4844 helpers]<https://eips.ethereum.org/EIPS/eip-4844#helpers>
/// (`calc_excess_blob_gas`).
#[inline]
pub const fn calc_excess_blob_gas(parent_excess_blob_gas: u64, parent_blob_gas_used: u64) -> u64 {
	(parent_excess_blob_gas + parent_blob_gas_used).saturating_sub(TARGET_BLOB_GAS_PER_BLOCK)
}

/// Approximates `factor * e ** (numerator / denominator)` using Taylor expansion.
///
/// This is used to calculate the blob price.
///
/// See also [the EIP-4844 helpers](https://eips.ethereum.org/EIPS/eip-4844#helpers)
/// (`fake_exponential`).
///
/// # Panics
///
/// This function panics if `denominator` is zero.
#[inline]
pub fn fake_exponential(factor: u64, numerator: u64, denominator: u64) -> u128 {
	assert_ne!(denominator, 0, "attempt to divide by zero");
	let factor = factor as u128;
	let numerator = numerator as u128;
	let denominator = denominator as u128;

	let mut i = 1;
	let mut output = 0;
	let mut numerator_accum = factor * denominator;
	while numerator_accum > 0 {
		output += numerator_accum;

		// Denominator is asserted as not zero at the start of the function.
		numerator_accum = (numerator_accum * numerator) / (denominator * i);
		i += 1;
	}
	output / denominator
}

/// Calculates the blob gas price from the header's excess blob gas field.
///
/// See also [the EIP-4844 helpers](https://eips.ethereum.org/EIPS/eip-4844#helpers)
/// (`get_blob_gasprice`).
#[inline]
pub fn calc_blob_gasprice(excess_blob_gas: u64) -> u128 {
	fake_exponential(
		MIN_BLOB_GASPRICE,
		excess_blob_gas,
		BLOB_GASPRICE_UPDATE_FRACTION,
	)
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Sign {
	Plus,
	Minus,
	Zero,
}

const SIGN_BIT_MASK: U256 = U256([
	0xffffffffffffffff,
	0xffffffffffffffff,
	0xffffffffffffffff,
	0x7fffffffffffffff,
]);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct I256(pub Sign, pub U256);

impl I256 {
	/// Zero value of I256.
	pub const fn zero() -> Self {
		Self(Sign::Zero, U256::zero())
	}
	/// Minimum value of I256.
	pub fn min_value() -> Self {
		Self(Sign::Minus, (U256::MAX & SIGN_BIT_MASK) + U256::from(1u64))
	}
}

impl Ord for I256 {
	fn cmp(&self, other: &Self) -> Ordering {
		match (self.0, other.0) {
			(Sign::Zero, Sign::Zero) => Ordering::Equal,
			(Sign::Zero, Sign::Plus) => Ordering::Less,
			(Sign::Zero, Sign::Minus) => Ordering::Greater,
			(Sign::Minus, Sign::Zero) => Ordering::Less,
			(Sign::Minus, Sign::Plus) => Ordering::Less,
			(Sign::Minus, Sign::Minus) => self.1.cmp(&other.1).reverse(),
			(Sign::Plus, Sign::Minus) => Ordering::Greater,
			(Sign::Plus, Sign::Zero) => Ordering::Greater,
			(Sign::Plus, Sign::Plus) => self.1.cmp(&other.1),
		}
	}
}

impl PartialOrd for I256 {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Default for I256 {
	fn default() -> Self {
		Self::zero()
	}
}

impl From<U256> for I256 {
	fn from(val: U256) -> Self {
		if val == U256::zero() {
			Self::zero()
		} else if val & SIGN_BIT_MASK == val {
			Self(Sign::Plus, val)
		} else {
			Self(Sign::Minus, !val + U256::from(1u64))
		}
	}
}

impl From<I256> for U256 {
	fn from(value: I256) -> Self {
		let sign = value.0;
		if sign == Sign::Zero {
			Self::zero()
		} else if sign == Sign::Plus {
			value.1
		} else {
			!value.1 + Self::from(1u64)
		}
	}
}

impl Div for I256 {
	type Output = Self;

	fn div(self, other: Self) -> Self {
		if other == Self::zero() {
			return Self::zero();
		}

		if self == Self::min_value() && other.1 == U256::from(1u64) {
			return Self::min_value();
		}

		let d = (self.1 / other.1) & SIGN_BIT_MASK;

		if d == U256::zero() {
			return Self::zero();
		}

		match (self.0, other.0) {
			(Sign::Zero, Sign::Plus)
			| (Sign::Plus, Sign::Zero)
			| (Sign::Zero, Sign::Zero)
			| (Sign::Plus, Sign::Plus)
			| (Sign::Minus, Sign::Minus) => Self(Sign::Plus, d),
			(Sign::Zero, Sign::Minus)
			| (Sign::Plus, Sign::Minus)
			| (Sign::Minus, Sign::Zero)
			| (Sign::Minus, Sign::Plus) => Self(Sign::Minus, d),
		}
	}
}

impl Rem for I256 {
	type Output = Self;

	fn rem(self, other: Self) -> Self {
		let r = (self.1 % other.1) & SIGN_BIT_MASK;

		if r == U256::zero() {
			return Self::zero();
		}

		Self(self.0, r)
	}
}

#[cfg(test)]
mod tests {
	use crate::utils::{Sign, I256};
	use primitive_types::U256;
	use std::num::Wrapping;

	#[test]
	fn div_i256() {
		// Sanity checks based on i8. Notice that we need to use `Wrapping` here because
		// Rust will prevent the overflow by default whereas the EVM does not.
		assert_eq!(Wrapping(i8::MIN) / Wrapping(-1), Wrapping(i8::MIN));

		assert_eq!(100i8 / -1, -100i8);
		assert_eq!(100i8 / 2, 50i8);

		// Now the same calculations based on i256
		let one = I256(Sign::Zero, U256::from(1));
		let one_hundred = I256(Sign::Zero, U256::from(100));
		let fifty = I256(Sign::Plus, U256::from(50));
		let two = I256(Sign::Zero, U256::from(2));
		let neg_one_hundred = I256(Sign::Minus, U256::from(100));
		let minus_one = I256(Sign::Minus, U256::from(1));
		let max_value = I256(Sign::Plus, U256::from(2).pow(U256::from(255)) - 1);
		let neg_max_value = I256(Sign::Minus, U256::from(2).pow(U256::from(255)) - 1);

		assert_eq!(I256::min_value() / minus_one, I256::min_value());
		assert_eq!(I256::min_value() / one, I256::min_value());
		assert_eq!(max_value / one, max_value);
		assert_eq!(max_value / minus_one, neg_max_value);

		assert_eq!(one_hundred / minus_one, neg_one_hundred);
		assert_eq!(one_hundred / two, fifty);
	}
}
