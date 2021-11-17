use crate::backend::Backend;
use crate::gasometer::{self, Gasometer, StorageTarget};
use crate::{
	Capture, Config, Context, CreateScheme, ExitError, ExitReason, ExitSucceed, Handler, Opcode,
	Runtime, Stack, Transfer,
};
use alloc::{
	collections::{BTreeMap, BTreeSet},
	rc::Rc,
	vec::Vec,
};
use core::{cmp::min, convert::Infallible};
use ethereum::Log;
use evm_core::{ExitFatal, ExitRevert};
use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

macro_rules! emit_exit {
	($reason:expr) => {{
		let reason = $reason;
		event!(Exit {
			reason: &reason,
			return_value: &Vec::new(),
		});
		reason
	}};
	($reason:expr, $return_value:expr) => {{
		let reason = $reason;
		let return_value = $return_value;
		event!(Exit {
			reason: &reason,
			return_value: &return_value,
		});
		(reason, return_value)
	}};
}

pub enum StackExitKind {
	Succeeded,
	Reverted,
	Failed,
}

#[derive(Default, Clone, Debug)]
pub struct Accessed {
	pub accessed_addresses: BTreeSet<H160>,
	pub accessed_storage: BTreeSet<(H160, H256)>,
}

impl Accessed {
	pub fn access_address(&mut self, address: H160) {
		self.accessed_addresses.insert(address);
	}

	pub fn access_addresses<I>(&mut self, addresses: I)
	where
		I: Iterator<Item = H160>,
	{
		for address in addresses {
			self.accessed_addresses.insert(address);
		}
	}

	pub fn access_storages<I>(&mut self, storages: I)
	where
		I: Iterator<Item = (H160, H256)>,
	{
		for storage in storages {
			self.accessed_storage.insert((storage.0, storage.1));
		}
	}
}

#[derive(Clone, Debug)]
pub struct StackSubstateMetadata<'config> {
	gasometer: Gasometer<'config>,
	is_static: bool,
	depth: Option<usize>,
	accessed: Option<Accessed>,
}

impl<'config> StackSubstateMetadata<'config> {
	pub fn new(gas_limit: u64, config: &'config Config) -> Self {
		let accessed = if config.increase_state_access_gas {
			Some(Accessed::default())
		} else {
			None
		};
		Self {
			gasometer: Gasometer::new(gas_limit, config),
			is_static: false,
			depth: None,
			accessed,
		}
	}

	pub fn swallow_commit(&mut self, other: Self) -> Result<(), ExitError> {
		self.gasometer.record_stipend(other.gasometer.gas())?;
		self.gasometer
			.record_refund(other.gasometer.refunded_gas())?;

		if let (Some(mut other_accessed), Some(self_accessed)) =
			(other.accessed, self.accessed.as_mut())
		{
			self_accessed
				.accessed_addresses
				.append(&mut other_accessed.accessed_addresses);
			self_accessed
				.accessed_storage
				.append(&mut other_accessed.accessed_storage);
		}

		Ok(())
	}

	pub fn swallow_revert(&mut self, other: Self) -> Result<(), ExitError> {
		self.gasometer.record_stipend(other.gasometer.gas())?;

		Ok(())
	}

	pub fn swallow_discard(&mut self, _other: Self) -> Result<(), ExitError> {
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
			accessed: self.accessed.as_ref().map(|_| Accessed::default()),
		}
	}

	pub fn gasometer(&self) -> &Gasometer<'config> {
		&self.gasometer
	}

	pub fn gasometer_mut(&mut self) -> &mut Gasometer<'config> {
		&mut self.gasometer
	}

	pub fn is_static(&self) -> bool {
		self.is_static
	}

	pub fn depth(&self) -> Option<usize> {
		self.depth
	}

	pub fn access_address(&mut self, address: H160) {
		if let Some(accessed) = &mut self.accessed {
			accessed.access_address(address)
		}
	}

	pub fn access_addresses<I>(&mut self, addresses: I)
	where
		I: Iterator<Item = H160>,
	{
		if let Some(accessed) = &mut self.accessed {
			accessed.access_addresses(addresses);
		}
	}

	pub fn access_storage(&mut self, address: H160, key: H256) {
		if let Some(accessed) = &mut self.accessed {
			accessed.accessed_storage.insert((address, key));
		}
	}

	pub fn access_storages<I>(&mut self, storages: I)
	where
		I: Iterator<Item = (H160, H256)>,
	{
		if let Some(accessed) = &mut self.accessed {
			accessed.access_storages(storages);
		}
	}

	pub fn accessed(&self) -> &Option<Accessed> {
		&self.accessed
	}
}

