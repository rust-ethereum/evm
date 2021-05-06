use core::convert::Infallible;
use primitive_types::{U256, H256, H160};
use sha3::{Keccak256, Digest};
use super::{StackExecutor, StackState, StackExitKind};
use crate::{
	ExitReason, Runtime, ExitError, Stack, Opcode, Capture, Handler, Transfer,
	Context, CreateScheme, ExitSucceed, Config, gasometer,
};

/// Can be injected in `StackExecutor` to inspect contract execution step by
/// step.
pub trait Hook {
	/// Called before the execution of a context.
	fn before_loop<'config, S: StackState<'config>>(
		&mut self,
		executor: &StackExecutor<'config, S>,
		runtime: &Runtime,
	);

	/// Called before each step.
	fn before_step<'config, S: StackState<'config>>(
		&mut self,
		executor: &StackExecutor<'config, S>,
		runtime: &Runtime,
	);

	/// Called after each step. Will not be called if runtime exited
	/// from the loop.
	fn after_step<'config, S: StackState<'config>>(
		&mut self,
		executor: &StackExecutor<'config, S>,
		runtime: &Runtime,
	);

	/// Called after the execution of a context.
	fn after_loop<'config, S: StackState<'config>>(
		&mut self,
		executor: &StackExecutor<'config, S>,
		runtime: &Runtime,
		reason: &ExitReason,
	);
}

impl Hook for () {
	fn before_loop<'config, S: StackState<'config>>(
		&mut self,
		_executor: &StackExecutor<'config, S>,
		_runtime: &Runtime,
	) {
	}

	fn before_step<'config, S: StackState<'config>>(
		&mut self,
		_executor: &StackExecutor<'config, S>,
		_runtime: &Runtime,
	) {
	}

	fn after_step<'config, S: StackState<'config>>(
		&mut self,
		_executor: &StackExecutor<'config, S>,
		_runtime: &Runtime,
	) {
	}

	fn after_loop<'config, S: StackState<'config>>(
		&mut self,
		_executor: &StackExecutor<'config, S>,
		_runtime: &Runtime,
		_reason: &ExitReason,
	) {
	}
}

fn hooked_execute<'config, S: StackState<'config>, H: Hook>(
	executor: &mut StackExecutor<'config, S>,
	runtime: &mut Runtime,
	hook: &mut H,
) -> ExitReason {
	hook.before_loop(executor, runtime);

	let reason = loop {
		hook.before_step(executor, runtime);

		match runtime.step(executor) {
			Ok(_) => {}
			Err(Capture::Exit(s)) => break s,
			Err(Capture::Trap(_)) => unreachable!("Trap is Infallible"),
		}

		hook.after_step(executor, runtime);
	};

	hook.after_loop(executor, runtime, &reason);

	reason
}

pub struct HookedStackExecutor<'config, S, H> {
	executor: StackExecutor<'config, S>,
	hook: H,
}

