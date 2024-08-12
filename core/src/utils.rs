use core::cmp::Ordering;
use core::ops::{Div, Rem};
use primitive_types::U256;

/// Precalculated `usize::MAX` for `U256`
#[allow(clippy::as_conversions)]
pub const USIZE_MAX: U256 = U256([usize::MAX as u64, 0, 0, 0]);
/// Precalculated `u64::MAX` for `U256`
pub const U64_MAX: U256 = U256([u64::MAX, 0, 0, 0]);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Sign {
	Plus,
	Minus,
	Zero,
}

const SIGN_BIT_MASK: U256 = U256([
	0xffff_ffff_ffff_ffff,
	0xffff_ffff_ffff_ffff,
	0xffff_ffff_ffff_ffff,
	0x7fff_ffff_ffff_ffff,
]);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct I256(pub Sign, pub U256);

impl I256 {
	/// Zero value of I256.
	#[must_use]
	pub const fn zero() -> Self {
		Self(Sign::Zero, U256::zero())
	}
	/// Minimum value of I256.
	#[must_use]
	pub fn min_value() -> Self {
		Self(Sign::Minus, (U256::MAX & SIGN_BIT_MASK) + U256::from(1u64))
	}
}

impl Ord for I256 {
	fn cmp(&self, other: &Self) -> Ordering {
		match (self.0, other.0) {
			(Sign::Zero, Sign::Zero) => Ordering::Equal,
			(Sign::Zero | Sign::Minus, Sign::Plus) | (Sign::Minus, Sign::Zero) => Ordering::Less,
			(Sign::Minus, Sign::Minus) => self.1.cmp(&other.1).reverse(),
			(Sign::Zero | Sign::Plus, Sign::Minus) | (Sign::Plus, Sign::Zero) => Ordering::Greater,
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
			(Sign::Zero | Sign::Plus, Sign::Plus | Sign::Zero) | (Sign::Minus, Sign::Minus) => {
				Self(Sign::Plus, d)
			}

			(Sign::Zero | Sign::Plus, Sign::Minus) | (Sign::Minus, Sign::Zero | Sign::Plus) => {
				Self(Sign::Minus, d)
			}
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
