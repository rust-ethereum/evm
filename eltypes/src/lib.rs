#![cfg_attr(not(feature = "std"), no_std)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(
	ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Default,
)]
pub struct Hello<M: ManagedTypeApi> {
	pub eth_address: ETHAddress,
	pub manage_vec: ManagedVec<M, EH256>,
}

#[derive(
	ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Default,
)]
pub struct ETHAddress {
	pub data: [u8; 20],
}

impl ETHAddress {
	pub fn from(h160: primitive_types::H160) -> Self {
		Self { data: h160.0 }
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.data
	}

	pub fn to_h160(&self) -> primitive_types::H160 {
		primitive_types::H160(self.data)
	}
}
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
impl Eq for EH256 {}

impl PartialEq for EH256 {
	fn eq(&self, other: &Self) -> bool {
		//TODO: Implement
		true
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

pub trait ToH256 {
	fn to_h256(self) -> H256;
}
impl ToH256 for EH256 {
	fn to_h256(self) -> H256 {
		H256::from(&self.data)
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
