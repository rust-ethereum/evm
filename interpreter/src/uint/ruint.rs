use super::{H256, U256Ext};
use ::ruint::Uint;

/// Ruint's U256 type definition.
pub type U256 = Uint<256, 4>;

const _: () = assert!(usize::BITS <= 64);

pub trait Sealed {}
impl Sealed for U256 {}

impl U256Ext for U256 {
	const ZERO: U256 = U256::ZERO;
	const ONE: U256 = U256::ONE;
	const VALUE_32: U256 = U256::from_limbs([32, 0, 0, 0]);
	const VALUE_64: U256 = U256::from_limbs([64, 0, 0, 0]);
	const VALUE_256: U256 = U256::from_limbs([256, 0, 0, 0]);
	const USIZE_MAX: U256 = U256::from_limbs([usize::MAX as u64, 0, 0, 0]);
	const U64_MAX: U256 = U256::from_limbs([u64::MAX, 0, 0, 0]);
	const U32_MAX: U256 = U256::from_limbs([u32::MAX as u64, 0, 0, 0]);
	const SIGN_BIT_MASK: U256 = U256::from_limbs([
		0xffff_ffff_ffff_ffff,
		0xffff_ffff_ffff_ffff,
		0xffff_ffff_ffff_ffff,
		0x7fff_ffff_ffff_ffff,
	]);

	fn add_mod(self, op2: U256, op3: U256) -> U256 {
		self.add_mod(op2, op3)
	}

	fn mul_mod(self, op2: U256, op3: U256) -> U256 {
		self.mul_mod(op2, op3)
	}

	fn to_usize(&self) -> usize {
		self.to::<usize>()
	}

	fn to_u64(&self) -> u64 {
		self.to::<u64>()
	}

	fn to_u32(&self) -> u32 {
		self.to::<u32>()
	}

	fn low_u32(&self) -> u32 {
		self.wrapping_to()
	}

	fn low_u64(&self) -> u64 {
		self.wrapping_to()
	}

	fn low_usize(&self) -> usize {
		self.wrapping_to()
	}

	fn from_u32(v: u32) -> Self {
		U256::from(v)
	}

	fn from_u64(v: u64) -> Self {
		U256::from(v)
	}

	fn from_usize(v: usize) -> Self {
		U256::from(v)
	}

	fn to_h256(self) -> H256 {
		H256(self.to_be_bytes())
	}

	fn from_h256(v: H256) -> Self {
		U256::from_be_bytes(v.0)
	}

	fn bit(&self, index: usize) -> bool {
		self.bit(index)
	}

	fn bits(&self) -> usize {
		self.bit_len()
	}

	fn log2floor(&self) -> u64 {
		let value = *self;

		let mut l: u64 = 256;
		let mut i = 3;
		loop {
			if value.as_limbs()[i] == 0u64 {
				l -= 64;
			} else {
				l -= value.as_limbs()[i].leading_zeros() as u64;
				if l == 0 {
					return l;
				} else {
					return l - 1;
				}
			}
			if i == 0 {
				break;
			}
			i -= 1;
		}
		l
	}

	fn append_to_rlp_stream(&self, rlp: &mut rlp::RlpStream) {
		rlp.append(self);
	}
}
