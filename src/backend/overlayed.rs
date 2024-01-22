use crate::{
	ExitError, ExitException, Log, MergeStrategy, RuntimeBackend, RuntimeBaseBackend,
	TransactionalBackend,
};
use alloc::collections::{BTreeMap, BTreeSet};
use core::mem;
use primitive_types::{H160, H256, U256};

pub struct OverlayedBackend<B> {
	backend: B,
	substate: Box<Substate>,
	accessed: BTreeSet<(H160, Option<H256>)>,
}

impl<B> OverlayedBackend<B> {
	pub fn new(backend: B) -> Self {
		Self {
			backend,
			substate: Box::new(Substate::new()),
			accessed: BTreeSet::new(),
		}
	}
}

impl<B: RuntimeBaseBackend> RuntimeBaseBackend for OverlayedBackend<B> {
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

impl<B: RuntimeBaseBackend> RuntimeBackend for OverlayedBackend<B> {
	fn original_storage(&self, address: H160, index: H256) -> H256 {
		self.backend.storage(address, index)
	}

	fn deleted(&self, address: H160) -> bool {
		self.substate.deleted(address)
	}

	fn is_cold(&self, address: H160, index: Option<H256>) -> bool {
		!self.accessed.contains(&(address, index))
	}

	fn mark_hot(&mut self, address: H160, index: Option<H256>) {
		self.accessed.insert((address, index));
	}

	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError> {
		self.substate.storages.insert((address, index), value);
		Ok(())
	}

	fn log(&mut self, log: Log) -> Result<(), ExitError> {
		self.substate.logs.push(log);
		Ok(())
	}

	fn mark_delete(&mut self, address: H160) {
		self.substate.deletes.insert(address);
	}

	fn reset_storage(&mut self, address: H160) {
		self.substate.storage_resets.insert(address);
	}

	fn set_code(&mut self, address: H160, code: Vec<u8>) -> Result<(), ExitError> {
		self.substate.codes.insert(address, code);
		Ok(())
	}

	fn reset_balance(&mut self, address: H160) {
		self.substate.balances.insert(address, U256::zero());
	}

	fn deposit(&mut self, target: H160, value: U256) {
		let current_balance = self.balance(target);
		self.substate
			.balances
			.insert(target, current_balance.saturating_add(value));
	}

	fn withdrawal(&mut self, source: H160, value: U256) -> Result<(), ExitError> {
		let current_balance = self.balance(source);
		if current_balance < value {
			return Err(ExitException::OutOfFund.into());
		}
		let new_balance = current_balance - value;
		self.substate.balances.insert(source, new_balance);
		Ok(())
	}

	fn inc_nonce(&mut self, address: H160) -> Result<(), ExitError> {
		let new_nonce = self.nonce(address).saturating_add(U256::from(1));
		self.substate.nonces.insert(address, new_nonce);
		Ok(())
	}
}

impl<B: RuntimeBaseBackend> TransactionalBackend for OverlayedBackend<B> {
	fn push_substate(&mut self) {
		let mut parent = Box::new(Substate::new());
		mem::swap(&mut parent, &mut self.substate);
		self.substate.parent = Some(parent);
	}

	fn pop_substate(&mut self, strategy: MergeStrategy) {
		let mut child = self.substate.parent.take().expect("uneven substate pop");
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
				for address in child.deletes {
					self.substate.deletes.insert(address);
				}
			}
			MergeStrategy::Revert | MergeStrategy::Discard => {}
		}
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
	deletes: BTreeSet<H160>,
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
			deletes: Default::default(),
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
		} else if self.storage_resets.contains(&address) {
			Some(H256::default())
		} else if let Some(parent) = self.parent.as_ref() {
			parent.known_storage(address, key)
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
}
