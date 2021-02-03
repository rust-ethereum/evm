mod state;

pub use self::state::MemoryStackSubstate;

use core::{convert::Infallible, cmp::min};
use alloc::{rc::Rc, vec, vec::Vec, collections::{BTreeMap, BTreeSet}};
use primitive_types::{U256, H256, H160};
use sha3::{Keccak256, Digest};
use crate::{ExitError, Stack, ExternalOpcode, Opcode, Capture, Handler, Transfer,
			Context, CreateScheme, Runtime, ExitReason, ExitSucceed, Config};
use crate::backend::{Log, Basic, Apply, Backend};
use crate::gasometer::{self, Gasometer};

pub enum StackExitKind {
	Succeeded,
	Reverted,
	Failed,
}

pub struct StackSubstateMetadata<'config> {
	gasometer: Gasometer<'config>,
	is_static: bool,
	depth: Option<usize>,
}

impl<'config> StackSubstateMetadata<'config> {
	pub fn swallow_commit(&mut self, other: Self) -> Result<(), ExitError> {
		self.gasometer.record_stipend(other.gasometer.gas())?;
		self.gasometer.record_refund(other.gasometer.refunded_gas())?;

		Ok(())
	}

	pub fn swallow_revert(&mut self, other: Self) -> Result<(), ExitError> {
		self.gasometer.record_stipend(other.gasometer.gas())?;

		Ok(())
	}

	pub fn swallow_discard(&mut self, other: Self) -> Result<(), ExitError> {
		Ok(())
	}

	pub fn spit_child(&self, gas_limit: u64, is_static: bool) -> Self {
		Self {
			gasometer: Gasometer::new(gas_limit, self.gasometer.config()),
			is_static: is_static || self.is_static,
			depth: match self.depth {
				None => Some(0),
				Some(n) => Some(n + 1),
			},
		}
	}
}

/// Stack-based executor.
pub struct StackExecutor<'backend, 'config, B> {
	backend: &'backend B,
	config: &'config Config,
	precompile: fn(H160, &[u8], Option<u64>, &Context) -> Option<Result<(ExitSucceed, Vec<u8>, u64), ExitError>>,
	substate: MemoryStackSubstate<'config>,
}

fn no_precompile(
	_address: H160,
	_input: &[u8],
	_target_gas: Option<u64>,
	_context: &Context,
) -> Option<Result<(ExitSucceed, Vec<u8>, u64), ExitError>> {
	None
}

impl<'backend, 'config, B: Backend> StackExecutor<'backend, 'config, B> {
	/// Create a new stack-based executor.
	pub fn new(
		backend: &'backend B,
		gas_limit: u64,
		config: &'config Config,
	) -> Self {
		Self::new_with_precompile(backend, gas_limit, config, no_precompile)
	}

	/// Create a new stack-based executor with given precompiles.
	pub fn new_with_precompile(
		backend: &'backend B,
		gas_limit: u64,
		config: &'config Config,
		precompile: fn(H160, &[u8], Option<u64>, &Context) -> Option<Result<(ExitSucceed, Vec<u8>, u64), ExitError>>,
	) -> Self {
		Self {
			backend,
			config,
			precompile,
			substate: MemoryStackSubstate::new(StackSubstateMetadata {
				gasometer: Gasometer::new(gas_limit, config),
				is_static: false,
				depth: None,
			}),
		}
	}

	/// Create a substate executor from the current executor.
	pub fn enter_substate(
		&mut self,
		gas_limit: u64,
		is_static: bool,
	) {
		self.substate.enter(gas_limit, is_static);
	}

