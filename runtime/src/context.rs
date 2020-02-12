use primitive_types::{H160, U256, H256};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CreateScheme {
	Legacy {
		caller: H160,
	},
	Create2 {
		caller: H160,
		code_hash: H256,
		salt: H256,
	},
	Fixed(H160),
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CallScheme {
	Call,
	CallCode,
	DelegateCall,
	StaticCall,
}

#[derive(Clone, Debug)]
pub struct Context {
	pub address: H160,
	pub caller: H160,
	pub apparent_value: U256,
}
