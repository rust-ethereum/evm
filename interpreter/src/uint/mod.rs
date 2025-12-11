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
// Note on name resolution: when trait and struct itself defines functions of
// identical name, then Rust would by default calls the implementation on the
// struct directly. We take advantage of this for the extension trait.
pub trait U256Ext: Sized {
	/// Zero value.
	const ZERO: Self;
	/// One value.
	const ONE: Self;

	/// An ADDMOD operation for U256.
	fn addmod(op1: Self, op2: Self, op3: Self) -> Self;
	/// An MULMOD operation for U256.
	fn mulmod(op1: Self, op2: Self, op3: Self) -> Self;

	/// Conversion to usize with overflow checking.
	/// Should panic if the number is larger than usize::MAX.
	fn as_usize(&self) -> usize;

	/// Conversion to H256 big-endian.
	fn to_h256(self) -> H256;
	/// Conversion from H256 big-endian.
	fn from_h256(v: H256) -> Self;
	/// Conversion to H160 big-endian, discard leading bits.
	fn to_h160(self) -> H160 {
		self.to_h256().into()
	}
	/// Conversion from H160 big-endian, with leading bits as zero.
	fn from_h160(v: H160) -> Self {
		Self::from_h256(v.into())
	}
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
