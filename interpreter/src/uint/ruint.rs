use super::U256Ext;
use ::ruint::Uint;

/// Ruint's U256 type definition.
pub type U256 = Uint<256, 4>;

impl U256Ext for U256 {
	const ZERO: U256 = U256::ZERO;
	const ONE: U256 = U256::ONE;

	fn addmod(_op1: U256, _op2: U256, _op3: U256) -> U256 {
		unimplemented!()
	}

	fn mulmod(_op1: U256, _op2: U256, _op3: U256) -> U256 {
		unimplemented!()
	}

	fn as_usize(&self) -> usize {
		self.to::<usize>()
	}
}
