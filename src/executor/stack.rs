use core::convert::Infallible;
use core::cmp::min;
use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, BTreeSet};
use primitive_types::{U256, H256, H160};
use sha3::{Keccak256, Digest};
use crate::{ExitError, Stack, ExternalOpcode, Opcode, Capture, Handler, Transfer,
			Context, CreateScheme, Runtime, ExitReason, ExitSucceed, Config};
use crate::backend::{Log, Basic, Apply, Backend};
use crate::gasometer::{self, Gasometer};

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct StackAccount {
	pub basic: Basic,
	pub code: Option<Vec<u8>>,
	pub storage: BTreeMap<H256, H256>,
	pub reset_storage: bool,
}

#[derive(Clone)]
pub struct StackExecutor<'backend, 'config, B> {
	backend: &'backend B,
	config: &'config Config,
	gasometer: Gasometer<'config>,
	state: BTreeMap<H160, StackAccount>,
	deleted: BTreeSet<H160>,
	logs: Vec<Log>,
	precompile: fn(H160, &[u8], Option<usize>) -> Option<Result<(ExitSucceed, Vec<u8>, usize), ExitError>>,
	is_static: bool,
	depth: Option<usize>,
}

fn no_precompile(
	_address: H160,
	_input: &[u8],
	_target_gas: Option<usize>
) -> Option<Result<(ExitSucceed, Vec<u8>, usize), ExitError>> {
	None
}

impl<'backend, 'config, B: Backend> StackExecutor<'backend, 'config, B> {
	pub fn new(
		backend: &'backend B,
		gas_limit: usize,
		config: &'config Config,
	) -> Self {
		Self::new_with_precompile(backend, gas_limit, config, no_precompile)
	}

	pub fn new_with_precompile(
		backend: &'backend B,
		gas_limit: usize,
		config: &'config Config,
		precompile: fn(H160, &[u8], Option<usize>) -> Option<Result<(ExitSucceed, Vec<u8>, usize), ExitError>>,
	) -> Self {
		Self {
			backend,
			gasometer: Gasometer::new(gas_limit, config),
			state: BTreeMap::new(),
			deleted: BTreeSet::new(),
			config,
			logs: Vec::new(),
			precompile: precompile,
			is_static: false,
			depth: None,
		}
	}

	pub fn substate(&self, gas_limit: usize, is_static: bool) -> StackExecutor<'backend, 'config, B> {
		Self {
			backend: self.backend,
			gasometer: Gasometer::new(gas_limit, self.gasometer.config()),
			config: self.config,
			state: self.state.clone(),
			deleted: self.deleted.clone(),
			logs: self.logs.clone(),
			precompile: self.precompile,
			is_static: is_static || self.is_static,
			depth: match self.depth {
				None => Some(0),
				Some(n) => Some(n + 1),
			},
		}
	}

	pub fn execute(&mut self, runtime: &mut Runtime) -> ExitReason {
		match runtime.run(self) {
			Capture::Exit(s) => s,
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
		self.deleted.append(&mut substate.deleted);
		self.state = substate.state;

		self.gasometer.merge_succeed(substate.gasometer)
	}

	pub fn merge_fail<'obackend, 'oconfig, OB>(
		&mut self,
		mut substate: StackExecutor<'obackend, 'oconfig, OB>
	) -> Result<(), ExitError> {
		self.logs.append(&mut substate.logs);

		self.gasometer.merge_fail(substate.gasometer)
	}

	pub fn transact_create(
		&mut self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		gas_limit: usize,
	) -> ExitReason {
		let transaction_cost = gasometer::create_transaction_cost(&init_code);
		match self.gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => return e.into(),
		}

		let address = match self.create_address(caller, CreateScheme::Dynamic) {
			Ok(a) => a,
			Err(e) => return e.into(),
		};

		let context = Context {
			caller,
			address,
			apparent_value: value,
		};

		match self.create_inner(address, Some(Transfer {
			source: caller,
			target: address,
			value
		}), init_code, Some(gas_limit), false, context) {
			Capture::Exit(s) => s,
			Capture::Trap(_) => unreachable!(),
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
		match self.gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => return e.into(),
		}

		self.account_mut(caller).basic.nonce += U256::one();

		let context = Context {
			caller,
			address,
			apparent_value: value,
		};

		match self.call_inner(address, Some(Transfer {
			source: caller,
			target: address,
			value
		}), data, Some(gas_limit), false, false, context) {
			Capture::Exit((s, _)) => s,
			Capture::Trap(_) => unreachable!(),
		}
	}