#[auto_impl::auto_impl(&mut, Box)]
pub trait StackState<'config>: Backend {
	fn metadata(&self) -> &StackSubstateMetadata<'config>;
	fn metadata_mut(&mut self) -> &mut StackSubstateMetadata<'config>;

	fn enter(&mut self, gas_limit: u64, is_static: bool);
	fn exit_commit(&mut self) -> Result<(), ExitError>;
	fn exit_revert(&mut self) -> Result<(), ExitError>;
	fn exit_discard(&mut self) -> Result<(), ExitError>;

	fn is_empty(&self, address: H160) -> bool;
	fn deleted(&self, address: H160) -> bool;
	fn is_cold(&self, address: H160) -> bool;
	fn is_storage_cold(&self, address: H160, key: H256) -> bool;

	fn inc_nonce(&mut self, address: H160);
	fn set_storage(&mut self, address: H160, key: H256, value: H256);
	fn reset_storage(&mut self, address: H160);
	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>);
	fn set_deleted(&mut self, address: H160);
	fn set_code(&mut self, address: H160, code: Vec<u8>);
	fn transfer(&mut self, transfer: Transfer) -> Result<(), ExitError>;
	fn reset_balance(&mut self, address: H160);
	fn touch(&mut self, address: H160);
}

/// Data returned by a precompile on success.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PrecompileOutput {
	pub exit_status: ExitSucceed,
	pub cost: u64,
	pub output: Vec<u8>,
	pub logs: Vec<Log>,
}

/// Data returned by a precompile in case of failure.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PrecompileFailure {
	/// Reverts the state changes and consume all the gas.
	Error { exit_status: ExitError },
	/// Reverts the state changes and consume the provided `cost`.
	/// Returns the provided error message.
	Revert {
		exit_status: ExitRevert,
		output: Vec<u8>,
		cost: u64,
	},
	/// Mark this failure as fatal, and all EVM execution stacks must be exited.
	Fatal { exit_status: ExitFatal },
}

/// A precompile result.
pub type PrecompileResult = Result<PrecompileOutput, PrecompileFailure>;

/// A set of precompiles.
/// Checks of the provided address being in the precompile set should be
/// as cheap as possible since it may be called often.
pub trait PrecompileSet {
	/// Tries to execute a precompile in the precompile set.
	/// If the provided address is not a precompile, returns None.
	fn execute(
		&self,
		address: H160,
		input: &[u8],
		gas_limit: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> Option<PrecompileResult>;

	/// Check if the given address is a precompile. Should only be called to
	/// perform the check while not executing the precompile afterward, since
	/// `execute` already performs a check internally.
	fn is_precompile(&self, address: H160) -> bool;
}

impl PrecompileSet for () {
	fn execute(
		&self,
		_: H160,
		_: &[u8],
		_: Option<u64>,
		_: &Context,
		_: bool,
	) -> Option<PrecompileResult> {
		None
	}

	fn is_precompile(&self, _: H160) -> bool {
		false
	}
}

/// Precompiles function signature. Expected input arguments are:
///  * Input
///  * Gas limit
///  * Context
///  * Is static
pub type PrecompileFn = fn(&[u8], Option<u64>, &Context, bool) -> PrecompileResult;

impl PrecompileSet for BTreeMap<H160, PrecompileFn> {
	fn execute(
		&self,
		address: H160,
		input: &[u8],
		gas_limit: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> Option<PrecompileResult> {
		self.get(&address)
			.map(|precompile| (*precompile)(input, gas_limit, context, is_static))
	}

	/// Check if the given address is a precompile. Should only be called to
	/// perform the check while not executing the precompile afterward, since
	/// `execute` already performs a check internally.
	fn is_precompile(&self, address: H160) -> bool {
		self.contains_key(&address)
	}
}

/// Stack-based executor.
pub struct StackExecutor<'config, 'precompiles, S, P> {
	config: &'config Config,
	state: S,
	precompile_set: &'precompiles P,
}

impl<'config, 'precompiles, S: StackState<'config>, P: PrecompileSet>
	StackExecutor<'config, 'precompiles, S, P>
{
	/// Return a reference of the Config.
	pub fn config(&self) -> &'config Config {
		self.config
	}

