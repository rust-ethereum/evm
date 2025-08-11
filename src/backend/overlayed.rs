use alloc::{
	boxed::Box,
	collections::{BTreeMap, BTreeSet},
	vec::Vec,
};
use core::mem;

use evm_interpreter::{
	runtime::{
		Log, RuntimeBackend, RuntimeBaseBackend, RuntimeEnvironment, SetCodeOrigin, TouchKind,
	},
	ExitError, ExitException, ExitFatal,
};
use primitive_types::{H160, H256, U256};

use crate::{backend::TransactionalBackend, standard::Config, MergeStrategy};

const RIPEMD: H160 = H160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]);

#[derive(Clone, Debug)]
pub struct OverlayedChangeSet {
	pub logs: Vec<Log>,
	pub balances: BTreeMap<H160, U256>,
	pub codes: BTreeMap<H160, Vec<u8>>,
	pub nonces: BTreeMap<H160, U256>,
	pub storage_resets: BTreeSet<H160>,
	pub storages: BTreeMap<(H160, H256), H256>,
	pub transient_storage: BTreeMap<(H160, H256), H256>,
	pub accessed: BTreeSet<(H160, Option<H256>)>,
	pub touched: BTreeSet<H160>,
	pub deletes: BTreeSet<H160>,
}

pub struct OverlayedBackend<'config, B> {
	backend: B,
	substate: Box<Substate>,
	accessed: BTreeSet<(H160, Option<H256>)>,
	touched_ripemd: bool,
	config: &'config Config,
}

impl<'config, B> OverlayedBackend<'config, B> {
	pub fn new(
		backend: B,
		accessed: BTreeSet<(H160, Option<H256>)>,
		config: &'config Config,
	) -> Self {
		Self {
			backend,
			substate: Box::new(Substate::new()),
			accessed,
			touched_ripemd: false,
			config,
		}
	}

	pub fn deconstruct(mut self) -> (B, OverlayedChangeSet) {
		if self.touched_ripemd {
			self.substate.touched.insert(RIPEMD);
		}

		(
			self.backend,
			OverlayedChangeSet {
				logs: self.substate.logs,
				balances: self.substate.balances,
				codes: self.substate.codes,
				nonces: self.substate.nonces,
				storage_resets: self.substate.storage_resets,
				storages: self.substate.storages,
				transient_storage: self.substate.transient_storage,
				deletes: self.substate.deletes,
				accessed: self.accessed,
				touched: self.substate.touched,
			},
		)
	}
}

impl<B: RuntimeEnvironment> RuntimeEnvironment for OverlayedBackend<'_, B> {
	fn block_hash(&self, number: U256) -> H256 {
		self.backend.block_hash(number)
	}

	fn block_number(&self) -> U256 {
		self.backend.block_number()
	}

	fn block_coinbase(&self) -> H160 {
		self.backend.block_coinbase()
	}

	fn block_timestamp(&self) -> U256 {
		self.backend.block_timestamp()
	}

	fn block_difficulty(&self) -> U256 {
		self.backend.block_difficulty()
	}

	fn block_randomness(&self) -> Option<H256> {
		self.backend.block_randomness()
	}

	fn block_gas_limit(&self) -> U256 {
		self.backend.block_gas_limit()
	}

	fn block_base_fee_per_gas(&self) -> U256 {
		self.backend.block_base_fee_per_gas()
	}

	fn chain_id(&self) -> U256 {
		self.backend.chain_id()
	}
}

impl<B: RuntimeBaseBackend> RuntimeBaseBackend for OverlayedBackend<'_, B> {
	fn balance(&self, address: H160) -> U256 {
		if let Some(balance) = self.substate.known_balance(address) {
			balance
		} else {
			self.backend.balance(address)
		}
	}

	fn code(&self, address: H160) -> Vec<u8> {
		if let Some(code) = self.substate.known_code(address) {
			code
		} else {
			self.backend.code(address)
		}
	}

	fn storage(&self, address: H160, index: H256) -> H256 {
		if let Some(value) = self.substate.known_storage(address, index) {
			value
		} else {
			self.backend.storage(address, index)
		}
	}

	fn transient_storage(&self, address: H160, index: H256) -> H256 {
		if let Some(value) = self.substate.known_transient_storage(address, index) {
			value
		} else {
			self.backend.transient_storage(address, index)
		}
	}

	fn exists(&self, address: H160) -> bool {
		if let Some(exists) = self.substate.known_exists(address) {
			exists
		} else {
			self.backend.exists(address)
		}
	}

	fn nonce(&self, address: H160) -> U256 {
		if let Some(nonce) = self.substate.known_nonce(address) {
			nonce
		} else {
			self.backend.nonce(address)
		}
	}
}

