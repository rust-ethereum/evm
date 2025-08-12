extern crate alloc;

use alloc::vec::Vec;
use core::convert::TryFrom;
use primitive_types::H160;

/// EIP-7702 delegation designator prefix
pub const EIP_7702_DELEGATION_PREFIX: &[u8] = &[0xef, 0x01, 0x00];

/// EIP-7702 delegation designator full length (prefix + address)
pub const EIP_7702_DELEGATION_SIZE: usize = 23;

/// EIP-7702 delegation designator struct for managing delegation addresses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Delegation {
	address: H160,
}

impl Delegation {
	/// Create a new delegation designator from an address
	pub fn new(address: H160) -> Self {
		Self { address }
	}

	/// Convert the delegation designator to its bytecode representation
	pub fn to_bytes(&self) -> Vec<u8> {
		let mut designator = Vec::with_capacity(EIP_7702_DELEGATION_SIZE);
		designator.extend_from_slice(EIP_7702_DELEGATION_PREFIX);
		designator.extend_from_slice(self.address.as_bytes());
		designator
	}

	/// Get the delegated address
	pub fn address(&self) -> &H160 {
		&self.address
	}

	/// Consume the designator and return the address
	pub fn into_address(self) -> H160 {
		self.address
	}
}

/// Check if code is an EIP-7702 delegation designator
pub fn is_delegation_designator(code: &[u8]) -> bool {
	code.len() == EIP_7702_DELEGATION_SIZE && code.starts_with(EIP_7702_DELEGATION_PREFIX)
}

impl From<H160> for Delegation {
	fn from(address: H160) -> Self {
		Self::new(address)
	}
}

impl TryFrom<&[u8]> for Delegation {
	type Error = DelegationError;

	fn try_from(code: &[u8]) -> Result<Self, Self::Error> {
		if !is_delegation_designator(code) {
			return Err(DelegationError::InvalidFormat);
		}

		let mut address_bytes = [0u8; 20];
		address_bytes.copy_from_slice(&code[3..23]);
		Ok(Self {
			address: H160::from(address_bytes),
		})
	}
}

/// Error type for delegation operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationError {
	/// The provided bytes do not represent a valid delegation designator
	InvalidFormat,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_delegation_designator_creation() {
		let address = H160::from_slice(&[1u8; 20]);
		let designator = Delegation::new(address);
		let bytes = designator.to_bytes();

		assert_eq!(bytes.len(), EIP_7702_DELEGATION_SIZE);
		assert_eq!(&bytes[0..3], EIP_7702_DELEGATION_PREFIX);
		assert_eq!(&bytes[3..23], address.as_bytes());
		assert_eq!(*designator.address(), address);
	}

	#[test]
	fn test_delegation_designator_detection() {
		let address = H160::from_slice(&[1u8; 20]);
		let designator = Delegation::new(address);
		let bytes = designator.to_bytes();

		assert!(is_delegation_designator(&bytes));
		let extracted = Delegation::try_from(&bytes);
		assert_eq!(extracted, Some(designator));
		assert_eq!(*extracted.unwrap().address(), address);
	}

	#[test]
	fn test_non_delegation_code() {
		let regular_code = vec![0x60, 0x00]; // PUSH1 0
		assert!(!is_delegation_designator(&regular_code));
		assert_eq!(Delegation::try_from(&regular_code), None);
	}
}
