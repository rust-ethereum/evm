use super::{H256, U256Ext};

pub use ::ethnum::U256;

const _: () = assert!(usize::BITS <= 128);

impl U256Ext for U256 {
	const ZERO: U256 = U256::ZERO;
	const ONE: U256 = U256::ONE;
	const VALUE_32: U256 = U256::new(32);
	const VALUE_64: U256 = U256::new(64);
	const VALUE_256: U256 = U256::new(256);
	const USIZE_MAX: U256 = U256::new(usize::MAX as u128);
	const SIGN_BIT_MASK: U256 = U256::from_words(
		0x7fff_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
		0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
	);

	fn addmod(_op1: U256, _op2: U256, _op3: U256) -> U256 {
		unimplemented!()
	}

	fn mulmod(_op1: U256, _op2: U256, _op3: U256) -> U256 {
		unimplemented!()
	}

	fn as_usize(&self) -> usize {
		unimplemented!()
	}

	fn as_u64(&self) -> u64 {
		unimplemented!()
	}

	fn low_u32(&self) -> u32 {
		unimplemented!()
	}

	fn from_u32(v: u32) -> Self {
		U256::from(v)
	}

	fn from_u64(v: u64) -> Self {
		U256::from(v)
	}

	fn from_usize(_v: usize) -> Self {
		unimplemented!()
	}

	fn to_h256(self) -> H256 {
		unimplemented!()
	}

	fn from_h256(_v: H256) -> Self {
		unimplemented!()
	}

	fn bit(&self, _index: usize) -> bool {
		unimplemented!()
	}

	fn log2floor(&self) -> u64 {
		unimplemented!()
	}

	fn append_to_rlp_stream(&self, _rlp: &mut rlp::RlpStream) {
		unimplemented!()
	}
}
