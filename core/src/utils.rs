use core::ops::{Rem, Div};
use core::cmp::Ordering;
use primitive_types::U256;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Sign {
	Plus,
	Minus,
	NoSign,
}

const SIGN_BIT_MASK: U256 = U256([0xffffffffffffffff, 0xffffffffffffffff,
								  0xffffffffffffffff, 0x7fffffffffffffff]);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct I256(pub Sign, pub U256);

impl I256 {
	/// Zero value of I256.
	pub fn zero() -> I256 { I256(Sign::NoSign, U256::zero()) }
	/// Minimum value of I256.
	pub fn min_value() -> I256 { I256(Sign::Minus, (U256::max_value() & SIGN_BIT_MASK) + U256::from(1u64)) }
}

impl Ord for I256 {
	fn cmp(&self, other: &I256) -> Ordering {
		match (self.0, other.0) {
			(Sign::NoSign, Sign::NoSign) => Ordering::Equal,
			(Sign::NoSign, Sign::Plus) => Ordering::Less,
			(Sign::NoSign, Sign::Minus) => Ordering::Greater,
			(Sign::Minus, Sign::NoSign) => Ordering::Less,
			(Sign::Minus, Sign::Plus) => Ordering::Less,
			(Sign::Minus, Sign::Minus) => self.1.cmp(&other.1).reverse(),
			(Sign::Plus, Sign::Minus) => Ordering::Greater,
			(Sign::Plus, Sign::NoSign) => Ordering::Greater,
			(Sign::Plus, Sign::Plus) => self.1.cmp(&other.1),
		}
	}
}

impl PartialOrd for I256 {
	fn partial_cmp(&self, other: &I256) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Default for I256 { fn default() -> I256 { I256::zero() } }
impl From<U256> for I256 {
	fn from(val: U256) -> I256 {
		if val == U256::zero() {
			I256::zero()
		} else if val & SIGN_BIT_MASK.into() == val {
			I256(Sign::Plus, val)
		} else {
			I256(Sign::Minus, !val + U256::from(1u64))
		}
	}
}
impl Into<U256> for I256 {
	fn into(self) -> U256 {
		let sign = self.0;
		if sign == Sign::NoSign {
			U256::zero()
		} else if sign == Sign::Plus {
			self.1
		} else {
			!self.1 + U256::from(1u64)
		}
	}
}

impl Div for I256 {
	type Output = I256;

	fn div(self, other: I256) -> I256 {
		if other == I256::zero() {
			return I256::zero();
		}

		if self == I256::min_value() && other == I256(Sign::Minus, U256::from(1u64)) {
			return I256::min_value();
		}

		let d = (self.1 / other.1) & SIGN_BIT_MASK.into();

		if d == U256::zero() {
			return I256::zero();
		}

		match (self.0, other.0) {
			(Sign::Plus, Sign::Plus) |
			(Sign::Minus, Sign::Minus) => I256(Sign::Plus, d),
			(Sign::Plus, Sign::Minus) |
			(Sign::Minus, Sign::Plus) => I256(Sign::Minus, d),
			_ => I256::zero()
		}
	}
}

impl Rem for I256 {
	type Output = I256;

	fn rem(self, other: I256) -> I256 {
		let r = (self.1 % other.1) & SIGN_BIT_MASK.into();

		if r == U256::zero() {
			return I256::zero()
		}

		I256(self.0, r)
	}
}
