use core::mem;
use alloc::collections::{BTreeMap, BTreeSet};
use primitive_types::{H160, H256, U256};
use crate::{ExitError, Transfer};
use crate::backend::{Basic, Log, Backend, Apply};
use crate::executor::stack::StackSubstateMetadata;

#[derive(Clone)]
struct MemoryStackAccount {
	pub basic: Basic,
	pub code: Option<Vec<u8>>,
	pub reset: bool,
}

pub struct MemoryStackSubstate<'config> {
	metadata: StackSubstateMetadata<'config>,
	parent: Option<Box<MemoryStackSubstate<'config>>>,
	logs: Vec<Log>,
	accounts: BTreeMap<H160, MemoryStackAccount>,
	storages: BTreeMap<(H160, H256), H256>,
	deletes: BTreeSet<H160>,
}

impl<'config> MemoryStackSubstate<'config> {
	pub fn new(metadata: StackSubstateMetadata<'config>) -> Self {
		Self {
			metadata,
			parent: None,
			logs: Vec::new(),
			accounts: BTreeMap::new(),
			storages: BTreeMap::new(),
			deletes: BTreeSet::new(),
		}
	}

	pub fn metadata(&self) -> &StackSubstateMetadata<'config> {
		&self.metadata
	}

	pub fn metadata_mut(&mut self) -> &mut StackSubstateMetadata<'config> {
		&mut self.metadata
	}

	#[must_use]
	pub fn deconstruct<B: Backend>(
		mut self, backend: &B,
	) -> (impl IntoIterator<Item=Apply<impl IntoIterator<Item=(H256, H256)>>>,
 		  impl IntoIterator<Item=Log>)
	{
		assert!(self.parent.is_none());

		let mut applies = Vec::<Apply<BTreeMap<H256, H256>>>::new();

		let mut addresses = BTreeSet::new();

		for address in self.accounts.keys() {
			addresses.insert(*address);
		}

		for (address, _) in self.storages.keys() {
			addresses.insert(*address);
		}

		for address in addresses {
			if self.deletes.contains(&address) {
				continue
			}

			let mut storage = BTreeMap::new();
			for ((oa, ok), ov) in &self.storages {
				if *oa == address {
					storage.insert(*ok, *ov);
				}
			}

			let apply = {
				let account = self.account_mut(address, backend);

				Apply::Modify {
					address,
					basic: account.basic.clone(),
					code: account.code.clone(),
					storage,
					reset_storage: account.reset,
				}
			};

			applies.push(apply);
		}

		for address in self.deletes {
			applies.push(Apply::Delete { address });
		}

		(applies, self.logs)
	}

	pub fn enter(&mut self, gas_limit: u64, is_static: bool) {
		let mut entering = Self {
			metadata: self.metadata.spit_child(gas_limit, is_static),
			parent: None,
			logs: Vec::new(),
			accounts: BTreeMap::new(),
			storages: BTreeMap::new(),
			deletes: BTreeSet::new(),
		};
		mem::swap(&mut entering, self);

		self.parent = Some(Box::new(entering));
	}

	pub fn exit_commit(&mut self) -> Result<(), ExitError> {
		let mut exited = *self.parent.take().expect("Cannot commit on root substate");
		mem::swap(&mut exited, self);

		self.metadata.swallow_commit(exited.metadata)?;
		self.logs.append(&mut exited.logs);
		self.accounts.append(&mut exited.accounts);
		self.storages.append(&mut exited.storages);
		self.deletes.append(&mut exited.deletes);

		Ok(())
	}

	pub fn exit_revert(&mut self) -> Result<(), ExitError> {
		let mut exited = *self.parent.take().expect("Cannot discard on root substate");
		mem::swap(&mut exited, self);

		self.metadata.swallow_revert(exited.metadata)?;
		self.logs.append(&mut exited.logs);

		Ok(())
	}

	pub fn exit_discard(&mut self) -> Result<(), ExitError> {
		let mut exited = *self.parent.take().expect("Cannot discard on root substate");
		mem::swap(&mut exited, self);

		self.metadata.swallow_discard(exited.metadata)?;
		self.logs.append(&mut exited.logs);

		Ok(())
	}

	fn known_account(&self, address: H160) -> Option<&MemoryStackAccount> {
		if let Some(account) = self.accounts.get(&address) {
			Some(account)
		} else if let Some(parent) = self.parent.as_ref() {
			parent.known_account(address)
		} else {
			None
		}
	}

	pub fn known_basic(&self, address: H160) -> Option<Basic> {
		self.known_account(address).map(|acc| acc.basic.clone())
	}

	pub fn known_code(&self, address: H160) -> Option<Vec<u8>> {
		self.known_account(address).and_then(|acc| acc.code.clone())
	}

	pub fn known_empty(&self, address: H160) -> Option<bool> {
		if let Some(account) = self.known_account(address) {
			if let Some(code) = &account.code {
				return Some(
					account.basic.balance == U256::zero() &&
						account.basic.nonce == U256::zero() &&
						code.len() == 0
				)
			}
		}

		None
	}

	pub fn known_storage(&self, address: H160, key: H256) -> Option<H256> {
		if let Some(value) = self.storages.get(&(address, key)) {
			return Some(*value)
		}

		if let Some(account) = self.accounts.get(&address) {
			if account.reset {
				return Some(H256::default())
			}
		}

		if let Some(parent) = self.parent.as_ref() {
			return parent.known_storage(address, key)
		}

		None
	}

	pub fn known_original_storage(&self, address: H160, key: H256) -> Option<H256> {
		if let Some(account) = self.accounts.get(&address) {
			if account.reset {
				return Some(H256::default())
			}
		}

		if let Some(parent) = self.parent.as_ref() {
			return parent.known_original_storage(address, key)
		}

		None
	}

	pub fn deleted(&self, address: H160) -> bool {
		if self.deletes.contains(&address) {
			return true
		}

		if let Some(parent) = self.parent.as_ref() {
			return parent.deleted(address)
		}

		false
	}

	fn account_mut<B: Backend>(&mut self, address: H160, backend: &B) -> &mut MemoryStackAccount {
		if !self.accounts.contains_key(&address) {
			let account = self.known_account(address)
				.cloned()
				.unwrap_or_else(|| MemoryStackAccount {
					basic: backend.basic(address),
					code: None,
					reset: false,
				});
			self.accounts.insert(address, account);
		}

		self.accounts.get_mut(&address).expect("New account was just inserted")
	}

	pub fn inc_nonce<B: Backend>(&mut self, address: H160, backend: &B) {
		self.account_mut(address, backend).basic.nonce += U256::one();
	}

	pub fn set_storage(&mut self, address: H160, key: H256, value: H256) {
		self.storages.insert((address, key), value);
	}

	pub fn reset_storage<B: Backend>(&mut self, address: H160, backend: &B) {
		let mut removing = Vec::new();

		for (oa, ok) in self.storages.keys() {
			if *oa == address {
				removing.push(*ok);
			}
		}

		for ok in removing {
			self.storages.remove(&(address, ok));
		}

		self.account_mut(address, backend).reset = true;
	}

	pub fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) {
		self.logs.push(Log {
			address, topics, data,
		});
	}

	pub fn set_deleted(&mut self, address: H160) {
		self.deletes.insert(address);
	}

	pub fn set_code<B: Backend>(&mut self, address: H160, code: Vec<u8>, backend: &B) {
		self.account_mut(address, backend).code = Some(code);
	}

	pub fn transfer<B: Backend>(&mut self, transfer: Transfer, backend: &B) -> Result<(), ExitError> {
		{
			let source = self.account_mut(transfer.source, backend);
			if source.basic.balance < transfer.value {
				return Err(ExitError::OutOfFund)
			}
			source.basic.balance -= transfer.value;
		}

		{
			let target = self.account_mut(transfer.target, backend);
			target.basic.balance.saturating_add(transfer.value);
		}

		Ok(())
	}

	pub fn reset_balance<B: Backend>(&mut self, address: H160, backend: &B) {
		self.account_mut(address, backend).basic.balance = U256::zero();
	}

	pub fn touch<B: Backend>(&mut self, address: H160, backend: &B) {
		self.account_mut(address, backend);
	}
}

// pub struct MemoryStackState<'backend, 'config, B> {
// 	backend: &'backend B,
// 	substate: MemoryStackSubstate<'config>,
// }
