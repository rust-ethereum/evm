use super::U256Ext;
use ::ruint::Uint;

/// Ruint's U256 type definition.
pub type U256 = Uint<256, 4>;

impl U256Ext for U256 {
	fn addmod(op1: U256, op2: U256, op3: U256) -> U256 {
		unimplemented!()
	}

	fn mulmod(op1: U256, op2: U256, op3: U256) -> U256 {
		unimplemented!()
	}
}
