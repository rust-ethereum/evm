#![cfg_attr(not(feature = "std"), no_std)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(
	TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Default, Clone, Debug, ManagedVecItem,
)]
pub struct EH256 {
	pub data: [u8; 32],
}

impl EH256 {
	pub fn from(h256: primitive_types::H256) -> Self {
		Self { data: h256.0 }
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.data
	}

	pub fn to_h256(&self) -> primitive_types::H256 {
		primitive_types::H256(self.data)
	}
}

pub trait ToEH256 {
	fn to_eh256(self) -> EH256;
}

impl ToEH256 for primitive_types::H256 {
	fn to_eh256(self) -> EH256 {
		EH256::from(self)
	}
}

pub trait ManagedVecforEH256<M: ManagedTypeApi> {
	fn managedvec_bytes(&self) -> ManagedVec<M, u8>;
}

impl<M: ManagedTypeApi> ManagedVecforEH256<M> for EH256 {
	fn managedvec_bytes(&self) -> ManagedVec<M, u8> {
		let mut result = ManagedVec::new();
		for i in 0..self.data.len() {
			result.push(self.data[i]);
		}
		result
	}
}
