use crate::{ExitError, ExitException, Log, RuntimeBackend, RuntimeFullBackend, Transfer};
use alloc::collections::{BTreeMap, BTreeSet};
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
	pub gas_price: U256,
	pub origin: H160,
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryAccount {
	pub balance: U256,
	pub code: Vec<u8>,
	pub nonce: U256,
	pub storage: BTreeMap<H256, H256>,
	pub original_storage: BTreeMap<H256, H256>,
}

#[derive(Clone, Debug)]
pub struct InMemorySuicideInfo {
	pub address: H160,
	pub target: H160,
}

#[derive(Clone, Debug)]
pub struct InMemoryBackend {
	pub environment: InMemoryEnvironment,
	pub state: BTreeMap<H160, InMemoryAccount>,
	pub logs: Vec<Log>,
	pub suicides: Vec<InMemorySuicideInfo>,
	pub hots: BTreeSet<(H160, Option<H256>)>,
}

impl RuntimeBackend for InMemoryBackend {
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

	fn gas_price(&self) -> U256 {
		self.environment.gas_price
	}

	fn origin(&self) -> H160 {
		self.environment.origin
	}

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

	fn original_storage(&self, address: H160, index: H256) -> H256 {
		self.state
			.get(&address)
			.cloned()
			.unwrap_or(Default::default())
			.storage
			.get(&index)
			.cloned()
			.unwrap_or(H256::default())
	}

	fn exists(&self, address: H160) -> bool {
		self.state.get(&address).is_some()
	}

	fn deleted(&self, address: H160) -> bool {
		self.suicides
			.iter()
			.any(|suicide| suicide.address == address)
	}

	fn is_cold(&self, address: H160, index: Option<H256>) -> bool {
		!self.hots.contains(&(address, index))
	}

	fn mark_hot(&mut self, address: H160, index: Option<H256>) -> Result<(), ExitError> {
		self.hots.insert((address, index));
		Ok(())
	}

	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError> {
		self.state
			.entry(address)
			.or_default()
			.storage
			.insert(index, value);
		Ok(())
	}

	fn log(&mut self, log: Log) -> Result<(), ExitError> {
		self.logs.push(log);
		Ok(())
	}

	fn mark_delete(&mut self, address: H160, target: H160) -> Result<(), ExitError> {
		self.suicides.push(InMemorySuicideInfo { address, target });
		Ok(())
	}
}

impl RuntimeFullBackend for InMemoryBackend {
	fn nonce(&self, address: H160) -> U256 {
		self.state
			.get(&address)
			.cloned()
			.unwrap_or(Default::default())
			.nonce
	}

	fn reset_storage(&mut self, address: H160) {
		self.state.entry(address).or_default().storage = Default::default();
	}

	fn set_code(&mut self, address: H160, code: Vec<u8>) {
		self.state.entry(address).or_default().code = code;
	}

	fn reset_balance(&mut self, address: H160) {
		self.state.entry(address).or_default().balance = U256::zero();
	}

	fn transfer(&mut self, transfer: Transfer) -> Result<(), ExitError> {
		{
			let source = self.state.entry(transfer.source).or_default();
			if source.balance < transfer.value {
				return Err(ExitException::OutOfFund.into());
			}
			source.balance -= transfer.value;
		}
		self.state.entry(transfer.target).or_default().balance += transfer.value;
		Ok(())
	}

	fn inc_nonce(&mut self, address: H160) -> Result<(), ExitError> {
		let entry = self.state.entry(address).or_default();
		entry.nonce = entry.nonce.saturating_add(U256::one());
		Ok(())
	}
}
