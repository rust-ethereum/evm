use super::{H256, U256Ext};
use ::primitive_types::U512;

pub use ::primitive_types::U256;

const _: () = assert!(usize::BITS <= 64);

impl U256Ext for U256 {
	const ZERO: U256 = U256::zero();
	const ONE: U256 = U256::one();
	const VALUE_32: U256 = U256([32, 0, 0, 0]);
	const VALUE_64: U256 = U256([64, 0, 0, 0]);
	const VALUE_256: U256 = U256([256, 0, 0, 0]);
	const USIZE_MAX: U256 = U256([usize::MAX as u64, 0, 0, 0]);
	const SIGN_BIT_MASK: U256 = U256([
		0xffff_ffff_ffff_ffff,
		0xffff_ffff_ffff_ffff,
		0xffff_ffff_ffff_ffff,
		0x7fff_ffff_ffff_ffff,
	]);

	fn addmod(op1: U256, op2: U256, op3: U256) -> U256 {
		let op1: U512 = op1.into();
		let op2: U512 = op2.into();
		let op3: U512 = op3.into();

		if op3 == U512::zero() {
			U256::ZERO
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
			U256::ZERO
		} else {
			let v = (op1 * op2) % op3;
			v.try_into()
				.expect("op3 is less than U256::MAX, thus it never overflows; qed")
		}
	}

	fn as_usize(&self) -> usize {
		self.as_usize()
	}

	fn as_u64(&self) -> u64 {
		self.as_u64()
	}

	fn low_u32(&self) -> u32 {
		self.low_u32()
	}

	fn from_u32(v: u32) -> U256 {
		U256::from(v)
	}

	fn from_u64(v: u64) -> U256 {
		U256::from(v)
	}

	fn from_usize(v: usize) -> U256 {
		U256::from(v)
	}

	fn to_h256(self) -> H256 {
		let mut r = H256::default();
		self.to_big_endian(&mut r[..]);
		r
	}

	fn from_h256(v: H256) -> Self {
		U256::from_big_endian(&v[..])
	}

	fn bit(&self, index: usize) -> bool {
		self.bit(index)
	}

	fn log2floor(&self) -> u64 {
		let value = *self;

		assert_ne!(value, U256::ZERO);
		let mut l: u64 = 256;
		for i in 0..4 {
			let i = 3 - i;
			if value.0[i] == 0u64 {
				l -= 64;
			} else {
				l -= value.0[i].leading_zeros() as u64;
				if l == 0 {
					return l;
				} else {
					return l - 1;
				}
			}
		}
		l
	}

	fn append_to_rlp_stream(&self, rlp: &mut rlp::RlpStream) {
		rlp.append(self);
	}
}
