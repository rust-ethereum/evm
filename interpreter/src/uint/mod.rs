//! Re-export of uint type that is currently use.
//!
//! Depending on the feature flag, different underlying crate may be used.

mod primitive_types;
pub use self::primitive_types::{H160, H256, U256};

/// Extension for specialized U256 operations.
pub trait U256Ext {
	/// An ADDMOD operation for U256.
	fn addmod(op1: Self, op2: Self, op3: Self) -> Self;
	/// An MULMOD operation for U256.
	fn mulmod(op1: Self, op2: Self, op3: Self) -> Self;
}
