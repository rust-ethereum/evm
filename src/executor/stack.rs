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

/// Account definition for the stack-based executor.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct StackAccount {
	/// Basic account information, including nonce and balance.
	pub basic: Basic,
	/// Code. `None` means the code is currently unknown.
	pub code: Option<Vec<u8>>,
	/// Storage. Not inserted values mean it is currently known, but not empty.
	pub storage: BTreeMap<H256, H256>,
	/// Whether the storage in the database should be reset before storage
	/// values are applied.
	pub reset_storage: bool,
}

/// Stack-based executor.
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
	/// Create a new stack-based executor.
	pub fn new(
		backend: &'backend B,
		gas_limit: usize,
		config: &'config Config,
	) -> Self {
		Self::new_with_precompile(backend, gas_limit, config, no_precompile)
	}

	/// Create a new stack-based executor with given precompiles.
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

	/// Create a substate executor from the current executor.
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

	/// Execute the runtime until it returns.
	pub fn execute(&mut self, runtime: &mut Runtime) -> ExitReason {
		match runtime.run(self) {
			Capture::Exit(s) => s,
			Capture::Trap(_) => unreachable!("Trap is Infallible"),
		}
	}

	/// Get remaining gas.
	pub fn gas(&self) -> usize {
		self.gasometer.gas()
	}

	/// Merge a substate executor that succeeded.
	pub fn merge_succeed<'obackend, 'oconfig, OB>(
		&mut self,
		mut substate: StackExecutor<'obackend, 'oconfig, OB>
	) -> Result<(), ExitError> {
		self.logs.append(&mut substate.logs);
		self.deleted.append(&mut substate.deleted);
		self.state = substate.state;

		self.gasometer.record_stipend(substate.gasometer.gas())?;
		self.gasometer.record_refund(substate.gasometer.refunded_gas())?;
		Ok(())
	}

	/// Merge a substate executor that reverted.
	pub fn merge_revert<'obackend, 'oconfig, OB>(
		&mut self,
		mut substate: StackExecutor<'obackend, 'oconfig, OB>
	) -> Result<(), ExitError> {
		self.logs.append(&mut substate.logs);

		self.gasometer.record_stipend(substate.gasometer.gas())?;
		Ok(())
	}

	/// Merge a substate executor that failed.
	pub fn merge_fail<'obackend, 'oconfig, OB>(
		&mut self,
		mut substate: StackExecutor<'obackend, 'oconfig, OB>
	) -> Result<(), ExitError> {
		self.logs.append(&mut substate.logs);

		Ok(())
	}

	/// Execute a `CREATE` transaction.
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

		match self.create_inner(
			caller,
			CreateScheme::Legacy { caller },
			value,
			init_code,
			Some(gas_limit),
			false,
		) {
			Capture::Exit((s, _, _)) => s,
			Capture::Trap(_) => unreachable!(),
		}
	}

	/// Execute a `CREATE2` transaction.
	pub fn transact_create2(
		&mut self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		salt: H256,
		gas_limit: usize,
	) -> ExitReason {
		let transaction_cost = gasometer::create_transaction_cost(&init_code);
		match self.gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => return e.into(),
		}
		let code_hash = H256::from_slice(Keccak256::digest(&init_code).as_slice());

		match self.create_inner(
			caller,
			CreateScheme::Create2 { caller, code_hash, salt },
			value,
			init_code,
			Some(gas_limit),
			false,
		) {
			Capture::Exit((s, _, _)) => s,
			Capture::Trap(_) => unreachable!(),
		}
	}

	/// Execute a `CALL` transaction.
	pub fn transact_call(
		&mut self,
		caller: H160,
		address: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: usize,
	) -> (ExitReason, Vec<u8>) {
		let transaction_cost = gasometer::call_transaction_cost(&data);
		match self.gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => return (e.into(), Vec::new()),
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
		}), data, Some(gas_limit), false, false, false, context) {
			Capture::Exit((s, v)) => (s, v),
			Capture::Trap(_) => unreachable!(),
		}
	}

	/// Get used gas for the current executor, given the price.
	pub fn used_gas(
		&self,
	) -> usize {
		self.gasometer.total_used_gas() -
			min(self.gasometer.total_used_gas() / 2, self.gasometer.refunded_gas() as usize)
	}

	/// Get fee needed for the current executor, given the price.
	pub fn fee(
		&self,
		price: U256,
	) -> U256 {
		let used_gas = self.used_gas();
		U256::from(used_gas) * price
	}

	/// Deconstruct the executor, return state to be applied.
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

	/// Get mutable account reference.
	pub fn account_mut(&mut self, address: H160) -> &mut StackAccount {
		self.state.entry(address).or_insert(StackAccount {
			basic: self.backend.basic(address),
			code: None,
			storage: BTreeMap::new(),
			reset_storage: false,
		})
	}

	/// Get account nonce.
	pub fn nonce(&self, address: H160) -> U256 {
		self.state.get(&address).map(|v| v.basic.nonce)
			.unwrap_or(self.backend.basic(address).nonce)
	}

	/// Withdraw balance from address.
	pub fn withdraw(&mut self, address: H160, balance: U256) -> Result<(), ExitError> {
		let source = self.account_mut(address);
		if source.basic.balance < balance {
			return Err(ExitError::OutOfFund.into())
		}
		source.basic.balance -= balance;

		Ok(())
	}

	/// Deposit balance to address.
	pub fn deposit(&mut self, address: H160, balance: U256) {
		let target = self.account_mut(address);
		target.basic.balance += balance;
	}

	/// Transfer balance with the given struct.
	pub fn transfer(&mut self, transfer: Transfer) -> Result<(), ExitError> {
		self.withdraw(transfer.source, transfer.value)?;
		self.deposit(transfer.target, transfer.value);

		Ok(())
	}

	/// Get the create address from given scheme.
	pub fn create_address(&self, scheme: CreateScheme) -> H160 {
		match scheme {
			CreateScheme::Create2 { caller, code_hash, salt } => {
				let mut hasher = Keccak256::new();
				hasher.input(&[0xff]);
				hasher.input(&caller[..]);
				hasher.input(&salt[..]);
				hasher.input(&code_hash[..]);
				H256::from_slice(hasher.result().as_slice()).into()
			},
			CreateScheme::Legacy { caller } => {
				let nonce = self.nonce(caller);
				let mut stream = rlp::RlpStream::new_list(2);
				stream.append(&caller);
				stream.append(&nonce);
				H256::from_slice(Keccak256::digest(&stream.out()).as_slice()).into()
			},
			CreateScheme::Fixed(naddress) => {
				naddress
			},
		}
	}

	fn create_inner(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<usize>,
		take_l64: bool,
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Infallible> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => return Capture::Exit((e.into(), None, Vec::new())),
				}
			}
		}

		fn l64(gas: usize) -> usize {
			gas - gas / 64
		}

		if let Some(depth) = self.depth {
			if depth + 1 > self.config.call_stack_limit {
				return Capture::Exit((ExitError::CallTooDeep.into(), None, Vec::new()))
			}
		}

		if self.balance(caller) < value {
			return Capture::Exit((ExitError::OutOfFund.into(), None, Vec::new()))
		}

		let mut after_gas = self.gasometer.gas();
		if take_l64 && self.config.call_l64_after_gas {
			after_gas = l64(after_gas);
		}
		let target_gas = target_gas.unwrap_or(after_gas);

		let gas_limit = min(after_gas, target_gas);
		try_or_fail!(self.gasometer.record_cost(gas_limit));

		let address = self.create_address(scheme);
		self.account_mut(caller).basic.nonce += U256::one();

		let mut substate = self.substate(gas_limit, false);
		{
			if let Some(code) = substate.account_mut(address).code.as_ref() {
				if code.len() != 0 {
					let _ = self.merge_fail(substate);
					return Capture::Exit((ExitError::CreateCollision.into(), None, Vec::new()))
				}
			} else  {
				let code = substate.backend.code(address);
				substate.account_mut(address).code = Some(code.clone());

				if code.len() != 0 {
					let _ = self.merge_fail(substate);
					return Capture::Exit((ExitError::CreateCollision.into(), None, Vec::new()))
				}
			}

			if substate.account_mut(address).basic.nonce > U256::zero() {
				let _ = self.merge_fail(substate);
				return Capture::Exit((ExitError::CreateCollision.into(), None, Vec::new()))
			}

			substate.account_mut(address).reset_storage = true;
			substate.account_mut(address).storage = BTreeMap::new();
		}

		let context = Context {
			address,
			caller,
			apparent_value: value,
		};
		let transfer = Transfer {
			source: caller,
			target: address,
			value,
		};
		match substate.transfer(transfer) {
			Ok(()) => (),
			Err(e) => {
				let _ = self.merge_revert(substate);
				return Capture::Exit((ExitReason::Error(e), None, Vec::new()))
			},
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

				if let Some(limit) = self.config.create_contract_limit {
					if out.len() > limit {
						substate.gasometer.fail();
						let _ = self.merge_fail(substate);
						return Capture::Exit((ExitError::CreateContractLimit.into(), None, Vec::new()))
					}
				}

				match substate.gasometer.record_deposit(out.len()) {
					Ok(()) => {
						let e = self.merge_succeed(substate);
						self.state.entry(address).or_insert(Default::default())
							.code = Some(out);
						try_or_fail!(e);
						Capture::Exit((ExitReason::Succeed(s), Some(address), Vec::new()))
					},
					Err(e) => {
						let _ = self.merge_fail(substate);
						Capture::Exit((ExitReason::Error(e), None, Vec::new()))
					},
				}
			},
			ExitReason::Error(e) => {
				substate.gasometer.fail();
				let _ = self.merge_fail(substate);
				Capture::Exit((ExitReason::Error(e), None, Vec::new()))
			},
			ExitReason::Revert(e) => {
				let _ = self.merge_revert(substate);
				Capture::Exit((ExitReason::Revert(e), None, runtime.machine().return_value()))
			},
			ExitReason::Fatal(e) => {
				self.gasometer.fail();
				Capture::Exit((ExitReason::Fatal(e), None, Vec::new()))
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
		take_stipend: bool,
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

		let mut after_gas = self.gasometer.gas();
		if take_l64 && self.config.call_l64_after_gas {
			after_gas = l64(after_gas);
		}

		let target_gas = target_gas.unwrap_or(after_gas);
		let mut gas_limit = min(target_gas, after_gas);

		try_or_fail!(self.gasometer.record_cost(gas_limit));

		if let Some(transfer) = transfer.as_ref() {
			if take_stipend && transfer.value != U256::zero() {
				gas_limit = gas_limit.saturating_add(self.config.call_stipend);
			}
		}

		let code = self.code(code_address);

		let mut substate = self.substate(gas_limit, is_static);
		substate.account_mut(context.address);

		if let Some(depth) = self.depth {
			if depth + 1 > self.config.call_stack_limit {
				let _ = self.merge_revert(substate);
				return Capture::Exit((ExitError::CallTooDeep.into(), Vec::new()))
			}
		}

		if let Some(transfer) = transfer {
			match substate.transfer(transfer) {
				Ok(()) => (),
				Err(e) => {
					let _ = self.merge_revert(substate);
					return Capture::Exit((ExitReason::Error(e), Vec::new()))
				},
			}
		}

		if let Some(ret) = (substate.precompile)(code_address, &input, Some(gas_limit)) {
			return match ret {
				Ok((s, out, cost)) => {
					let _ = substate.gasometer.record_cost(cost);
					let _ = self.merge_succeed(substate);
					Capture::Exit((ExitReason::Succeed(s), out))
				},
				Err(e) => {
					let _ = self.merge_fail(substate);
					Capture::Exit((ExitReason::Error(e), Vec::new()))
				},
			}
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
				let _ = self.merge_succeed(substate);
				Capture::Exit((ExitReason::Succeed(s), runtime.machine().return_value()))
			},
			ExitReason::Error(e) => {
				let _ = self.merge_fail(substate);
				Capture::Exit((ExitReason::Error(e), Vec::new()))
			},
			ExitReason::Revert(e) => {
				let _ = self.merge_revert(substate);
				Capture::Exit((ExitReason::Revert(e), runtime.machine().return_value()))
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
		if !self.exists(address) {
			return H256::default()
		}

		let (balance, nonce, code_size) = if let Some(account) = self.state.get(&address) {
			(account.basic.balance, account.basic.nonce,
			 account.code.as_ref().map(|c| U256::from(c.len())).unwrap_or(self.code_size(address)))
		} else {
			let basic = self.backend.basic(address);
			(basic.balance, basic.nonce, U256::from(self.backend.code_size(address)))
		};

		if balance == U256::zero() && nonce == U256::zero() && code_size == U256::zero() {
			return H256::default()
		}

		let value = self.state.get(&address).and_then(|v| {
			v.code.as_ref().map(|c| {
				H256::from_slice(Keccak256::digest(&c).as_slice())
			})
		}).unwrap_or(self.backend.code_hash(address));
		value
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
		if self.config.empty_considered_exists {
			self.state.get(&address).is_some() || self.backend.exists(address)
		} else {
			if let Some(account) = self.state.get(&address) {
				account.basic.nonce != U256::zero() ||
					account.basic.balance != U256::zero() ||
					account.code.as_ref().map(|c| c.len() != 0).unwrap_or(false) ||
					self.backend.code(address).len() != 0
			} else {
				self.backend.basic(address).nonce != U256::zero() ||
					self.backend.basic(address).balance != U256::zero() ||
					self.backend.code(address).len() != 0
			}
		}
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

	fn deleted(&self, address: H160) -> bool { self.deleted.contains(&address) }

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

	fn mark_delete(&mut self, address: H160, target: H160) -> Result<(), ExitError> {
		let balance = self.balance(address);

		self.transfer(Transfer {
			source: address,
			target: target,
			value: balance
		})?;
		self.account_mut(address).basic.balance = U256::zero();

		self.deleted.insert(address);

		Ok(())
	}

	fn create(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<usize>,
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Self::CreateInterrupt> {
		self.create_inner(caller, scheme, value, init_code, target_gas, true)
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
		self.call_inner(code_address, transfer, input, target_gas, is_static, true, true, context)
	}

	fn pre_validate(
		&mut self,
		context: &Context,
		opcode: Result<Opcode, ExternalOpcode>,
		stack: &Stack
	) -> Result<(), ExitError> {
		let (gas_cost, memory_cost) = gasometer::opcode_cost(
			context.address, opcode, stack, self.is_static, &self.config, self
		)?;

		self.gasometer.record_opcode(gas_cost, memory_cost)?;

		Ok(())
	}
}