impl<'config, S: StackState<'config>, H: Hook> HookedStackExecutor<'config, S, H> {
	/// Create a new stack-based executor.
	pub fn new(state: S, config: &'config Config, hook: H) -> Self {
		Self {
			executor: StackExecutor::new(state, config),
			hook,
		}
	}

	/// Create a new stack-based executor with given precompiles.
	pub fn new_with_precompile(
		state: S,
		config: &'config Config,
		hook: H,
		precompile: fn(
			H160,
			&[u8],
			Option<u64>,
			&Context,
		) -> Option<Result<(ExitSucceed, Vec<u8>, u64), ExitError>>,
	) -> Self {
		Self {
			executor: StackExecutor::new_with_precompile(state, config, precompile),
			hook,
		}
	}

	/// Return a reference of the Config.
	pub fn config(&self) -> &'config Config {
		self.executor.config()
	}

	pub fn state(&self) -> &S {
		self.executor.state()
	}

	pub fn state_mut(&mut self) -> &mut S {
		self.executor.state_mut()
	}

	pub fn into_state(self) -> S {
		self.executor.into_state()
	}

	/// Create a substate executor from the current executor.
	pub fn enter_substate(&mut self, gas_limit: u64, is_static: bool) {
		self.executor.enter_substate(gas_limit, is_static)
	}

	/// Exit a substate. Panic if it results an empty substate stack.
	pub fn exit_substate(&mut self, kind: StackExitKind) -> Result<(), ExitError> {
		self.executor.exit_substate(kind)
	}

	/// Execute the runtime until it returns.
	pub fn execute(&mut self, runtime: &mut Runtime) -> ExitReason {
		hooked_execute(&mut self.executor, runtime, &mut self.hook)
	}

	/// Get remaining gas.
	pub fn gas(&self) -> u64 {
		self.executor.gas()
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
		match self.executor.state.metadata_mut().gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => return e.into(),
		}

		let hook = &mut self.hook;
		match super::create_inner(
			&mut self.executor,
			caller,
			CreateScheme::Legacy { caller },
			value,
			init_code,
			Some(gas_limit),
			false,
			|executor, runtime| {
				hooked_execute(executor, runtime, hook)
			}
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
		match self.executor.state.metadata_mut().gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => return e.into(),
		}
		let code_hash = H256::from_slice(Keccak256::digest(&init_code).as_slice());

		let hook = &mut self.hook;
		match super::create_inner(
			&mut self.executor,
			caller,
			CreateScheme::Create2 { caller, code_hash, salt },
			value,
			init_code,
			Some(gas_limit),
			false,
			|executor, runtime| {
				hooked_execute(executor, runtime, hook)
			}
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
		gas_limit: u64,
	) -> (ExitReason, Vec<u8>) {
		let transaction_cost = gasometer::call_transaction_cost(&data);
		match self.executor.state.metadata_mut().gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => return (e.into(), Vec::new()),
		}

		self.executor.state.inc_nonce(caller);

		let context = Context {
			caller,
			address,
			apparent_value: value,
		};

		let hook = &mut self.hook;
		match super::call_inner(
			&mut self.executor,
			address,
			Some(Transfer {
				source: caller,
				target: address,
				value
			}),
			data,
			Some(gas_limit),
			false,
			false,
			false,
			context,
			|executor, runtime| {
				hooked_execute(executor, runtime, hook)
			}
		) {
			Capture::Exit((s, v)) => (s, v),
			Capture::Trap(_) => unreachable!(),
		}
	}

	/// Get used gas for the current executor, given the price.
	pub fn used_gas(&self) -> u64 {
		self.executor.used_gas()
	}

	/// Get fee needed for the current executor, given the price.
	pub fn fee(&self, price: U256) -> U256 {
		self.executor.fee(price)
	}

	/// Get account nonce.
	pub fn nonce(&self, address: H160) -> U256 {
		self.executor.nonce(address)
	}

	/// Get the create address from given scheme.
	pub fn create_address(&self, scheme: CreateScheme) -> H160 {
		self.executor.create_address(scheme)
	}
}

impl<'config, S: StackState<'config>, H: Hook> Handler for HookedStackExecutor<'config, S, H> {
	type CreateInterrupt = Infallible;
	type CreateFeedback = Infallible;
	type CallInterrupt = Infallible;
	type CallFeedback = Infallible;

	fn balance(&self, address: H160) -> U256 { self.executor.balance(address) }
	fn code_size(&self, address: H160) -> U256 { self.executor.code_size(address) }
	fn code_hash(&self, address: H160) -> H256 { self.executor.code_hash(address) }
	fn code(&self, address: H160) -> Vec<u8> { self.executor.code(address) }
	fn storage(&self, address: H160, index: H256) -> H256 { self.executor.storage(address, index) }
	fn original_storage(&self, address: H160, index: H256) -> H256 { self.executor.original_storage(address, index) }
	fn exists(&self, address: H160) -> bool { self.executor.exists(address) }
	fn gas_left(&self) -> U256 { self.executor.gas_left() }
	fn gas_price(&self) -> U256 { self.executor.gas_price() }
	fn origin(&self) -> H160 { self.executor.origin() }
	fn block_hash(&self, number: U256) -> H256 { self.executor.block_hash(number) }
	fn block_number(&self) -> U256 { self.executor.block_number() }
	fn block_coinbase(&self) -> H160 { self.executor.block_coinbase() }
	fn block_timestamp(&self) -> U256 { self.executor.block_timestamp() }
	fn block_difficulty(&self) -> U256 { self.executor.block_difficulty() }
	fn block_gas_limit(&self) -> U256 { self.executor.block_gas_limit() }
	fn chain_id(&self) -> U256 { self.executor.chain_id() }
	fn deleted(&self, address: H160) -> bool { self.executor.deleted(address) }

	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError> {
		self.executor.set_storage(address, index, value)
	}

	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError> {
		self.executor.log(address, topics, data)
	}

	fn mark_delete(&mut self, address: H160, target: H160) -> Result<(), ExitError> {
		self.executor.mark_delete(address, target)
	}

	fn create(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<u64>,
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Self::CreateInterrupt> {
		let hook = &mut self.hook;
		super::create_inner(&mut self.executor, caller, scheme, value, init_code, target_gas, true, |executor, runtime| {
			hooked_execute(executor, runtime, hook)
		})
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
		let hook = &mut self.hook;
		super::call_inner(&mut self.executor, code_address, transfer, input, target_gas, is_static, true, true, context, |executor, runtime| {
			hooked_execute(executor, runtime, hook)
		})
	}

	#[inline]
	fn pre_validate(
		&mut self,
		context: &Context,
		opcode: Opcode,
		stack: &Stack,
	) -> Result<(), ExitError> {
		self.executor.pre_validate(context, opcode, stack)
	}
}
