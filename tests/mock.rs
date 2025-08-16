extern crate evm;

use std::collections::BTreeMap;

use evm::backend::{OverlayedChangeSet, RuntimeBaseBackend, RuntimeEnvironment};
use primitive_types::{H160, H256, U256};

#[derive(Default, Clone, Debug)]
pub struct MockAccount {
	pub balance: U256,
	pub code: Vec<u8>,
	pub nonce: U256,
	pub storage: BTreeMap<H256, H256>,
	pub transient_storage: BTreeMap<H256, H256>,
}

#[derive(Clone, Debug, Default)]
pub struct MockBackend {
	pub state: BTreeMap<H160, MockAccount>,
}

impl MockBackend {
	pub fn apply_overlayed(&mut self, changeset: &OverlayedChangeSet) {
		for (address, balance) in changeset.balances.clone() {
			self.state.entry(address).or_default().balance = balance;
		}

		for (address, code) in changeset.codes.clone() {
			self.state.entry(address).or_default().code = code;
		}

		for (address, nonce) in changeset.nonces.clone() {
			self.state.entry(address).or_default().nonce = nonce;
		}

		for address in changeset.storage_resets.clone() {
			self.state.entry(address).or_default().storage = BTreeMap::new();
		}

		for ((address, key), value) in changeset.storages.clone() {
			let account = self.state.entry(address).or_default();

			if value == H256::default() {
				account.storage.remove(&key);
			} else {
				account.storage.insert(key, value);
			}
		}

		for address in changeset.deletes.clone() {
			self.state.remove(&address);
		}
	}
}

impl RuntimeEnvironment for MockBackend {
	fn block_hash(&self, _number: U256) -> H256 {
		Default::default()
	}

	fn block_number(&self) -> U256 {
		Default::default()
	}

	fn block_coinbase(&self) -> H160 {
		Default::default()
	}

	fn block_timestamp(&self) -> U256 {
		Default::default()
	}

	fn block_difficulty(&self) -> U256 {
		Default::default()
	}

	fn block_randomness(&self) -> Option<H256> {
		Default::default()
	}

	fn block_gas_limit(&self) -> U256 {
		Default::default()
	}

	fn block_base_fee_per_gas(&self) -> U256 {
		Default::default()
	}

	fn blob_base_fee_per_gas(&self) -> U256 {
		Default::default()
	}

	fn blob_versioned_hash(&self, _index: U256) -> H256 {
		Default::default()
	}

	fn chain_id(&self) -> U256 {
		Default::default()
	}
}

impl RuntimeBaseBackend for MockBackend {
	fn balance(&self, address: H160) -> U256 {
		self.state
			.get(&address)
			.cloned()
			.unwrap_or(Default::default())
			.balance
	}

	fn code(&self, address: H160) -> Vec<u8> {
		self.state
			.get(&address)
			.cloned()
			.unwrap_or(Default::default())
			.code
	}

	fn exists(&self, address: H160) -> bool {
		self.state.contains_key(&address)
	}

	fn storage(&self, address: H160, index: H256) -> H256 {
		self.state
			.get(&address)
			.cloned()
			.unwrap_or(Default::default())
			.storage
			.get(&index)
			.cloned()
			.unwrap_or(H256::default())
	}

	fn transient_storage(&self, address: H160, index: H256) -> H256 {
		self.state
			.get(&address)
			.cloned()
			.unwrap_or(Default::default())
			.transient_storage
			.get(&index)
			.cloned()
			.unwrap_or(H256::default())
	}

	fn nonce(&self, address: H160) -> U256 {
		self.state
			.get(&address)
			.cloned()
			.unwrap_or(Default::default())
			.nonce
	}
}
