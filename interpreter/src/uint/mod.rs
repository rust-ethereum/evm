//! Re-export of uint type that is currently use.
//!
//! Depending on the feature flag, different underlying crate may be used.
//! * Default (no feature flag): use `primitive-types` crate.
//! * `ruint` feature flag: use `ruint` crate.
//! * `ethnum` feature flag: use `ethnum` crate.

// H256 and H160 are pretty standardized, and there's no performance difference
// in different implementations, so we always only use the one from
// `primitive-types`.
pub use ::primitive_types::{H160, H256};

/// Extension for specialized U256 operations.
pub trait U256Ext {
	/// An ADDMOD operation for U256.
	fn addmod(op1: Self, op2: Self, op3: Self) -> Self;
	/// An MULMOD operation for U256.
	fn mulmod(op1: Self, op2: Self, op3: Self) -> Self;
}

// Use default primitive-types U256 implementation.
#[cfg(all(not(feature = "ruint"), not(feature = "ethnum")))]
mod primitive_types;
#[cfg(all(not(feature = "ruint"), not(feature = "ethnum")))]
pub use self::primitive_types::U256;

// Use ruint U256 implementation.
#[cfg(feature = "ruint")]
mod ruint;
#[cfg(feature = "ruint")]
pub use self::ruint::U256;

// Use ethnum U256 implementation.
#[cfg(feature = "ethnum")]
mod ethnum;
#[cfg(feature = "ethnum")]
pub use self::ethnum::U256;
