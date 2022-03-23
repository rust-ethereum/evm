use primitive_types::{H160, H256, U256};

/// Create scheme.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CreateScheme {
	/// Legacy create scheme of `CREATE`.
	Legacy {
		/// Caller of the create.
		caller: H160,
	},
	/// Create scheme of `CREATE2`.
	Create2 {
		/// Caller of the create.
		caller: H160,
		/// Code hash.
		code_hash: H256,
		/// Salt.
		salt: H256,
	},
	/// Create at a fixed location.
	Fixed(H160),
}

/// Call scheme.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CallScheme {
	/// `CALL`
	Call,
	/// `CALLCODE`
	CallCode,
	/// `DELEGATECALL`
	DelegateCall,
	/// `STATICCALL`
	StaticCall,
}

/// Context of the runtime.
#[derive(Clone, Debug)]
pub struct Context {
	/// Execution address.
	pub address: H160,
	/// Caller of the EVM.
	pub caller: H160,
	/// Apparent value of the EVM.
	pub apparent_value: U256,
}
