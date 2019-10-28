use primitive_types::{H160, H256, U256};

pub struct Account {
	pub balance: U256,
	pub nonce: U256,
}

pub enum Apply<I> {
	Modify {
		address: H160,
		account: Account,
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

	fn account(&self, address: H160) -> Account;
	fn code_hash(&self, address: H160) -> H256;
	fn code_size(&self, address: H160) -> usize;
	fn code(&self, address: H160) -> Vec<u8>;
	fn storage(&self, address: H160, index: H256) -> H256;

	fn apply<A: IntoIterator<Item=Apply<I>>, I: IntoIterator<Item=(H256, H256)>>(
		&mut self,
		values: A,
	);
}
