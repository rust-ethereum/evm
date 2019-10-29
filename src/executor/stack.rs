use core::convert::Infallible;
use core::cmp::min;
use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, BTreeSet};
use primitive_types::{U256, H256, H160};
use sha3::{Keccak256, Digest};
use crate::{ExitError, Stack, ExternalOpcode, Opcode, Capture, Handler,
			Context, CreateScheme, Runtime, ExitReason, ExitSucceed, Config};
use crate::backend::{Log, Basic, Apply, Backend};
use crate::gasometer::{self, Gasometer};

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct StackAccount {
	pub basic: Basic,
	pub code: Option<Vec<u8>>,
	pub storage: BTreeMap<H256, H256>,
}

#[derive(Clone)]
pub struct StackExecutor<'backend, 'config, B> {
	backend: &'backend B,
	config: &'config Config,
	gasometer: Gasometer<'config>,
	state: BTreeMap<H160, StackAccount>,
	deleted: BTreeSet<H160>,
	logs: Vec<Log>,
}

impl<'backend, 'config, B: Backend> StackExecutor<'backend, 'config, B> {
	pub fn new(
		backend: &'backend B,
		gas_limit: usize,
		config: &'config Config,
	) -> Self {
		Self {
			backend,
			gasometer: Gasometer::new(gas_limit, config),
			state: BTreeMap::new(),
			deleted: BTreeSet::new(),
			config,
			logs: Vec::new(),
		}
	}