	/// Exit a substate. Panic if it results an empty substate stack.
	pub fn exit_substate(
		&mut self,
		kind: StackExitKind,
	) -> Result<(), ExitError> {
		match kind {
			StackExitKind::Succeeded => self.substate.exit_commit(),
			StackExitKind::Reverted => self.substate.exit_revert(),
			StackExitKind::Failed => self.substate.exit_discard(),
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
	pub fn gas(&self) -> u64 {
		self.substate.metadata().gasometer.gas()
	}

	/// Execute a `CREATE` transaction.
	pub fn transact_create(
		&mut self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		gas_limit: u64,
	) -> ExitReason {
		let transaction_cost = gasometer::create_transaction_cost(&init_code);
		match self.substate.metadata_mut().gasometer.record_transaction(transaction_cost) {
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
		gas_limit: u64,
	) -> ExitReason {
		let transaction_cost = gasometer::create_transaction_cost(&init_code);
		match self.substate.metadata_mut().gasometer.record_transaction(transaction_cost) {
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

// 	/// Execute a `CALL` transaction.
// 	pub fn transact_call(
// 		&mut self,
// 		caller: H160,
// 		address: H160,
// 		value: U256,
// 		data: Vec<u8>,
// 		gas_limit: u64,
// 	) -> (ExitReason, Vec<u8>) {
// 		let current = self.substates.last_mut()
// 			.expect("substate vec always have length greater than one; qed");

// 		let transaction_cost = gasometer::call_transaction_cost(&data);
// 		match current.gasometer.record_transaction(transaction_cost) {
// 			Ok(()) => (),
// 			Err(e) => return (e.into(), Vec::new()),
// 		}

// 		self.account_mut(caller).basic.nonce += U256::one();

// 		let context = Context {
// 			caller,
// 			address,
// 			apparent_value: value,
// 		};

// 		match self.call_inner(address, Some(Transfer {
// 			source: caller,
// 			target: address,
// 			value
// 		}), data, Some(gas_limit), false, false, false, context) {
// 			Capture::Exit((s, v)) => (s, v),
// 			Capture::Trap(_) => unreachable!(),
// 		}
// 	}

	/// Get used gas for the current executor, given the price.
	pub fn used_gas(
		&self,
	) -> u64 {
		self.substate.metadata().gasometer.total_used_gas() -
			min(self.substate.metadata().gasometer.total_used_gas() / 2,
				self.substate.metadata().gasometer.refunded_gas() as u64)
	}

	/// Get fee needed for the current executor, given the price.
	pub fn fee(
		&self,
		price: U256,
	) -> U256 {
		let used_gas = self.used_gas();
		U256::from(used_gas) * price
	}

// 	/// Deconstruct the executor, return state to be applied. Panic if the
// 	/// executor is not in the top-level substate.
// 	#[must_use]
// 	pub fn deconstruct(
// 		mut self
// 	) -> (impl IntoIterator<Item=Apply<impl IntoIterator<Item=(H256, H256)>>>,
// 		  impl IntoIterator<Item=Log>)
// 	{
// 		assert_eq!(self.substates.len(), 1);

// 		let current = self.substates.pop()
// 			.expect("substate vec always have length greater than one; qed");

// 		let applies = current.state.deconstruct();
// 		let logs = current.logs;

// 		(applies, logs)
// 	}

	/// Get account nonce.
	pub fn nonce(&self, address: H160) -> U256 {
		self.substate.known_basic(address).map(|acc| acc.nonce)
			.unwrap_or_else(|| self.backend.basic(address).nonce)
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
		target_gas: Option<u64>,
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

		fn l64(gas: u64) -> u64 {
			gas - gas / 64
		}

		if let Some(depth) = self.substate.metadata().depth {
			if depth > self.config.call_stack_limit {
				return Capture::Exit((ExitError::CallTooDeep.into(), None, Vec::new()))
			}
		}

		if self.balance(caller) < value {
			return Capture::Exit((ExitError::OutOfFund.into(), None, Vec::new()))
		}

		let after_gas = if take_l64 && self.config.call_l64_after_gas {
			if self.config.estimate {
				let initial_after_gas = self.substate.metadata().gasometer.gas();
				let diff = initial_after_gas - l64(initial_after_gas);
				try_or_fail!(self.substate.metadata_mut().gasometer.record_cost(diff));
				self.substate.metadata().gasometer.gas()
			} else {
				l64(self.substate.metadata().gasometer.gas())
			}
		} else {
			self.substate.metadata().gasometer.gas()
		};

		let target_gas = target_gas.unwrap_or(after_gas);

		let gas_limit = min(after_gas, target_gas);
		try_or_fail!(
			self.substate.metadata_mut().gasometer.record_cost(gas_limit)
		);

		let address = self.create_address(scheme);
		self.substate.inc_nonce(caller, self.backend);

		self.enter_substate(gas_limit, false);

		{
			if self.code_size(address) != U256::zero() {
				let _ = self.exit_substate(StackExitKind::Failed);
				return Capture::Exit((ExitError::CreateCollision.into(), None, Vec::new()))
			}

			if self.nonce(address) > U256::zero() {
				let _ = self.exit_substate(StackExitKind::Failed);
				return Capture::Exit((ExitError::CreateCollision.into(), None, Vec::new()))
			}

			self.substate.reset_storage(address, self.backend);
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
		match self.substate.transfer(transfer, self.backend) {
			Ok(()) => (),
			Err(e) => {
				let _ = self.exit_substate(StackExitKind::Reverted);
				return Capture::Exit((ExitReason::Error(e), None, Vec::new()))
			},
		}

		if self.config.create_increase_nonce {
			self.substate.inc_nonce(address, self.backend);
		}

		let mut runtime = Runtime::new(
			Rc::new(init_code),
			Rc::new(Vec::new()),
			context,
			self.config,
		);

		let reason = self.execute(&mut runtime);
		log::debug!(target: "evm", "Create execution using address {}: {:?}", address, reason);

		match reason {
			ExitReason::Succeed(s) => {
				let out = runtime.machine().return_value();

				if let Some(limit) = self.config.create_contract_limit {
					if out.len() > limit {
						self.substate.metadata_mut().gasometer.fail();
						let _ = self.exit_substate(StackExitKind::Failed);
						return Capture::Exit((ExitError::CreateContractLimit.into(), None, Vec::new()))
					}
				}

				match self.substate.metadata_mut().gasometer.record_deposit(out.len()) {
					Ok(()) => {
						let e = self.exit_substate(StackExitKind::Succeeded);
						self.substate.set_code(address, out, self.backend);
						try_or_fail!(e);
						Capture::Exit((ExitReason::Succeed(s), Some(address), Vec::new()))
					},
					Err(e) => {
						let _ = self.exit_substate(StackExitKind::Failed);
						Capture::Exit((ExitReason::Error(e), None, Vec::new()))
					},
				}
			},
			ExitReason::Error(e) => {
				self.substate.metadata_mut().gasometer.fail();
				let _ = self.exit_substate(StackExitKind::Failed);
				Capture::Exit((ExitReason::Error(e), None, Vec::new()))
			},
			ExitReason::Revert(e) => {
				let _ = self.exit_substate(StackExitKind::Reverted);
				Capture::Exit((ExitReason::Revert(e), None, runtime.machine().return_value()))
			},
			ExitReason::Fatal(e) => {
				self.substate.metadata_mut().gasometer.fail();
				let _ = self.exit_substate(StackExitKind::Failed);
				Capture::Exit((ExitReason::Fatal(e), None, Vec::new()))
			},
		}
	}

	fn call_inner(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<u64>,
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

		fn l64(gas: u64) -> u64 {
			gas - gas / 64
		}

		let after_gas = if take_l64 && self.config.call_l64_after_gas {
			if self.config.estimate {
				let initial_after_gas = self.substate.metadata().gasometer.gas();
				let diff = initial_after_gas - l64(initial_after_gas);
				try_or_fail!(self.substate.metadata_mut().gasometer.record_cost(diff));
				self.substate.metadata().gasometer.gas()
			} else {
				l64(self.substate.metadata().gasometer.gas())
			}
		} else {
			self.substate.metadata().gasometer.gas()
		};

		let target_gas = target_gas.unwrap_or(after_gas);
		let mut gas_limit = min(target_gas, after_gas);

		try_or_fail!(
			self.substate.metadata_mut().gasometer.record_cost(gas_limit)
		);

		if let Some(transfer) = transfer.as_ref() {
			if take_stipend && transfer.value != U256::zero() {
				gas_limit = gas_limit.saturating_add(self.config.call_stipend);
			}
		}

		let code = self.code(code_address);

		self.enter_substate(gas_limit, is_static);
		self.substate.touch(context.address, self.backend);

		if let Some(depth) = self.substate.metadata().depth {
			if depth > self.config.call_stack_limit {
				let _ = self.exit_substate(StackExitKind::Reverted);
				return Capture::Exit((ExitError::CallTooDeep.into(), Vec::new()))
			}
		}

		if let Some(transfer) = transfer {
			match self.substate.transfer(transfer, self.backend) {
				Ok(()) => (),
				Err(e) => {
					let _ = self.exit_substate(StackExitKind::Reverted);
					return Capture::Exit((ExitReason::Error(e), Vec::new()))
				},
			}
		}

		if let Some(ret) = (self.precompile)(code_address, &input, Some(gas_limit), &context) {
			return match ret {
				Ok((s, out, cost)) => {
					let _ = self.substate.metadata_mut().gasometer.record_cost(cost);
					let _ = self.exit_substate(StackExitKind::Succeeded);
					Capture::Exit((ExitReason::Succeed(s), out))
				},
				Err(e) => {
					let _ = self.exit_substate(StackExitKind::Failed);
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

		let reason = self.execute(&mut runtime);
		log::debug!(target: "evm", "Call execution using address {}: {:?}", code_address, reason);

		match reason {
			ExitReason::Succeed(s) => {
				let _ = self.exit_substate(StackExitKind::Succeeded);
				Capture::Exit((ExitReason::Succeed(s), runtime.machine().return_value()))
			},
			ExitReason::Error(e) => {
				let _ = self.exit_substate(StackExitKind::Failed);
				Capture::Exit((ExitReason::Error(e), Vec::new()))
			},
			ExitReason::Revert(e) => {
				let _ = self.exit_substate(StackExitKind::Reverted);
				Capture::Exit((ExitReason::Revert(e), runtime.machine().return_value()))
			},
			ExitReason::Fatal(e) => {
				self.substate.metadata_mut().gasometer.fail();
				let _ = self.exit_substate(StackExitKind::Failed);
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
		self.substate.known_basic(address).map(|basic| basic.balance).unwrap_or_else(|| {
			self.backend.basic(address).balance
		})
	}

	fn code_size(&self, address: H160) -> U256 {
		self.substate.known_code(address).map(|code| U256::from(code.len())).unwrap_or_else(|| {
			U256::from(self.backend.code_size(address))
		})
	}

	fn code_hash(&self, address: H160) -> H256 {
		if self.code_size(address) == U256::zero() {
			return H256::default()
		}

		self.substate.known_code(address).map(|code| H256::from_slice(Keccak256::digest(&code).as_slice()))
			.unwrap_or_else(|| self.backend.code_hash(address))
	}

	fn code(&self, address: H160) -> Vec<u8> {
		self.substate.known_code(address).map(|code| code.clone())
			.unwrap_or_else(|| self.backend.code(address))
	}

	fn storage(&self, address: H160, index: H256) -> H256 {
		self.substate.known_storage(address, index)
			.unwrap_or_else(|| self.backend.storage(address, index))
	}

	fn original_storage(&self, address: H160, index: H256) -> H256 {
		self.substate.known_original_storage(address, index)
			.unwrap_or_else(|| self.backend.original_storage(address, index).unwrap_or_default())
	}

	fn exists(&self, address: H160) -> bool {
		if self.config.empty_considered_exists {
			match self.substate.known_empty(address) {
				Some(true) => true,
				Some(false) => true,
				None => self.backend.exists(address),
			}
		} else {
			match self.substate.known_empty(address) {
				Some(true) => false,
				Some(false) => true,
				None => !(
					self.backend.basic(address).balance == U256::zero() &&
						self.backend.basic(address).nonce == U256::zero() &&
						self.backend.code(address).len() == 0
				),
			}
		}
	}

	fn gas_left(&self) -> U256 {
		U256::from(self.substate.metadata().gasometer.gas())
	}

	fn gas_price(&self) -> U256 { self.backend.gas_price() }
	fn origin(&self) -> H160 { self.backend.origin() }
	fn block_hash(&self, number: U256) -> H256 { self.backend.block_hash(number) }
	fn block_number(&self) -> U256 { self.backend.block_number() }
	fn block_coinbase(&self) -> H160 { self.backend.block_coinbase() }
	fn block_timestamp(&self) -> U256 { self.backend.block_timestamp() }
	fn block_difficulty(&self) -> U256 { self.backend.block_difficulty() }
	fn block_gas_limit(&self) -> U256 { self.backend.block_gas_limit() }
	fn chain_id(&self) -> U256 { self.backend.chain_id() }

	fn deleted(&self, address: H160) -> bool {
		self.substate.deleted(address)
	}

	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError> {
		self.substate.set_storage(address, index, value);
		Ok(())
	}

	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError> {
		self.substate.log(address, topics, data);
		Ok(())
	}

	fn mark_delete(&mut self, address: H160, target: H160) -> Result<(), ExitError> {
		let balance = self.balance(address);

		self.substate.transfer(Transfer {
			source: address,
			target: target,
			value: balance,
		}, self.backend);
		self.substate.reset_balance(address, self.backend);
		self.substate.set_deleted(address);

		Ok(())
	}

	fn create(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<u64>,
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Self::CreateInterrupt> {
		self.create_inner(caller, scheme, value, init_code, target_gas, true)
	}

	fn call(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<u64>,
		is_static: bool,
		context: Context,
	) -> Capture<(ExitReason, Vec<u8>), Self::CallInterrupt> {
		unimplemented!()
		// self.call_inner(code_address, transfer, input, target_gas, is_static, true, true, context)
	}

	fn pre_validate(
		&mut self,
		context: &Context,
		opcode: Result<Opcode, ExternalOpcode>,
		stack: &Stack
	) -> Result<(), ExitError> {
		let is_static = self.substate.metadata().is_static;
		let (gas_cost, memory_cost) = gasometer::opcode_cost(
			context.address, opcode, stack, is_static, &self.config, self
		)?;

		let gasometer = &mut self.substate.metadata_mut().gasometer;

		log::trace!(target: "evm", "Running opcode: {:?}, Pre gas-left: {:?}", opcode, gasometer.gas());

		gasometer.record_opcode(gas_cost, memory_cost)?;

		Ok(())
	}
}