	/// Return a reference to the precompile set.
	pub fn precompiles(&self) -> &'precompiles P {
		self.precompile_set
	}

	/// Create a new stack-based executor with given precompiles.
	pub fn new_with_precompiles(
		state: S,
		config: &'config Config,
		precompile_set: &'precompiles P,
	) -> Self {
		Self {
			config,
			state,
			precompile_set,
		}
	}

	pub fn state(&self) -> &S {
		&self.state
	}

	pub fn state_mut(&mut self) -> &mut S {
		&mut self.state
	}

	pub fn into_state(self) -> S {
		self.state
	}

	/// Create a substate executor from the current executor.
	pub fn enter_substate(&mut self, gas_limit: u64, is_static: bool) {
		self.state.enter(gas_limit, is_static);
	}

	/// Exit a substate. Panic if it results an empty substate stack.
	pub fn exit_substate(&mut self, kind: StackExitKind) -> Result<(), ExitError> {
		match kind {
			StackExitKind::Succeeded => self.state.exit_commit(),
			StackExitKind::Reverted => self.state.exit_revert(),
			StackExitKind::Failed => self.state.exit_discard(),
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
		self.state.metadata().gasometer.gas()
	}

	fn record_create_transaction_cost(
		&mut self,
		init_code: &[u8],
		access_list: &[(H160, Vec<H256>)],
	) -> Result<(), ExitError> {
		let transaction_cost = gasometer::create_transaction_cost(init_code, access_list);
		let gasometer = &mut self.state.metadata_mut().gasometer;
		gasometer.record_transaction(transaction_cost)
	}

	/// Execute a `CREATE` transaction.
	pub fn transact_create(
		&mut self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		gas_limit: u64,
		access_list: Vec<(H160, Vec<H256>)>, // See EIP-2930
	) -> ExitReason {
		event!(TransactCreate {
			caller,
			value,
			init_code: &init_code,
			gas_limit,
			address: self.create_address(CreateScheme::Legacy { caller }),
		});

		if let Err(e) = self.record_create_transaction_cost(&init_code, &access_list) {
			return emit_exit!(e.into());
		}
		self.initialize_with_access_list(access_list);

		match self.create_inner(
			caller,
			CreateScheme::Legacy { caller },
			value,
			init_code,
			Some(gas_limit),
			false,
		) {
			Capture::Exit((s, _, _)) => emit_exit!(s),
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
		access_list: Vec<(H160, Vec<H256>)>, // See EIP-2930
	) -> ExitReason {
		let code_hash = H256::from_slice(Keccak256::digest(&init_code).as_slice());
		event!(TransactCreate2 {
			caller,
			value,
			init_code: &init_code,
			salt,
			gas_limit,
			address: self.create_address(CreateScheme::Create2 {
				caller,
				code_hash,
				salt,
			}),
		});

		if let Err(e) = self.record_create_transaction_cost(&init_code, &access_list) {
			return emit_exit!(e.into());
		}
		self.initialize_with_access_list(access_list);

		match self.create_inner(
			caller,
			CreateScheme::Create2 {
				caller,
				code_hash,
				salt,
			},
			value,
			init_code,
			Some(gas_limit),
			false,
		) {
			Capture::Exit((s, _, _)) => emit_exit!(s),
			Capture::Trap(_) => unreachable!(),
		}
	}

	/// Execute a `CALL` transaction with a given caller, address, value and
	/// gas limit and data.
	///
	/// Takes in an additional `access_list` parameter for EIP-2930 which was
	/// introduced in the Ethereum Berlin hard fork. If you do not wish to use
	/// this functionality, just pass in an empty vector.
	pub fn transact_call(
		&mut self,
		caller: H160,
		address: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: u64,
		access_list: Vec<(H160, Vec<H256>)>,
	) -> (ExitReason, Vec<u8>) {
		event!(TransactCall {
			caller,
			address,
			value,
			data: &data,
			gas_limit,
		});

		let transaction_cost = gasometer::call_transaction_cost(&data, &access_list);
		let gasometer = &mut self.state.metadata_mut().gasometer;
		match gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => return emit_exit!(e.into(), Vec::new()),
		}

		// Initialize initial addresses for EIP-2929
		if self.config.increase_state_access_gas {
			let addresses = core::iter::once(caller).chain(core::iter::once(address));
			self.state.metadata_mut().access_addresses(addresses);

			self.initialize_with_access_list(access_list);
		}

		self.state.inc_nonce(caller);

		let context = Context {
			caller,
			address,
			apparent_value: value,
		};

		match self.call_inner(
			address,
			Some(Transfer {
				source: caller,
				target: address,
				value,
			}),
			data,
			Some(gas_limit),
			false,
			false,
			false,
			context,
		) {
			Capture::Exit((s, v)) => emit_exit!(s, v),
			Capture::Trap(_) => unreachable!(),
		}
	}

	/// Get used gas for the current executor, given the price.
	pub fn used_gas(&self) -> u64 {
		self.state.metadata().gasometer.total_used_gas()
			- min(
				self.state.metadata().gasometer.total_used_gas() / self.config.max_refund_quotient,
				self.state.metadata().gasometer.refunded_gas() as u64,
			)
	}

	/// Get fee needed for the current executor, given the price.
	pub fn fee(&self, price: U256) -> U256 {
		let used_gas = self.used_gas();
		U256::from(used_gas) * price
	}

	/// Get account nonce.
	pub fn nonce(&self, address: H160) -> U256 {
		self.state.basic(address).nonce
	}

	/// Get the create address from given scheme.
	pub fn create_address(&self, scheme: CreateScheme) -> H160 {
		match scheme {
			CreateScheme::Create2 {
				caller,
				code_hash,
				salt,
			} => {
				let mut hasher = Keccak256::new();
				hasher.input(&[0xff]);
				hasher.input(&caller[..]);
				hasher.input(&salt[..]);
				hasher.input(&code_hash[..]);
				H256::from_slice(hasher.result().as_slice()).into()
			}
			CreateScheme::Legacy { caller } => {
				let nonce = self.nonce(caller);
				let mut stream = rlp::RlpStream::new_list(2);
				stream.append(&caller);
				stream.append(&nonce);
				H256::from_slice(Keccak256::digest(&stream.out()).as_slice()).into()
			}
			CreateScheme::Fixed(naddress) => naddress,
		}
	}

	pub fn initialize_with_access_list(&mut self, access_list: Vec<(H160, Vec<H256>)>) {
		let addresses = access_list.iter().map(|a| a.0);
		self.state.metadata_mut().access_addresses(addresses);

		let storage_keys = access_list
			.into_iter()
			.flat_map(|(address, keys)| keys.into_iter().map(move |key| (address, key)));
		self.state.metadata_mut().access_storages(storage_keys);
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
			};
		}

