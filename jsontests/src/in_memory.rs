use std::collections::BTreeMap;

use evm::{
	backend::OverlayedChangeSet,
	interpreter::runtime::{RuntimeBaseBackend, RuntimeEnvironment},
};
use primitive_types::{H160, H256, U256};

#[derive(Clone, Debug)]
pub struct InMemoryEnvironment {
	pub block_hashes: BTreeMap<U256, H256>,
	pub block_number: U256,
	pub block_coinbase: H160,
	pub block_timestamp: U256,
	pub block_difficulty: U256,
	pub block_randomness: Option<H256>,
	pub block_gas_limit: U256,
	pub block_base_fee_per_gas: U256,
	pub chain_id: U256,
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryAccount {
	pub balance: U256,
	pub code: Vec<u8>,
	pub nonce: U256,
	pub storage: BTreeMap<H256, H256>,
}

#[derive(Clone, Debug)]
pub struct InMemorySuicideInfo {
	pub address: H160,
}

#[derive(Clone, Debug)]
pub struct InMemoryBackend {
	pub environment: InMemoryEnvironment,
	pub state: BTreeMap<H160, InMemoryAccount>,
}

impl InMemoryBackend {
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

impl RuntimeEnvironment for InMemoryBackend {
	fn block_hash(&self, number: U256) -> H256 {
		self.environment
			.block_hashes
			.get(&number)
			.cloned()
			.unwrap_or(H256::default())
	}

	fn block_number(&self) -> U256 {
		self.environment.block_number
	}

	fn block_coinbase(&self) -> H160 {
		self.environment.block_coinbase
	}

	fn block_timestamp(&self) -> U256 {
		self.environment.block_timestamp
	}

	fn block_difficulty(&self) -> U256 {
		self.environment.block_difficulty
	}

	fn block_randomness(&self) -> Option<H256> {
		self.environment.block_randomness
	}

	fn block_gas_limit(&self) -> U256 {
		self.environment.block_gas_limit
	}

	fn block_base_fee_per_gas(&self) -> U256 {
		self.environment.block_base_fee_per_gas
	}

	fn chain_id(&self) -> U256 {
		self.environment.chain_id
	}
}

impl RuntimeBaseBackend for InMemoryBackend {
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
		self.state.get(&address).is_some()
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

	fn nonce(&self, address: H160) -> U256 {
		self.state
			.get(&address)
			.cloned()
			.unwrap_or(Default::default())
			.nonce
	}
}
