#[allow(unused_imports)]
use crate::uint::{U256, U256Ext};
use crate::utils::{I256, Sign};

#[inline]
pub fn slt(op1: U256, op2: U256) -> U256 {
	let op1: I256 = op1.into();
	let op2: I256 = op2.into();

	if op1.lt(&op2) { U256::ONE } else { U256::ZERO }
}

#[inline]
pub fn sgt(op1: U256, op2: U256) -> U256 {
	let op1: I256 = op1.into();
	let op2: I256 = op2.into();

	if op1.gt(&op2) { U256::ONE } else { U256::ZERO }
}

#[inline]
pub fn iszero(op1: U256) -> U256 {
	if op1 == U256::ZERO {
		U256::ONE
	} else {
		U256::ZERO
	}
}

#[inline]
pub fn not(op1: U256) -> U256 {
	!op1
}

#[inline]
pub fn byte(op1: U256, op2: U256) -> U256 {
	let mut ret = U256::ZERO;

	for i in 0..256 {
		if i < 8 && op1 < U256::VALUE_32 {
			let o: usize = op1.as_usize();
			let t = 255 - (7 - i + 8 * o);
			let bit_mask = U256::ONE << t;
			let value = (op2 & bit_mask) >> t;
			ret = ret.overflowing_add(value << i).0;
		}
	}

	ret
}

#[inline]
pub fn shl(shift: U256, value: U256) -> U256 {
	if value == U256::ZERO || shift >= U256::VALUE_256 {
		U256::ZERO
	} else {
		let shift: u64 = shift.as_u64();
		value << shift as usize
	}
}

#[inline]
pub fn shr(shift: U256, value: U256) -> U256 {
	if value == U256::ZERO || shift >= U256::VALUE_256 {
		U256::ZERO
	} else {
		let shift: u64 = shift.as_u64();
		value >> shift as usize
	}
}

#[inline]
pub fn sar(shift: U256, value: U256) -> U256 {
	let value = I256::from(value);

	if value == I256::zero() || shift >= U256::VALUE_256 {
		let I256(sign, _) = value;
		match sign {
			// value is 0 or >=1, pushing 0
			Sign::Plus | Sign::Zero => U256::ZERO,
			// value is <0, pushing -1
			Sign::Minus => I256(Sign::Minus, U256::ONE).into(),
		}
	} else {
		let shift: u64 = shift.as_u64();

		match value.0 {
			Sign::Plus | Sign::Zero => value.1 >> shift as usize,
			Sign::Minus => {
				let shifted = ((value.1.overflowing_sub(U256::ONE).0) >> shift as usize)
					.overflowing_add(U256::ONE)
					.0;
				I256(Sign::Minus, shifted).into()
			}
		}
	}
}
