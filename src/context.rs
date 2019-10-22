use primitive_types::{H160, U256};

pub enum CreateScheme {
	Create,
	Create2
}

pub enum CallScheme {
	Call,
	CallCode,
	DelegateCall,
	StaticCall,
}

pub enum ActionValue {
	Transfer(U256),
	Apparent(U256),
}

impl ActionValue {
	pub fn value(&self) -> &U256 {
		match self {
			ActionValue::Transfer(val) => val,
			ActionValue::Apparent(val) => val,
		}
	}
}

pub struct Context {
	pub address: H160,
	pub caller: H160,
	pub origin: H160,
	pub gas_price: U256,
	pub value: ActionValue,
}
