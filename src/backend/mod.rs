mod memory;

pub use self::memory::{MemoryBackend, MemoryVicinity, MemoryAccount};

use primitive_types::{H160, H256, U256};

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct Basic {
	pub balance: U256,
	pub nonce: U256,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Log {
	pub address: H160,
	pub topics: Vec<H256>,
	pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub enum Apply<I> {
	Modify {
		address: H160,
		basic: Basic,
		code: Option<Vec<u8>>,
		storage: I,
	},
	Delete {
		address: H160,
	},
}

pub trait Backend {
	fn gas_price(&self) -> U256;
	fn origin(&self) -> H160;
	fn block_hash(&self, number: U256) -> H256;
	fn block_number(&self) -> U256;
	fn block_coinbase(&self) -> H160;
	fn block_timestamp(&self) -> U256;
	fn block_difficulty(&self) -> U256;
	fn block_gas_limit(&self) -> U256;
	fn chain_id(&self) -> U256;

	fn exists(&self, address: H160) -> bool;
	fn basic(&self, address: H160) -> Basic;
	fn code_hash(&self, address: H160) -> H256;
	fn code_size(&self, address: H160) -> usize;
	fn code(&self, address: H160) -> Vec<u8>;
	fn storage(&self, address: H160, index: H256) -> H256;
}

pub trait ApplyBackend {
	fn apply<A, I, L>(
		&mut self,
		values: A,
		logs: L,
		delete_empty: bool,
	) where
		A: IntoIterator<Item=Apply<I>>,
		I: IntoIterator<Item=(H256, H256)>,
		L: IntoIterator<Item=Log>;
}
