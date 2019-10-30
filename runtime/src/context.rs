use primitive_types::{H160, U256};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CreateScheme {
	Dynamic,
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