impl<B: RuntimeBaseBackend> RuntimeBackend for OverlayedBackend<'_, B> {
	fn original_storage(&self, address: H160, index: H256) -> H256 {
		if let Some(value) = self.substate.known_original_storage(address, index) {
			value
		} else {
			self.backend.storage(address, index)
		}
	}

	fn created(&self, address: H160) -> bool {
		self.substate.created(address)
	}

	fn deleted(&self, address: H160) -> bool {
		self.substate.deleted(address)
	}

	fn is_cold(&self, address: H160, index: Option<H256>) -> bool {
		!self.accessed.contains(&(address, index))
	}

	fn mark_hot(&mut self, address: H160, kind: TouchKind) {
		self.accessed.insert((address, None));

		if kind == TouchKind::StateChange {
			if address == RIPEMD {
				self.touched_ripemd = true;
			}
			self.substate.touched.insert(address);
		}
	}

	fn mark_storage_hot(&mut self, address: H160, index: H256) {
		self.accessed.insert((address, Some(index)));
	}

	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError> {
		self.substate.storages.insert((address, index), value);
		Ok(())
	}

	fn set_transient_storage(
		&mut self,
		address: H160,
		index: H256,
		value: H256,
	) -> Result<(), ExitError> {
		self.substate
			.transient_storage
			.insert((address, index), value);
		Ok(())
	}

	fn log(&mut self, log: Log) -> Result<(), ExitError> {
		self.substate.logs.push(log);
		Ok(())
	}

	fn mark_delete_reset(&mut self, address: H160) {
		self.substate.balances.insert(address, U256::zero());

		if self.config.suicide_only_in_same_tx {
			if self.created(address) {
				self.substate.deletes.insert(address);
			}
		} else {
			self.substate.deletes.insert(address);
		}
	}

	fn mark_create(&mut self, address: H160) {
		self.substate.creates.insert(address);
	}

	fn reset_storage(&mut self, address: H160) {
		self.substate.storage_resets.insert(address);
	}

	fn set_code(
		&mut self,
		address: H160,
		code: Vec<u8>,
		_origin: SetCodeOrigin,
	) -> Result<(), ExitError> {
		self.substate.codes.insert(address, code);
		Ok(())
	}

	fn deposit(&mut self, target: H160, value: U256) {
		if value == U256::zero() {
			return;
		}

		let current_balance = self.balance(target);
		self.substate
			.balances
			.insert(target, current_balance.saturating_add(value));
	}

	fn withdrawal(&mut self, source: H160, value: U256) -> Result<(), ExitError> {
		if value == U256::zero() {
			return Ok(());
		}

		let current_balance = self.balance(source);
		if current_balance < value {
			return Err(ExitException::OutOfFund.into());
		}
		let new_balance = current_balance - value;
		self.substate.balances.insert(source, new_balance);
		Ok(())
	}

	fn inc_nonce(&mut self, address: H160) -> Result<(), ExitError> {
		let old_nonce = self.nonce(address);
		if old_nonce >= U256::from(u64::MAX) {
			return Err(ExitException::MaxNonce.into());
		}
		let new_nonce = old_nonce.saturating_add(U256::from(1));
		self.substate.nonces.insert(address, new_nonce);
		Ok(())
	}
}

impl<'config, B: RuntimeBaseBackend> TransactionalBackend for OverlayedBackend<'config, B> {
	fn push_substate(&mut self) {
		let mut parent = Box::new(Substate::new());
		mem::swap(&mut parent, &mut self.substate);
		self.substate.parent = Some(parent);
	}

