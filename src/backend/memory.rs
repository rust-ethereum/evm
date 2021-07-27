use super::{Apply, ApplyBackend, Backend, Basic, Log};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use primitive_types::{H160, H256, U256};

/// Vivinity value of a memory backend.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemoryVicinity {
	/// Gas price.
	pub gas_price: U256,
	/// Origin.
	pub origin: H160,
	/// Chain ID.
	pub chain_id: U256,
	/// Environmental block hashes.
	pub block_hashes: Vec<H256>,
	/// Environmental block number.
	pub block_number: U256,
	/// Environmental coinbase.
	pub block_coinbase: H160,
	/// Environmental block timestamp.
	pub block_timestamp: U256,
	/// Environmental block difficulty.
	pub block_difficulty: U256,
	/// Environmental block gas limit.
	pub block_gas_limit: U256,
}

/// Account information of a memory backend.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemoryAccount {
	/// Account nonce.
	pub nonce: U256,
	/// Account balance.
	pub balance: U256,
	/// Full account storage.
	pub storage: BTreeMap<H256, H256>,
	/// Account code.
	pub code: Vec<u8>,
}

/// Memory backend, storing all state values in a `BTreeMap` in memory.
#[derive(Clone, Debug)]
pub struct MemoryBackend<'vicinity> {
	vicinity: &'vicinity MemoryVicinity,
	state: BTreeMap<H160, MemoryAccount>,
	logs: Vec<Log>,
}

impl<'vicinity> MemoryBackend<'vicinity> {
	/// Create a new memory backend.
	pub fn new(vicinity: &'vicinity MemoryVicinity, state: BTreeMap<H160, MemoryAccount>) -> Self {
		Self {
			vicinity,
			state,
			logs: Vec::new(),
		}
	}

	/// Get the underlying `BTreeMap` storing the state.
	pub fn state(&self) -> &BTreeMap<H160, MemoryAccount> {
		&self.state
	}
}

impl<'vicinity> Backend for MemoryBackend<'vicinity> {
	fn gas_price(&self) -> U256 {
		self.vicinity.gas_price
	}
	fn origin(&self) -> H160 {
		self.vicinity.origin
	}
	fn block_hash(&self, number: U256) -> H256 {
		if number >= self.vicinity.block_number
			|| self.vicinity.block_number - number - U256::one()
				>= U256::from(self.vicinity.block_hashes.len())
		{
			H256::default()
		} else {
			let index = (self.vicinity.block_number - number - U256::one()).as_usize();
			self.vicinity.block_hashes[index]
		}
	}
	fn block_number(&self) -> U256 {
		self.vicinity.block_number
	}
	fn block_coinbase(&self) -> H160 {
		self.vicinity.block_coinbase
	}
	fn block_timestamp(&self) -> U256 {
		self.vicinity.block_timestamp
	}
	fn block_difficulty(&self) -> U256 {
		self.vicinity.block_difficulty
	}
	fn block_gas_limit(&self) -> U256 {
		self.vicinity.block_gas_limit
	}

	fn chain_id(&self) -> U256 {
		self.vicinity.chain_id
	}

	fn exists(&self, address: H160) -> bool {
		self.state.contains_key(&address)
	}

	fn basic(&self, address: H160) -> Basic {
		self.state
			.get(&address)
			.map(|a| Basic {
				balance: a.balance,
				nonce: a.nonce,
			})
			.unwrap_or_default()
	}

	fn code(&self, address: H160) -> Vec<u8> {
		self.state
			.get(&address)
			.map(|v| v.code.clone())
			.unwrap_or_default()
	}

	fn storage(&self, address: H160, index: H256) -> H256 {
		self.state
			.get(&address)
			.map(|v| v.storage.get(&index).cloned().unwrap_or_default())
			.unwrap_or_default()
	}

	fn original_storage(&self, address: H160, index: H256) -> Option<H256> {
		Some(self.storage(address, index))
	}
}

impl<'vicinity> ApplyBackend for MemoryBackend<'vicinity> {
	fn apply<A, I, L>(&mut self, values: A, logs: L, delete_empty: bool)
	where
		A: IntoIterator<Item = Apply<I>>,
		I: IntoIterator<Item = (H256, H256)>,
		L: IntoIterator<Item = Log>,
	{
		for apply in values {
			match apply {
				Apply::Modify {
					address,
					basic,
					code,
					storage,
					reset_storage,
				} => {
					let is_empty = {
						let account = self.state.entry(address).or_insert_with(Default::default);
						account.balance = basic.balance;
						account.nonce = basic.nonce;
						if let Some(code) = code {
							account.code = code;
						}

						if reset_storage {
							account.storage = BTreeMap::new();
						}

						let zeros = account
							.storage
							.iter()
							.filter(|(_, v)| v == &&H256::default())
							.map(|(k, _)| *k)
							.collect::<Vec<H256>>();

						for zero in zeros {
							account.storage.remove(&zero);
						}

						for (index, value) in storage {
							if value == H256::default() {
								account.storage.remove(&index);
							} else {
								account.storage.insert(index, value);
							}
						}

						account.balance == U256::zero()
							&& account.nonce == U256::zero()
							&& account.code.is_empty()
					};

					if is_empty && delete_empty {
						self.state.remove(&address);
					}
				}
				Apply::Delete { address } => {
					self.state.remove(&address);
				}
			}
		}

		for log in logs {
			self.logs.push(log);
		}
	}
}
