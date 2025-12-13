//! Small utilities.

use core::{
	cmp::Ordering,
	ops::{Div, Rem},
};

use crate::error::{ExitError, ExitFatal};
#[allow(unused_imports)]
use crate::uint::{H160, H256, U256, U256Ext};

/// Convert [U256] into [H256].
#[must_use]
pub fn u256_to_h256(v: U256) -> H256 {
	v.to_h256()
}

/// Convert [H256] to [U256].
#[must_use]
pub fn h256_to_u256(v: H256) -> U256 {
	U256::from_h256(v)
}

/// Convert [U256] into [H160]
#[must_use]
pub fn u256_to_h160(v: U256) -> H160 {
	v.to_h160()
}

/// Convert [U256] to [usize].
pub fn u256_to_usize(v: U256) -> Result<usize, ExitError> {
	if v > U256::USIZE_MAX {
		return Err(ExitFatal::NotSupported.into());
	}
	Ok(v.low_usize())
}

/// Sign of [I256].
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Sign {
	/// Plus
	Plus,
	/// Minus
	Minus,
	/// Zero
	Zero,
}

/// Signed 256-bit integer.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct I256(pub Sign, pub U256);

impl I256 {
	/// Zero value of I256.
	#[must_use]
	pub const fn zero() -> I256 {
		I256(Sign::Zero, U256::ZERO)
	}
	/// Minimum value of I256.
	#[must_use]
	pub fn min_value() -> I256 {
		I256(
			Sign::Minus,
			(U256::MAX & U256::SIGN_BIT_MASK) + U256::from(1u64),
		)
	}
}

impl Ord for I256 {
	fn cmp(&self, other: &I256) -> Ordering {
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
	fn partial_cmp(&self, other: &I256) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Default for I256 {
	fn default() -> I256 {
		I256::zero()
	}
}

impl From<U256> for I256 {
	fn from(val: U256) -> I256 {
		if val == U256::ZERO {
			I256::zero()
		} else if val & U256::SIGN_BIT_MASK == val {
			I256(Sign::Plus, val)
		} else {
			I256(Sign::Minus, !val + U256::from(1u64))
		}
	}
}

impl From<I256> for U256 {
	fn from(value: I256) -> U256 {
		let sign = value.0;
		if sign == Sign::Zero {
			U256::ZERO
		} else if sign == Sign::Plus {
			value.1
		} else {
			!value.1 + U256::from(1u64)
		}
	}
}

impl Div for I256 {
	type Output = I256;

	fn div(self, other: I256) -> I256 {
		if other == I256::zero() {
			return I256::zero();
		}

		if self == I256::min_value() && other.1 == U256::from(1u64) {
			return I256::min_value();
		}

		let d = (self.1 / other.1) & U256::SIGN_BIT_MASK;

		if d == U256::ZERO {
			return I256::zero();
		}

		match (self.0, other.0) {
			(Sign::Zero, Sign::Plus)
			| (Sign::Plus, Sign::Zero)
			| (Sign::Zero, Sign::Zero)
			| (Sign::Plus, Sign::Plus)
			| (Sign::Minus, Sign::Minus) => I256(Sign::Plus, d),
			(Sign::Zero, Sign::Minus)
			| (Sign::Plus, Sign::Minus)
			| (Sign::Minus, Sign::Zero)
			| (Sign::Minus, Sign::Plus) => I256(Sign::Minus, d),
		}
	}
}

impl Rem for I256 {
	type Output = I256;

	fn rem(self, other: I256) -> I256 {
		let r = (self.1 % other.1) & U256::SIGN_BIT_MASK;

		if r == U256::ZERO {
			return I256::zero();
		}

		I256(self.0, r)
	}
}

#[cfg(test)]
mod tests {
	use std::num::Wrapping;

	use super::*;

	#[test]
	fn div_i256() {
		// Sanity checks based on i8. Notice that we need to use `Wrapping` here because
		// Rust will prevent the overflow by default whereas the EVM does not.
		assert_eq!(Wrapping(i8::MIN) / Wrapping(-1), Wrapping(i8::MIN));

		assert_eq!(100i8 / -1, -100i8);
		assert_eq!(100i8 / 2, 50i8);

		// Now the same calculations based on i256
		let one = I256(Sign::Zero, U256::from_usize(1));
		let one_hundred = I256(Sign::Zero, U256::from_usize(100));
		let fifty = I256(Sign::Plus, U256::from_usize(50));
		let two = I256(Sign::Zero, U256::from_usize(2));
		let neg_one_hundred = I256(Sign::Minus, U256::from_usize(100));
		let minus_one = I256(Sign::Minus, U256::from_usize(1));
		let max_value = I256(
			Sign::Plus,
			U256::from_usize(2).pow(U256::from_usize(255)) - U256::ONE,
		);
		let neg_max_value = I256(
			Sign::Minus,
			U256::from_usize(2).pow(U256::from_usize(255)) - U256::ONE,
		);

		assert_eq!(I256::min_value() / minus_one, I256::min_value());
		assert_eq!(I256::min_value() / one, I256::min_value());
		assert_eq!(max_value / one, max_value);
		assert_eq!(max_value / minus_one, neg_max_value);

		assert_eq!(one_hundred / minus_one, neg_one_hundred);
		assert_eq!(one_hundred / two, fifty);
	}
}