		fn check_first_byte(config: &Config, code: &[u8]) -> Result<(), ExitError> {
			if config.disallow_executable_format {
				if let Some(0xef) = code.get(0) {
					return Err(ExitError::InvalidCode);
				}
			}
			Ok(())
		}

		fn l64(gas: u64) -> u64 {
			gas - gas / 64
		}

		let address = self.create_address(scheme);

		self.state.metadata_mut().access_address(caller);
		self.state.metadata_mut().access_address(address);

		event!(Create {
			caller,
			address,
			scheme,
			value,
			init_code: &init_code,
			target_gas
		});

		if let Some(depth) = self.state.metadata().depth {
			if depth > self.config.call_stack_limit {
				return Capture::Exit((ExitError::CallTooDeep.into(), None, Vec::new()));
			}
		}

		if self.balance(caller) < value {
			return Capture::Exit((ExitError::OutOfFund.into(), None, Vec::new()));
		}

		let after_gas = if take_l64 && self.config.call_l64_after_gas {
			if self.config.estimate {
				let initial_after_gas = self.state.metadata().gasometer.gas();
				let diff = initial_after_gas - l64(initial_after_gas);
				try_or_fail!(self.state.metadata_mut().gasometer.record_cost(diff));
				self.state.metadata().gasometer.gas()
			} else {
				l64(self.state.metadata().gasometer.gas())
			}
		} else {
			self.state.metadata().gasometer.gas()
		};

		let target_gas = target_gas.unwrap_or(after_gas);

