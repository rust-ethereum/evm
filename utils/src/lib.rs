#![cfg_attr(not(feature = "std"), no_std)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub trait AdvangeManagedVec<M: ManagedTypeApi, T: ManagedVecItem> {
	fn resize(&self, size: usize, value: T) -> ManagedVec<M, T>;
}

pub trait AsBytes<M: ManagedTypeApi> {
	fn as_bytes(&self) -> Vec<u8>;
}

impl<M: ManagedTypeApi> AsBytes<M> for ManagedVec<M, u8> {
	fn as_bytes(&self) -> Vec<u8> {
		let mut data = Vec::<u8>::new();
		for i in 0..self.len() {
			let item = self.try_get(i).unwrap();
			data.push(item);
		}
		data
	}
}

impl<M: ManagedTypeApi, T: ManagedVecItem + Clone> AdvangeManagedVec<M, T> for ManagedVec<M, T> {
	fn resize(&self, size: usize, value: T) -> ManagedVec<M, T> {
		let mut result = ManagedVec::new();
		for _ in 0..size {
			result.push(value.clone());
		}
		result
	}
}