	fn pop_substate(&mut self, strategy: MergeStrategy) -> Result<(), ExitError> {
		let mut child = self
			.substate
			.parent
			.take()
			.ok_or(ExitError::Fatal(ExitFatal::UnevenSubstate))?;
		mem::swap(&mut child, &mut self.substate);
		let child = child;

		match strategy {
			MergeStrategy::Commit => {
				for log in child.logs {
					self.substate.logs.push(log);
				}
				for (address, balance) in child.balances {
					self.substate.balances.insert(address, balance);
				}
				for (address, code) in child.codes {
					self.substate.codes.insert(address, code);
				}
				for (address, nonce) in child.nonces {
					self.substate.nonces.insert(address, nonce);
				}
				for address in child.storage_resets {
					self.substate.storage_resets.insert(address);
				}
				for ((address, key), value) in child.storages {
					self.substate.storages.insert((address, key), value);
				}
				for ((address, key), value) in child.transient_storage {
					self.substate
						.transient_storage
						.insert((address, key), value);
				}
				for address in child.deletes {
					self.substate.deletes.insert(address);
				}
				for address in child.creates {
					self.substate.creates.insert(address);
				}
				for address in child.touched {
					self.substate.touched.insert(address);
				}
			}
			MergeStrategy::Revert | MergeStrategy::Discard => {}
		}

		Ok(())
	}
}

struct Substate {
	parent: Option<Box<Substate>>,
	logs: Vec<Log>,
	balances: BTreeMap<H160, U256>,
	codes: BTreeMap<H160, Vec<u8>>,
	nonces: BTreeMap<H160, U256>,
	storage_resets: BTreeSet<H160>,
	storages: BTreeMap<(H160, H256), H256>,
	transient_storage: BTreeMap<(H160, H256), H256>,
	deletes: BTreeSet<H160>,
	creates: BTreeSet<H160>,
	touched: BTreeSet<H160>,
}

impl Substate {
	pub fn new() -> Self {
		Self {
			parent: None,
			logs: Vec::new(),
			balances: Default::default(),
			codes: Default::default(),
			nonces: Default::default(),
			storage_resets: Default::default(),
			storages: Default::default(),
			transient_storage: Default::default(),
			deletes: Default::default(),
			creates: Default::default(),
			touched: Default::default(),
		}
	}

	pub fn known_balance(&self, address: H160) -> Option<U256> {
		if let Some(balance) = self.balances.get(&address) {
			Some(*balance)
		} else if let Some(parent) = self.parent.as_ref() {
			parent.known_balance(address)
		} else {
			None
		}
	}

	pub fn known_code(&self, address: H160) -> Option<Vec<u8>> {
		if let Some(code) = self.codes.get(&address) {
			Some(code.clone())
		} else if let Some(parent) = self.parent.as_ref() {
			parent.known_code(address)
		} else {
			None
		}
	}

	pub fn known_nonce(&self, address: H160) -> Option<U256> {
		if let Some(nonce) = self.nonces.get(&address) {
			Some(*nonce)
		} else if let Some(parent) = self.parent.as_ref() {
			parent.known_nonce(address)
		} else {
			None
		}
	}

	pub fn known_storage(&self, address: H160, key: H256) -> Option<H256> {
		if let Some(value) = self.storages.get(&(address, key)) {
			Some(*value)
		} else if self.deletes.contains(&address) {
			None
		} else if self.storage_resets.contains(&address) {
			Some(H256::default())
		} else if let Some(parent) = self.parent.as_ref() {
			parent.known_storage(address, key)
		} else {
			None
		}
	}

	pub fn known_original_storage(&self, address: H160, _key: H256) -> Option<H256> {
		if self.deletes.contains(&address) {
			None
		} else if self.storage_resets.contains(&address) {
			Some(H256::default())
		} else if let Some(parent) = self.parent.as_ref() {
			parent.known_original_storage(address, _key)
		} else {
			None
		}
	}

	pub fn known_transient_storage(&self, address: H160, key: H256) -> Option<H256> {
		if let Some(value) = self.transient_storage.get(&(address, key)) {
			Some(*value)
		} else if let Some(parent) = self.parent.as_ref() {
			parent.known_transient_storage(address, key)
		} else {
			None
		}
	}

	pub fn known_exists(&self, address: H160) -> Option<bool> {
		if self.balances.contains_key(&address)
			|| self.nonces.contains_key(&address)
			|| self.codes.contains_key(&address)
		{
			Some(true)
		} else if let Some(parent) = self.parent.as_ref() {
			parent.known_exists(address)
		} else {
			None
		}
	}

	pub fn deleted(&self, address: H160) -> bool {
		if self.deletes.contains(&address) {
			true
		} else if let Some(parent) = self.parent.as_ref() {
			parent.deleted(address)
		} else {
			false
		}
	}

	pub fn created(&self, address: H160) -> bool {
		if self.creates.contains(&address) {
			true
		} else if let Some(parent) = self.parent.as_ref() {
			parent.created(address)
		} else {
			false
		}
	}
}