	pub fn substate(&self, gas_limit: usize) -> StackExecutor<'backend, 'config, B> {
		Self {
			backend: self.backend,
			gasometer: Gasometer::new(gas_limit, self.gasometer.config()),
			config: self.config,
			state: self.state.clone(),
			deleted: self.deleted.clone(),
			logs: self.logs.clone(),
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

	pub fn merge_succeed<'obackend, 'oconfig, OB>(
		&mut self,
		mut substate: StackExecutor<'obackend, 'oconfig, OB>
	) -> Result<(), ExitError> {
		self.logs.append(&mut substate.logs);
		self.deleted.intersection(&substate.deleted);
		self.state = substate.state;

		self.gasometer.merge(substate.gasometer)
	}

	pub fn merge_fail<'obackend, 'oconfig, OB>(
		&mut self,
		mut substate: StackExecutor<'obackend, 'oconfig, OB>
	) -> Result<(), ExitError> {
		self.logs.append(&mut substate.logs);

		self.gasometer.merge(substate.gasometer)
	}

	pub fn transact_create(
		&mut self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		gas_limit: usize,
	) -> ExitReason {
		let transaction_cost = gasometer::create_transaction_cost(&init_code);
		self.gasometer.record_transaction(transaction_cost)?;

		let address = self.create_address(caller, CreateScheme::Dynamic)?;

		let mut substate = self.substate(self.gasometer.gas());
		substate.transfer(caller, address, value)?;

		let context = Context {
			caller,
			address,
			apparent_value: value,
		};

		match substate.create(address, init_code, Some(gas_limit), context) {
			Ok(Capture::Exit(s)) => {
				self.merge_succeed(substate)?;
				Ok(s)
			},
			Ok(Capture::Trap(_)) => unreachable!(),
			Err(e) => {
				self.merge_fail(substate)?;
				Err(e)
			},
		}
	}

	pub fn transact_call(
		&mut self,
		caller: H160,
		address: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: usize,
	) -> ExitReason {
		let transaction_cost = gasometer::call_transaction_cost(&data);
		self.gasometer.record_transaction(transaction_cost)?;

		let mut substate = self.substate(self.gasometer.gas());
		substate.transfer(caller, address, value)?;

		let context = Context {
			caller,
			address,
			apparent_value: value,
		};

		match substate.call(address, data, Some(gas_limit), false, context) {
			Ok(Capture::Exit((s, _))) => {
				self.merge_succeed(substate)?;
				Ok(s)
			},
			Ok(Capture::Trap(_)) => unreachable!(),
			Err(e) => {
				self.merge_fail(substate)?;
				Err(e)
			},
		}
	}

	pub fn pay_fee(
		&mut self,
		source: H160,
		target: H160,
		price: U256,
	) -> Result<(), ExitError> {
		let gas = self.gasometer.gas();
		let used_gas = self.gasometer.total_used_gas() - self.gasometer.refunded_gas() as usize;
		let fee = U256::from(used_gas) * price;

		self.transfer(source, target, fee)?;
		self.gasometer = Gasometer::new(gas, self.gasometer.config());

		Ok(())
	}

	#[must_use]
	pub fn deconstruct(
		self
	) -> (impl IntoIterator<Item=Apply<impl IntoIterator<Item=(H256, H256)>>>,
		  impl IntoIterator<Item=Log>)
	{
		let mut applies = Vec::<Apply<BTreeMap<H256, H256>>>::new();

		for (address, account) in self.state {
			if self.deleted.contains(&address) {
				continue
			}

			applies.push(Apply::Modify {
				address,
				basic: account.basic,
				code: account.code,
				storage: account.storage,
			});
		}

		for address in self.deleted {
			applies.push(Apply::Delete { address });
		}

		let logs = self.logs;

		(applies, logs)
	}

	pub fn account_mut(&mut self, address: H160) -> &mut StackAccount {
		self.state.entry(address).or_insert(StackAccount {
			basic: self.backend.basic(address),
			code: None,
			storage: BTreeMap::new(),
		})
	}

	pub fn nonce(&self, address: H160) -> U256 {
		self.state.get(&address).map(|v| v.basic.nonce)
			.unwrap_or(self.backend.basic(address).nonce)
	}
}

impl<'backend, 'config, B: Backend> Handler for StackExecutor<'backend, 'config, B> {
	type CreateInterrupt = Infallible;
	type CreateFeedback = Infallible;
	type CallInterrupt = Infallible;
	type CallFeedback = Infallible;

	fn balance(&self, address: H160) -> U256 {
		self.state.get(&address).map(|v| v.basic.balance)
			.unwrap_or(self.backend.basic(address).balance)
	}

	fn code_size(&self, address: H160) -> U256 {
		U256::from(
			self.state.get(&address).and_then(|v| v.code.as_ref().map(|c| c.len()))
				.unwrap_or(self.backend.code_size(address))
		)
	}

	fn code_hash(&self, address: H160) -> H256 {
		self.state.get(&address).and_then(|v| {
			v.code.as_ref().map(|c| {
				H256::from_slice(Keccak256::digest(&c).as_slice())
			})
		}).unwrap_or(self.backend.code_hash(address))
	}

	fn code(&self, address: H160) -> Vec<u8> {
		self.state.get(&address).and_then(|v| {
			v.code.clone()
		}).unwrap_or(self.backend.code(address))
	}

	fn storage(&self, address: H160, index: H256) -> H256 {
		self.state.get(&address)
			.and_then(|v| v.storage.get(&index).cloned())
			.unwrap_or(self.backend.storage(address, index))
	}

	fn original_storage(&self, address: H160, index: H256) -> H256 {
		self.backend.storage(address, index)
	}

	fn exists(&self, address: H160) -> bool {
		self.state.get(&address).is_some() || self.backend.exists(address)
	}

	fn gas_left(&self) -> U256 { U256::from(self.gasometer.gas()) }

	fn gas_price(&self) -> U256 { self.backend.gas_price() }
	fn origin(&self) -> H160 { self.backend.origin() }
	fn block_hash(&self, number: U256) -> H256 { self.backend.block_hash(number) }
	fn block_number(&self) -> U256 { self.backend.block_number() }
	fn block_coinbase(&self) -> H160 { self.backend.block_coinbase() }
	fn block_timestamp(&self) -> U256 { self.backend.block_timestamp() }
	fn block_difficulty(&self) -> U256 { self.backend.block_difficulty() }
	fn block_gas_limit(&self) -> U256 { self.backend.block_gas_limit() }
	fn chain_id(&self) -> U256 { self.backend.chain_id() }

	fn create_address(&mut self, address: H160, scheme: CreateScheme) -> Result<H160, ExitError> {
		match scheme {
			CreateScheme::Fixed(naddress) => {
				self.account_mut(address).basic.nonce += U256::one();
				Ok(naddress)
			},
			CreateScheme::Dynamic => {
				let nonce = self.nonce(address);
				self.account_mut(address).basic.nonce += U256::one();

				let mut stream = rlp::RlpStream::new_list(2);
				stream.append(&address);
				stream.append(&nonce);
				Ok(H256::from_slice(Keccak256::digest(&stream.out()).as_slice()).into())
			},
		}
	}

	fn deleted(&self, address: H160) -> bool { self.deleted.contains(&address) }

	fn is_recoverable(&self) -> bool { true }

	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError> {
		self.account_mut(address).storage.insert(index, value);

		Ok(())
	}

	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError> {
		self.logs.push(Log {
			address, topics, data
		});

		Ok(())
	}

	fn transfer(&mut self, source: H160, target: H160, value: U256) -> Result<(), ExitError> {
		{
			let source = self.account_mut(source);
			if source.basic.balance < value {
				return Err(ExitError::Other("not enough fund"))
			}
			source.basic.balance -= value;
		}

		{
			let target = self.account_mut(target);
			target.basic.balance += value;
		}

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
	) -> Result<Capture<ExitSucceed, Self::CreateInterrupt>, ExitError> {
		let after_gas = self.gasometer.gas(); // TODO: support l64(after_gas)
		let target_gas = target_gas.unwrap_or(after_gas);

		let gas_limit = min(after_gas, target_gas);

		let mut substate = self.substate(gas_limit);
		if self.config.create_increase_nonce {
			substate.account_mut(address).basic.nonce += U256::one();
		}

		let mut runtime = Runtime::new(
			Rc::new(init_code),
			Rc::new(Vec::new()),
			self.config.stack_limit,
			self.config.memory_limit,
			context,
		);

		let reason = substate.execute(&mut runtime);

		match reason {
			Ok(s) => {
				let e = self.merge_succeed(substate);
				self.state.entry(address).or_insert(Default::default())
					.code = Some(runtime.machine().return_value());
				e?;

				Ok(Capture::Exit(s))
			},
			Err(e) => {
				self.merge_fail(substate)?;
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
	) -> Result<Capture<(ExitSucceed, Vec<u8>), Self::CallInterrupt>, ExitError> {
		let after_gas = self.gasometer.gas(); // TODO: support l64(after_gas)
		let target_gas = target_gas.unwrap_or(after_gas);
		let code = self.code(code_address);

		let gas_limit = min(after_gas, target_gas);
		let mut substate = self.substate(gas_limit);
		let mut runtime = Runtime::new(
			Rc::new(code),
			Rc::new(input),
			self.config.stack_limit,
			self.config.memory_limit,
			context,
		);

		let reason = substate.execute(&mut runtime);

		match reason {
			Ok(s) => {
				self.merge_succeed(substate)?;
				Ok(Capture::Exit((s, runtime.machine().return_value())))
			},
			Err(e) => {
				self.merge_fail(substate)?;
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
		let (gas_cost, memory_cost) = gasometer::opcode_cost(context.address, opcode, stack, self)?;
		self.gasometer.record_opcode(gas_cost, memory_cost)?;

		Ok(())
	}
}