		let gas_limit = min(after_gas, target_gas);
		try_or_fail!(self.state.metadata_mut().gasometer.record_cost(gas_limit));

		self.state.inc_nonce(caller);

		self.enter_substate(gas_limit, false);

		{
			if self.code_size(address) != U256::zero() {
				let _ = self.exit_substate(StackExitKind::Failed);
				return Capture::Exit((ExitError::CreateCollision.into(), None, Vec::new()));
			}

			if self.nonce(address) > U256::zero() {
				let _ = self.exit_substate(StackExitKind::Failed);
				return Capture::Exit((ExitError::CreateCollision.into(), None, Vec::new()));
			}

			self.state.reset_storage(address);
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
		match self.state.transfer(transfer) {
			Ok(()) => (),
			Err(e) => {
				let _ = self.exit_substate(StackExitKind::Reverted);
				return Capture::Exit((ExitReason::Error(e), None, Vec::new()));
			}
		}

		if self.config.create_increase_nonce {
			self.state.inc_nonce(address);
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

				// As of EIP-3541 code starting with 0xef cannot be deployed
				if let Err(e) = check_first_byte(self.config, &out) {
					self.state.metadata_mut().gasometer.fail();
					let _ = self.exit_substate(StackExitKind::Failed);
					return Capture::Exit((e.into(), None, Vec::new()));
				}

				if let Some(limit) = self.config.create_contract_limit {
					if out.len() > limit {
						self.state.metadata_mut().gasometer.fail();
						let _ = self.exit_substate(StackExitKind::Failed);
						return Capture::Exit((
							ExitError::CreateContractLimit.into(),
							None,
							Vec::new(),
						));
					}
				}

				match self
					.state
					.metadata_mut()
					.gasometer
					.record_deposit(out.len())
				{
					Ok(()) => {
						let e = self.exit_substate(StackExitKind::Succeeded);
						self.state.set_code(address, out);
						try_or_fail!(e);
						Capture::Exit((ExitReason::Succeed(s), Some(address), Vec::new()))
					}
					Err(e) => {
						let _ = self.exit_substate(StackExitKind::Failed);
						Capture::Exit((ExitReason::Error(e), None, Vec::new()))
					}
				}
			}
			ExitReason::Error(e) => {
				self.state.metadata_mut().gasometer.fail();
				let _ = self.exit_substate(StackExitKind::Failed);
				Capture::Exit((ExitReason::Error(e), None, Vec::new()))
			}
			ExitReason::Revert(e) => {
				let _ = self.exit_substate(StackExitKind::Reverted);
				Capture::Exit((
					ExitReason::Revert(e),
					None,
					runtime.machine().return_value(),
				))
			}
			ExitReason::Fatal(e) => {
				self.state.metadata_mut().gasometer.fail();
				let _ = self.exit_substate(StackExitKind::Failed);
				Capture::Exit((ExitReason::Fatal(e), None, Vec::new()))
			}
		}
	}

	#[allow(clippy::too_many_arguments)]
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
			};
		}

		fn l64(gas: u64) -> u64 {
			gas - gas / 64
		}

		event!(Call {
			code_address,
			transfer: &transfer,
			input: &input,
			target_gas,
			is_static,
			context: &context,
		});

		let after_gas = if take_l64 && self.config.call_l64_after_gas {
			if self.config.estimate {
				let initial_after_gas = self.state.metadata().gasometer.gas();
				let diff = initial_after_gas - l64(initial_after_gas);
				try_or_fail!(self.state.metadata_mut().gasometer.record_cost(diff));
				self.state.metadata().gasometer.gas()
			} else {
				l64(self.state.metadata().gasometer.gas())
			}
		} else {
			self.state.metadata().gasometer.gas()
		};

		let target_gas = target_gas.unwrap_or(after_gas);
		let mut gas_limit = min(target_gas, after_gas);

		try_or_fail!(self.state.metadata_mut().gasometer.record_cost(gas_limit));

		if let Some(transfer) = transfer.as_ref() {
			if take_stipend && transfer.value != U256::zero() {
				gas_limit = gas_limit.saturating_add(self.config.call_stipend);
			}
		}

		let code = self.code(code_address);

		self.enter_substate(gas_limit, is_static);
		self.state.touch(context.address);

		if let Some(depth) = self.state.metadata().depth {
			if depth > self.config.call_stack_limit {
				let _ = self.exit_substate(StackExitKind::Reverted);
				return Capture::Exit((ExitError::CallTooDeep.into(), Vec::new()));
			}
		}

		if let Some(transfer) = transfer {
			match self.state.transfer(transfer) {
				Ok(()) => (),
				Err(e) => {
					let _ = self.exit_substate(StackExitKind::Reverted);
					return Capture::Exit((ExitReason::Error(e), Vec::new()));
				}
			}
		}

		if let Some(result) =
			self.precompile_set
				.execute(code_address, &input, Some(gas_limit), &context, is_static)
		{
			return match result {
				Ok(PrecompileOutput {
					exit_status,
					output,
					cost,
					logs,
				}) => {
					for Log {
						address,
						topics,
						data,
					} in logs
					{
						match self.log(address, topics, data) {
							Ok(_) => continue,
							Err(error) => {
								return Capture::Exit((ExitReason::Error(error), output));
							}
						}
					}

					let _ = self.state.metadata_mut().gasometer.record_cost(cost);
					let _ = self.exit_substate(StackExitKind::Succeeded);
					Capture::Exit((ExitReason::Succeed(exit_status), output))
				}
				Err(PrecompileFailure::Error { exit_status }) => {
					let _ = self.exit_substate(StackExitKind::Failed);
					Capture::Exit((ExitReason::Error(exit_status), Vec::new()))
				}
				Err(PrecompileFailure::Revert {
					exit_status,
					output,
					cost,
				}) => {
					let _ = self.state.metadata_mut().gasometer.record_cost(cost);
					let _ = self.exit_substate(StackExitKind::Reverted);
					Capture::Exit((ExitReason::Revert(exit_status), output))
				}
				Err(PrecompileFailure::Fatal { exit_status }) => {
					self.state.metadata_mut().gasometer.fail();
					let _ = self.exit_substate(StackExitKind::Failed);
					Capture::Exit((ExitReason::Fatal(exit_status), Vec::new()))
				}
			};
		}

		let mut runtime = Runtime::new(Rc::new(code), Rc::new(input), context, self.config);

		let reason = self.execute(&mut runtime);
		log::debug!(target: "evm", "Call execution using address {}: {:?}", code_address, reason);

		match reason {
			ExitReason::Succeed(s) => {
				let _ = self.exit_substate(StackExitKind::Succeeded);
				Capture::Exit((ExitReason::Succeed(s), runtime.machine().return_value()))
			}
			ExitReason::Error(e) => {
				let _ = self.exit_substate(StackExitKind::Failed);
				Capture::Exit((ExitReason::Error(e), Vec::new()))
			}
			ExitReason::Revert(e) => {
				let _ = self.exit_substate(StackExitKind::Reverted);
				Capture::Exit((ExitReason::Revert(e), runtime.machine().return_value()))
			}
			ExitReason::Fatal(e) => {
				self.state.metadata_mut().gasometer.fail();
				let _ = self.exit_substate(StackExitKind::Failed);
				Capture::Exit((ExitReason::Fatal(e), Vec::new()))
			}
		}
	}
}

