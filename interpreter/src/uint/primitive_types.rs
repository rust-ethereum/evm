pub use primitive_types::{H160, H256, U256};

use super::U256Ext;
use primitive_types::U512;

impl U256Ext for U256 {
	fn addmod(op1: U256, op2: U256, op3: U256) -> U256 {
		let op1: U512 = op1.into();
		let op2: U512 = op2.into();
		let op3: U512 = op3.into();

		if op3 == U512::zero() {
			U256::zero()
		} else {
			let v = (op1 + op2) % op3;
			v.try_into()
				.expect("op3 is less than U256::MAX, thus it never overflows; qed")
		}
	}

	fn mulmod(op1: U256, op2: U256, op3: U256) -> U256 {
		let op1: U512 = op1.into();
		let op2: U512 = op2.into();
		let op3: U512 = op3.into();

		if op3 == U512::zero() {
			U256::zero()
		} else {
			let v = (op1 * op2) % op3;
			v.try_into()
				.expect("op3 is less than U256::MAX, thus it never overflows; qed")
		}
	}
}