	pub fn pay_fee(
		&mut self,
		source: H160,
		target: H160,
		price: U256,
	) -> Result<(), ExitError> {
		let gas = self.gasometer.gas();
		let used_gas = self.gasometer.total_used_gas() -
			min(self.gasometer.total_used_gas() / 2, self.gasometer.refunded_gas() as usize);
		let fee = U256::from(used_gas) * price;

		self.transfer(Transfer { source, target, value: fee })?;
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
				reset_storage: account.reset_storage,
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
			reset_storage: false,
		})
	}

	pub fn nonce(&self, address: H160) -> U256 {
		self.state.get(&address).map(|v| v.basic.nonce)
			.unwrap_or(self.backend.basic(address).nonce)
	}

	fn create_inner(
		&mut self,
		address: H160,
		transfer: Option<Transfer>,
		init_code: Vec<u8>,
		target_gas: Option<usize>,
		take_l64: bool,
		context: Context,
	) -> Capture<ExitReason, Infallible> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => return Capture::Exit(e.into()),
				}
			}
		}

		fn l64(gas: usize) -> usize {
			gas - gas / 64
		}

		if let Some(depth) = self.depth {
			if depth + 1 > self.config.call_limit {
				return Capture::Exit(ExitError::CallTooDeep.into())
			}
		}

		let mut after_gas = self.gasometer.gas();
		if take_l64 && self.config.call_l64_after_gas {
			after_gas = l64(after_gas);
		}
		let target_gas = target_gas.unwrap_or(after_gas);

		let gas_limit = min(after_gas, target_gas);

		let mut substate = self.substate(gas_limit, false);
		{
			if substate.account_mut(address).code.is_none() {
				let code = substate.backend.code(address);
				substate.account_mut(address).code = Some(code.clone());

				if code.len() != 0 {
					return Capture::Exit(ExitError::CreateCollision.into())
				}
			}

			if substate.account_mut(address).basic.nonce != U256::zero() {
				return Capture::Exit(ExitError::CreateCollision.into())
			}

			substate.account_mut(address).reset_storage = true;
			substate.account_mut(address).storage = BTreeMap::new();
		}


		if let Some(transfer) = transfer {
			try_or_fail!(substate.transfer(transfer));
		}

		if self.config.create_increase_nonce {
			substate.account_mut(address).basic.nonce += U256::one();
		}

		let mut runtime = Runtime::new(
			Rc::new(init_code),
			Rc::new(Vec::new()),
			context,
			self.config,
		);

		let reason = substate.execute(&mut runtime);

		match reason {
			ExitReason::Succeed(s) => {
				let out = runtime.machine().return_value();
				match substate.gasometer.record_deposit(out.len()) {
					Ok(()) => {
						let e = self.merge_succeed(substate);
						self.state.entry(address).or_insert(Default::default())
							.code = Some(out);
						try_or_fail!(e);
						Capture::Exit(ExitReason::Succeed(s))
					},
					Err(e) => {
						try_or_fail!(self.merge_fail(substate));
						Capture::Exit(ExitReason::Error(e))
					},
				}
			},
			ExitReason::Error(e) => {
				substate.gasometer.fail();
				try_or_fail!(self.merge_fail(substate));
				Capture::Exit(ExitReason::Error(e))
			},
			ExitReason::Revert(e) => {
				try_or_fail!(self.merge_fail(substate));
				Capture::Exit(ExitReason::Revert(e))
			},
			ExitReason::Fatal(e) => {
				self.gasometer.fail();
				Capture::Exit(ExitReason::Fatal(e))
			},
		}
	}

	fn call_inner(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<usize>,
		is_static: bool,
		take_l64: bool,
		context: Context,
	) -> Capture<(ExitReason, Vec<u8>), Infallible> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => return Capture::Exit((e.into(), Vec::new())),
				}
			}
		}

		fn l64(gas: usize) -> usize {
			gas - gas / 64
		}

		if let Some(depth) = self.depth {
			if depth + 1 > self.config.call_limit {
				return Capture::Exit((ExitError::CallTooDeep.into(), Vec::new()))
			}
		}

		let mut after_gas = self.gasometer.gas();
		if take_l64 && self.config.call_l64_after_gas {
			after_gas = l64(after_gas);
		}
		let target_gas = min(target_gas.unwrap_or(after_gas), after_gas);

		if let Some(ret) = (self.precompile)(code_address, &input, Some(target_gas)) {
			return match ret {
				Ok((s, out, cost)) => {
					try_or_fail!(self.gasometer.record_cost(cost));
					Capture::Exit((ExitReason::Succeed(s), out))
				},
				Err(e) => {
					try_or_fail!(self.gasometer.record_cost(target_gas));
					Capture::Exit((ExitReason::Error(e), Vec::new()))
				},
			}
		}

		let code = self.code(code_address);

		let gas_limit = min(after_gas, target_gas);

		let mut substate = self.substate(gas_limit, is_static);
		if let Some(transfer) = transfer {
			try_or_fail!(substate.transfer(transfer));
		}

		let mut runtime = Runtime::new(
			Rc::new(code),
			Rc::new(input),
			context,
			self.config,
		);

		let reason = substate.execute(&mut runtime);

		match reason {
			ExitReason::Succeed(s) => {
				try_or_fail!(self.merge_succeed(substate));
				Capture::Exit((ExitReason::Succeed(s), runtime.machine().return_value()))
			},
			ExitReason::Error(e) => {
				substate.gasometer.fail();
				try_or_fail!(self.merge_fail(substate));
				Capture::Exit((ExitReason::Error(e), Vec::new()))
			},
			ExitReason::Revert(e) => {
				try_or_fail!(self.merge_fail(substate));
				Capture::Exit((ExitReason::Revert(e), Vec::new()))
			},
			ExitReason::Fatal(e) => {
				self.gasometer.fail();
				Capture::Exit((ExitReason::Fatal(e), Vec::new()))
			},
		}
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
			.and_then(|v| {
				let s = v.storage.get(&index).cloned();

				if v.reset_storage {
					Some(s.unwrap_or(H256::default()))
				} else {
					s
				}

			})
			.unwrap_or(self.backend.storage(address, index))
	}

	fn original_storage(&self, address: H160, index: H256) -> H256 {
		if let Some(account) = self.state.get(&address) {
			if account.reset_storage {
				return H256::default()
			}
		}
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

	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError> {
		if self.account_mut(address).reset_storage {
			if value == H256::default() {
				self.account_mut(address).storage.remove(&index);
			} else {
				self.account_mut(address).storage.insert(index, value);
			}
		} else {
			self.account_mut(address).storage.insert(index, value);
		}

		Ok(())
	}

	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError> {
		self.logs.push(Log {
			address, topics, data
		});

		Ok(())
	}

	fn transfer(&mut self, transfer: Transfer) -> Result<(), ExitError> {
		{
			let source = self.account_mut(transfer.source);
			if source.basic.balance < transfer.value {
				return Err(ExitError::Other("not enough fund"))
			}
			source.basic.balance -= transfer.value;
		}

		{
			let target = self.account_mut(transfer.target);
			target.basic.balance += transfer.value;
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
		transfer: Option<Transfer>,
		init_code: Vec<u8>,
		target_gas: Option<usize>,
		context: Context,
	) -> Capture<ExitReason, Self::CreateInterrupt> {
		self.create_inner(address, transfer, init_code, target_gas, true, context)
	}

	fn call(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<usize>,
		is_static: bool,
		context: Context,
	) -> Capture<(ExitReason, Vec<u8>), Self::CallInterrupt> {
		self.call_inner(code_address, transfer, input, target_gas, is_static, true, context)
	}

	fn pre_validate(
		&mut self,
		context: &Context,
		opcode: Result<Opcode, ExternalOpcode>,
		stack: &Stack
	) -> Result<(), ExitError> {
		let pre_gas = self.gasometer.gas();

		// TODO: Add opcode check.
		let (gas_cost, memory_cost) = gasometer::opcode_cost(
			context.address, opcode, stack, self.is_static, self
		)?;

		self.gasometer.record_opcode(gas_cost, memory_cost)?;

		Ok(())
	}
}