impl<'config, 'precompiles, S: StackState<'config>, P: PrecompileSet> Handler
	for StackExecutor<'config, 'precompiles, S, P>
{
	type CreateInterrupt = Infallible;
	type CreateFeedback = Infallible;
	type CallInterrupt = Infallible;
	type CallFeedback = Infallible;

	fn balance(&self, address: H160) -> U256 {
		self.state.basic(address).balance
	}

	fn code_size(&self, address: H160) -> U256 {
		U256::from(self.state.code(address).len())
	}

	fn code_hash(&self, address: H160) -> H256 {
		if !self.exists(address) {
			return H256::default();
		}

		H256::from_slice(Keccak256::digest(&self.state.code(address)).as_slice())
	}

	fn code(&self, address: H160) -> Vec<u8> {
		self.state.code(address)
	}

	fn storage(&self, address: H160, index: H256) -> H256 {
		self.state.storage(address, index)
	}

	fn original_storage(&self, address: H160, index: H256) -> H256 {
		self.state
			.original_storage(address, index)
			.unwrap_or_default()
	}

	fn exists(&self, address: H160) -> bool {
		if self.config.empty_considered_exists {
			self.state.exists(address)
		} else {
			self.state.exists(address) && !self.state.is_empty(address)
		}
	}

	fn is_cold(&self, address: H160, maybe_index: Option<H256>) -> bool {
		match maybe_index {
			None => !self.precompile_set.is_precompile(address) && self.state.is_cold(address),
			Some(index) => self.state.is_storage_cold(address, index),
		}
	}

	fn gas_left(&self) -> U256 {
		U256::from(self.state.metadata().gasometer.gas())
	}

	fn gas_price(&self) -> U256 {
		self.state.gas_price()
	}
	fn origin(&self) -> H160 {
		self.state.origin()
	}
	fn block_hash(&self, number: U256) -> H256 {
		self.state.block_hash(number)
	}
	fn block_number(&self) -> U256 {
		self.state.block_number()
	}
	fn block_coinbase(&self) -> H160 {
		self.state.block_coinbase()
	}
	fn block_timestamp(&self) -> U256 {
		self.state.block_timestamp()
	}
	fn block_difficulty(&self) -> U256 {
		self.state.block_difficulty()
	}
	fn block_gas_limit(&self) -> U256 {
		self.state.block_gas_limit()
	}
	fn block_base_fee_per_gas(&self) -> U256 {
		self.state.block_base_fee_per_gas()
	}
	fn chain_id(&self) -> U256 {
		self.state.chain_id()
	}

	fn deleted(&self, address: H160) -> bool {
		self.state.deleted(address)
	}

	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError> {
		self.state.set_storage(address, index, value);
		Ok(())
	}

	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError> {
		self.state.log(address, topics, data);
		Ok(())
	}

	fn mark_delete(&mut self, address: H160, target: H160) -> Result<(), ExitError> {
		let balance = self.balance(address);

		event!(Suicide {
			target,
			address,
			balance,
		});

		self.state.transfer(Transfer {
			source: address,
			target,
			value: balance,
		})?;
		self.state.reset_balance(address);
		self.state.set_deleted(address);

		Ok(())
	}

	#[cfg(not(feature = "tracing"))]
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

	#[cfg(feature = "tracing")]
	fn create(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<u64>,
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Self::CreateInterrupt> {
		let capture = self.create_inner(caller, scheme, value, init_code, target_gas, true);

		if let Capture::Exit((ref reason, _, ref return_value)) = capture {
			emit_exit!(reason, return_value);
		}

		capture
	}

	#[cfg(not(feature = "tracing"))]
	fn call(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<u64>,
		is_static: bool,
		context: Context,
	) -> Capture<(ExitReason, Vec<u8>), Self::CallInterrupt> {
		self.call_inner(
			code_address,
			transfer,
			input,
			target_gas,
			is_static,
			true,
			true,
			context,
		)
	}

	#[cfg(feature = "tracing")]
	fn call(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<u64>,
		is_static: bool,
		context: Context,
	) -> Capture<(ExitReason, Vec<u8>), Self::CallInterrupt> {
		let capture = self.call_inner(
			code_address,
			transfer,
			input,
			target_gas,
			is_static,
			true,
			true,
			context,
		);

		if let Capture::Exit((ref reason, ref return_value)) = capture {
			emit_exit!(reason, return_value);
		}

		capture
	}

	#[inline]
	fn pre_validate(
		&mut self,
		context: &Context,
		opcode: Opcode,
		stack: &Stack,
	) -> Result<(), ExitError> {
		// log::trace!(target: "evm", "Running opcode: {:?}, Pre gas-left: {:?}", opcode, gasometer.gas());

		if let Some(cost) = gasometer::static_opcode_cost(opcode) {
			self.state.metadata_mut().gasometer.record_cost(cost)?;
		} else {
			let is_static = self.state.metadata().is_static;
			let (gas_cost, target, memory_cost) = gasometer::dynamic_opcode_cost(
				context.address,
				opcode,
				stack,
				is_static,
				self.config,
				self,
			)?;

			let gasometer = &mut self.state.metadata_mut().gasometer;

			gasometer.record_dynamic_cost(gas_cost, memory_cost)?;
			match target {
				StorageTarget::Address(address) => {
					self.state.metadata_mut().access_address(address)
				}
				StorageTarget::Slot(address, key) => {
					self.state.metadata_mut().access_storage(address, key)
				}
				StorageTarget::None => (),
			}
		}

		Ok(())
	}
}
