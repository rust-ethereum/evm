use core::convert::Infallible;
use core::cmp::min;
use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, BTreeSet};
use primitive_types::{U256, H256, H160};
use sha3::{Keccak256, Digest};
use crate::{ExitError, Stack, ExternalOpcode, Opcode, Capture, Handler,
			Context, CreateScheme, Runtime, ExitReason};
use crate::gasometer::{self, Gasometer};

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Account {
	pub nonce: U256,
	pub balance: U256,
	pub storage: BTreeMap<H256, H256>,
	pub code: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vicinity {
	pub gas_price: U256,
	pub origin: H160,
	pub block_hashes: Vec<H256>,
	pub block_number: U256,
	pub block_coinbase: H160,
	pub block_timestamp: U256,
	pub block_difficulty: U256,
	pub block_gas_limit: U256,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Log {
	pub address: H160,
	pub topics: Vec<H256>,
	pub data: Vec<u8>,
}

pub struct Executor<'ostate, 'vicinity, 'gconfig> {
	original_state: &'ostate BTreeMap<H160, Account>,
	gasometer: Gasometer<'gconfig>,
	state: BTreeMap<H160, Account>,
	vicinity: &'vicinity Vicinity,
	deleted: BTreeSet<H160>,
	logs: Vec<Log>,
}

impl<'ostate, 'vicinity, 'gconfig> Executor<'ostate, 'vicinity, 'gconfig> {
	pub fn new(
		original_state: &'ostate BTreeMap<H160, Account>,
		vicinity: &'vicinity Vicinity,
		gas_limit: usize,
		gasometer_config: &'gconfig gasometer::Config) -> Self {
		Self {
			state: original_state.clone(),
			original_state,
			vicinity,
			deleted: BTreeSet::new(),
			logs: Vec::new(),
			gasometer: Gasometer::new(gas_limit, gasometer_config),
		}
	}

	pub fn execute(&mut self, runtime: &mut Runtime) -> ExitReason {
		match runtime.run(self) {
			Capture::Exit(reason) => reason,
			Capture::Trap(_) => unreachable!("Trap is Infallible"),
		}
	}

	pub fn gas(&self) -> usize {
		self.gasometer.gas()
	}

	pub fn state(&self) -> &BTreeMap<H160, Account> {
		&self.state
	}

	pub fn finalize(&mut self) {
		for address in &self.deleted {
			self.state.remove(address);
		}

		self.deleted = BTreeSet::new();
	}

	pub fn nonce(&self, address: H160) -> U256 {
		self.state.get(&address).map(|v| v.nonce).unwrap_or(U256::zero())
	}
}

impl<'ostate, 'vicinity, 'gconfig> Handler for Executor<'ostate, 'vicinity, 'gconfig> {
	type CreateInterrupt = Infallible;
	type CreateFeedback = Infallible;
	type CallInterrupt = Infallible;
	type CallFeedback = Infallible;

	fn balance(&self, address: H160) -> U256 {
		self.state.get(&address).map(|v| v.balance).unwrap_or(U256::zero())
	}

	fn code_size(&self, address: H160) -> U256 {
		self.state.get(&address).map(|v| U256::from(v.code.len())).unwrap_or(U256::zero())
	}

	fn code_hash(&self, address: H160) -> H256 {
		self.state.get(&address).map(|v| {
			H256::from_slice(Keccak256::digest(&v.code).as_slice())
		}).unwrap_or(H256::default())
	}

	fn code(&self, address: H160) -> Vec<u8> {
		self.state.get(&address).map(|v| v.code.clone()).unwrap_or(Vec::new())
	}

	fn storage(&self, address: H160, index: H256) -> H256 {
		self.state.get(&address)
			.map(|v| v.storage.get(&index).cloned().unwrap_or(H256::default()))
			.unwrap_or(H256::default())
	}

	fn original_storage(&self, address: H160, index: H256) -> H256 {
		self.original_state.get(&address)
			.map(|v| v.storage.get(&index).cloned().unwrap_or(H256::default()))
			.unwrap_or(H256::default())
	}

	fn gas_left(&self) -> U256 { U256::from(self.gasometer.gas()) }
	fn gas_price(&self) -> U256 { self.vicinity.gas_price }
	fn origin(&self) -> H160 { self.vicinity.origin }
	fn block_hash(&self, number: U256) -> H256 {
		if number >= self.vicinity.block_number ||
			self.vicinity.block_number - number - U256::one() >= U256::from(self.vicinity.block_hashes.len())
		{
			H256::default()
		} else {
			let index = (self.vicinity.block_number - number - U256::one()).as_usize();
			self.vicinity.block_hashes[index]
		}
	}
	fn block_number(&self) -> U256 { self.vicinity.block_number }
	fn block_coinbase(&self) -> H160 { self.vicinity.block_coinbase }
	fn block_timestamp(&self) -> U256 { self.vicinity.block_timestamp }
	fn block_difficulty(&self) -> U256 { self.vicinity.block_difficulty }
	fn block_gas_limit(&self) -> U256 { self.vicinity.block_gas_limit }

	fn create_address(&mut self, address: H160, scheme: CreateScheme) -> Result<H160, ExitError> {
		match scheme {
			CreateScheme::Fixed(address) => Ok(address),
			CreateScheme::Dynamic => {
				let nonce = self.nonce(address);
				self.state.entry(address).or_insert(Default::default()).nonce += U256::one();

				let mut stream = rlp::RlpStream::new_list(2);
				stream.append(&address);
				stream.append(&nonce);
				Ok(H256::from_slice(Keccak256::digest(&stream.out()).as_slice()).into())
			},
		}
	}
	fn exists(&self, address: H160) -> bool { self.state.get(&address).is_some() }
	fn deleted(&self, address: H160) -> bool { self.deleted.contains(&address) }

	fn is_recoverable(&self) -> bool { true }

	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError> {
		match self.state.get_mut(&address) {
			Some(entry) => {
				if value == H256::default() {
					entry.storage.remove(&index);
				} else {
					entry.storage.insert(index, value);
				}
				Ok(())
			},
			None => Err(ExitError::Other("logic error: set storage, but account does not exist")),
		}
	}

	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError> {
		self.logs.push(Log {
			address, topics, data
		});

		Ok(())
	}

	fn transfer(&mut self, source: H160, target: H160, value: U256) -> Result<(), ExitError> {
		if value == U256::zero() {
			return Ok(())
		}

		match self.state.get_mut(&source) {
			Some(source) => {
				if source.balance >= value {
					source.balance -= value;
				} else {
					return Err(ExitError::Other("not enough fund"))
				}
			},
			None => return Err(ExitError::Other("not enough fund"))
		}

		self.state.entry(target)
			.or_insert(Default::default())
			.balance += value;

		Ok(())
	}

	fn mark_delete(&mut self, address: H160) -> Result<(), ExitError> {
		self.deleted.insert(address);

		Ok(())
	}

	fn create(
		&mut self,
		address: H160,
		init_code: Vec<u8>,
		target_gas: Option<usize>,
		context: Context,
	) -> Result<Capture<(), Self::CreateInterrupt>, ExitError> {
		let after_gas = self.gasometer.gas(); // TODO: support l64(after_gas)
		let target_gas = target_gas.unwrap_or(after_gas);

		let gas_limit = min(after_gas, target_gas);

		let mut substate = Self::new(
			self.original_state,
			self.vicinity,
			gas_limit,
			self.gasometer.config()
		);
		substate.state.insert(address, Default::default());

		let mut runtime = Runtime::new(
			Rc::new(init_code),
			Rc::new(Vec::new()),
			1024,
			usize::max_value(),
			context,
		);

		let reason = substate.execute(&mut runtime);

		self.gasometer.merge(substate.gasometer)?;
		self.logs.append(&mut substate.logs);

		match reason {
			ExitReason::Succeed(_) => {
				self.deleted.intersection(&substate.deleted);
				self.state = substate.state;
				self.state.entry(address).or_insert(Default::default())
					.code = runtime.machine().return_value();

				Ok(Capture::Exit(()))
			},
			ExitReason::Error(e) => {
				Err(e)
			},
		}
	}

	fn call(
		&mut self,
		code_address: H160,
		input: Vec<u8>,
		target_gas: Option<usize>,
		_is_static: bool, // TODO: support this
		context: Context,
	) -> Result<Capture<Vec<u8>, Self::CallInterrupt>, ExitError> {
		let after_gas = self.gasometer.gas(); // TODO: support l64(after_gas)
		let target_gas = target_gas.unwrap_or(after_gas);
		let code = self.code(code_address);

		let gas_limit = min(after_gas, target_gas);
		let mut substate = Self::new(
			self.original_state,
			self.vicinity,
			gas_limit,
			self.gasometer.config()
		);
		let mut runtime = Runtime::new(
			Rc::new(code),
			Rc::new(input),
			1024,
			usize::max_value(),
			context,
		);

		let reason = substate.execute(&mut runtime);

		self.gasometer.merge(substate.gasometer)?;
		self.logs.append(&mut substate.logs);

		match reason {
			ExitReason::Succeed(_) => {
				self.deleted.intersection(&substate.deleted);
				self.state = substate.state;

				Ok(Capture::Exit(runtime.machine().return_value()))
			},
			ExitReason::Error(e) => {
				Err(e)
			},
		}
	}

	fn pre_validate(
		&mut self,
		context: &Context,
		opcode: Result<Opcode, ExternalOpcode>,
		stack: &Stack
	) -> Result<(), ExitError> {
		// TODO: Add opcode check.
		let (gas_cost, memory_cost) = gasometer::cost(context.address, opcode, stack, self)?;
		self.gasometer.record(gas_cost, memory_cost)?;

		Ok(())
	}
}
