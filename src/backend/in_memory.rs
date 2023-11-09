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
	pub gas_price: U256,
	pub origin: H160,
}
