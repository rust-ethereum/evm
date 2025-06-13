use crate::backend::Backend;
use crate::executor::stack::precompile::{
	IsPrecompileResult, PrecompileFailure, PrecompileHandle, PrecompileOutput, PrecompileSet,
};
use crate::executor::stack::tagged_runtime::{RuntimeKind, TaggedRuntime};
use crate::gasometer::{self, Gasometer, StorageTarget};
use crate::maybe_borrowed::MaybeBorrowed;
use crate::{
	Capture, Config, Context, CreateScheme, ExitError, ExitReason, Handler, Opcode, Runtime, Stack,
	Transfer,
};
use alloc::{collections::BTreeSet, rc::Rc, vec::Vec};
use core::{cmp::min, convert::Infallible};
use evm_core::ExitFatal;
use evm_runtime::Resolve;
use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

/// EIP-7702 authorization tuple
#[derive(Clone, Debug)]
pub struct Authorization {
	/// Chain ID
	pub chain_id: U256,
	/// Address to delegate to
	pub address: H160,
	/// Nonce for the authorization
	pub nonce: U256,
	/// Address that authorized this delegation (recovered from signature)
	pub authorizing_address: H160,
}

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

const DEFAULT_CALL_STACK_CAPACITY: usize = 4;

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
	fn created(&self, address: H160) -> bool;
	fn is_cold(&self, address: H160) -> bool;
	fn is_storage_cold(&self, address: H160, key: H256) -> bool;

	fn inc_nonce(&mut self, address: H160) -> Result<(), ExitError>;
	fn set_storage(&mut self, address: H160, key: H256, value: H256);
	fn set_transient_storage(&mut self, address: H160, key: H256, value: H256);
	fn set_delegation(&mut self, address: H160, target: Option<H160>);
	fn reset_storage(&mut self, address: H160);
	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>);
	fn set_deleted(&mut self, address: H160);
	fn set_created(&mut self, address: H160);
	fn set_code(&mut self, address: H160, code: Vec<u8>);
	fn transfer(&mut self, transfer: Transfer) -> Result<(), ExitError>;
	fn reset_balance(&mut self, address: H160);
	fn touch(&mut self, address: H160);

	/// Fetch the code size of an address.
	/// Provide a default implementation by fetching the code, but
	/// can be customized to use a more performant approach that don't need to
	/// fetch the code.
	fn code_size(&self, address: H160) -> U256 {
		U256::from(self.code(address).len())
	}

	/// Fetch the code hash of an address.
	/// Provide a default implementation by fetching the code, but
	/// can be customized to use a more performant approach that don't need to
	/// fetch the code.
	fn code_hash(&self, address: H160) -> H256 {
		H256::from_slice(Keccak256::digest(self.code(address)).as_slice())
	}

	fn record_external_operation(
		&mut self,
		_op: crate::ExternalOperation,
	) -> Result<(), ExitError> {
		Ok(())
	}

	fn record_external_dynamic_opcode_cost(
		&mut self,
		_opcode: Opcode,
		_gas_cost: crate::gasometer::GasCost,
		_target: StorageTarget,
	) -> Result<(), ExitError> {
		Ok(())
	}

	fn record_external_cost(
		&mut self,
		_ref_time: Option<u64>,
		_proof_size: Option<u64>,
		_storage_growth: Option<u64>,
	) -> Result<(), ExitError> {
		Ok(())
	}

	fn refund_external_cost(&mut self, _ref_time: Option<u64>, _proof_size: Option<u64>) {}
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

	/// Process EIP-7702 authorization list.
	/// This should be called before transaction execution to set up delegation indicators.
	/// The authorizing addresses should already be recovered from signatures by the caller.
	pub fn process_authorizations(
		&mut self,
		authorizations: &[Authorization],
	) -> Result<(), ExitError> {
		if !self.config.has_eip_7702 {
			return Ok(());
		}

		for auth in authorizations {
			// Verify chain ID (if not zero, must match current chain)
			if !auth.chain_id.is_zero() && auth.chain_id != self.state.chain_id() {
				continue; // Skip authorizations for other chains
			}

			let authorizing_address = auth.authorizing_address;

			// Verify nonce matches current account nonce
			let current_nonce = self.state.basic(authorizing_address).nonce;
			if auth.nonce == current_nonce {
				// Set delegation indicator to 0xef0100 + auth.address
				let mut delegation_code = vec![0xef, 0x01, 0x00];
				delegation_code.extend_from_slice(auth.address.as_bytes());

				// Set the delegation in the state
				self.state
					.set_delegation(authorizing_address, Some(auth.address));
				self.state.set_code(authorizing_address, delegation_code);

				// Increment nonce
				self.state.inc_nonce(authorizing_address)?;
			}
		}

		Ok(())
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
		let mut call_stack = Vec::with_capacity(DEFAULT_CALL_STACK_CAPACITY);
		call_stack.push(TaggedRuntime {
			kind: RuntimeKind::Execute,
			inner: MaybeBorrowed::Borrowed(runtime),
		});
		let (reason, _, _) = self.execute_with_call_stack(&mut call_stack);
		reason
	}

	/// Execute using Runtimes on the call_stack until it returns.
	fn execute_with_call_stack(
		&mut self,
		call_stack: &mut Vec<TaggedRuntime<'_>>,
	) -> (ExitReason, Option<H160>, Vec<u8>) {
		// This `interrupt_runtime` is used to pass the runtime obtained from the
		// `Capture::Trap` branch in the match below back to the top of the call stack.
		// The reason we can't simply `push` the runtime directly onto the stack in the
		// `Capture::Trap` branch is because the borrow-checker complains that the stack
		// is already borrowed as long as we hold a pointer on the last element
		// (i.e. the currently executing runtime).
		let mut interrupt_runtime = None;
		loop {
			if let Some(rt) = interrupt_runtime.take() {
				call_stack.push(rt);
			}
			let runtime = match call_stack.last_mut() {
				Some(runtime) => runtime,
				None => {
					return (
						ExitReason::Fatal(ExitFatal::UnhandledInterrupt),
						None,
						Vec::new(),
					);
				}
			};
			let reason = {
				let inner_runtime = &mut runtime.inner;
				match inner_runtime.run(self) {
					Capture::Exit(reason) => reason,
					Capture::Trap(Resolve::Call(rt, _)) => {
						interrupt_runtime = Some(rt.0);
						continue;
					}
					Capture::Trap(Resolve::Create(rt, _)) => {
						interrupt_runtime = Some(rt.0);
						continue;
					}
				}
			};
			let runtime_kind = runtime.kind;
			let (reason, maybe_address, return_data) = match runtime_kind {
				RuntimeKind::Create(created_address) => {
					let (reason, maybe_address, return_data) = self.cleanup_for_create(
						created_address,
						reason,
						runtime.inner.machine().return_value(),
					);
					(reason, maybe_address, return_data)
				}
				RuntimeKind::Call(code_address) => {
					let return_data = self.cleanup_for_call(
						code_address,
						&reason,
						runtime.inner.machine().return_value(),
					);
					(reason, None, return_data)
				}
				RuntimeKind::Execute => (reason, None, runtime.inner.machine().return_value()),
			};
			// We're done with that runtime now, so can pop it off the call stack
			call_stack.pop();
			// Now pass the results from that runtime on to the next one in the stack
			let runtime = match call_stack.last_mut() {
				Some(r) => r,
				None => return (reason, None, return_data),
			};
			emit_exit!(&reason, &return_data);
			let inner_runtime = &mut runtime.inner;
			let maybe_error = match runtime_kind {
				RuntimeKind::Create(_) => {
					inner_runtime.finish_create(reason, maybe_address, return_data)
				}
				RuntimeKind::Call(_) => inner_runtime.finish_call(reason, return_data),
				RuntimeKind::Execute => inner_runtime.finish_call(reason, return_data),
			};
			// Early exit if passing on the result caused an error
			if let Err(e) = maybe_error {
				return (e, None, Vec::new());
			}
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

	fn maybe_record_init_code_cost(&mut self, init_code: &[u8]) -> Result<(), ExitError> {
		if let Some(limit) = self.config.max_initcode_size {
			// EIP-3860
			if init_code.len() > limit {
				self.state.metadata_mut().gasometer.fail();
				return Err(ExitError::CreateContractLimit);
			}
			return self
				.state
				.metadata_mut()
				.gasometer
				.record_cost(gasometer::init_code_cost(init_code));
		}
		Ok(())
	}

	/// Execute a `CREATE` transaction.
	pub fn transact_create(
		&mut self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		gas_limit: u64,
		access_list: Vec<(H160, Vec<H256>)>, // See EIP-2930
	) -> (ExitReason, Vec<u8>) {
		event!(TransactCreate {
			caller,
			value,
			init_code: &init_code,
			gas_limit,
			address: self.create_address(CreateScheme::Legacy { caller }),
		});

		if let Some(limit) = self.config.max_initcode_size {
			if init_code.len() > limit {
				self.state.metadata_mut().gasometer.fail();
				return emit_exit!(ExitError::CreateContractLimit.into(), Vec::new());
			}
		}

		if let Err(e) = self.record_create_transaction_cost(&init_code, &access_list) {
			return emit_exit!(e.into(), Vec::new());
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
			Capture::Exit((s, _, v)) => emit_exit!(s, v),
			Capture::Trap(rt) => {
				let mut cs = Vec::with_capacity(DEFAULT_CALL_STACK_CAPACITY);
				cs.push(rt.0);
				let (s, _, v) = self.execute_with_call_stack(&mut cs);
				emit_exit!(s, v)
			}
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
	) -> (ExitReason, Vec<u8>) {
		if let Some(limit) = self.config.max_initcode_size {
			if init_code.len() > limit {
				self.state.metadata_mut().gasometer.fail();
				return emit_exit!(ExitError::CreateContractLimit.into(), Vec::new());
			}
		}

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
			return emit_exit!(e.into(), Vec::new());
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
			Capture::Exit((s, _, v)) => emit_exit!(s, v),
			Capture::Trap(rt) => {
				let mut cs = Vec::with_capacity(DEFAULT_CALL_STACK_CAPACITY);
				cs.push(rt.0);
				let (s, _, v) = self.execute_with_call_stack(&mut cs);
				emit_exit!(s, v)
			}
		}
	}

	/// Execute a `CREATE` transaction that force the contract address
	pub fn transact_create_force_address(
		&mut self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		gas_limit: u64,
		access_list: Vec<(H160, Vec<H256>)>, // See EIP-2930
		contract_address: H160,
	) -> (ExitReason, Vec<u8>) {
		event!(TransactCreate {
			caller,
			value,
			init_code: &init_code,
			gas_limit,
			address: self.create_address(CreateScheme::Fixed(contract_address)),
		});

		if let Some(limit) = self.config.max_initcode_size {
			if init_code.len() > limit {
				self.state.metadata_mut().gasometer.fail();
				return emit_exit!(ExitError::CreateContractLimit.into(), Vec::new());
			}
		}

		if let Err(e) = self.record_create_transaction_cost(&init_code, &access_list) {
			return emit_exit!(e.into(), Vec::new());
		}
		self.initialize_with_access_list(access_list);

		match self.create_inner(
			caller,
			CreateScheme::Fixed(contract_address),
			value,
			init_code,
			Some(gas_limit),
			false,
		) {
			Capture::Exit((s, _, v)) => emit_exit!(s, v),
			Capture::Trap(rt) => {
				let mut cs = Vec::with_capacity(DEFAULT_CALL_STACK_CAPACITY);
				cs.push(rt.0);
				let (s, _, v) = self.execute_with_call_stack(&mut cs);
				emit_exit!(s, v)
			}
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

		// The nonce must be incremented before any gas record,
		// otherwise we might OOG without incrementing the nonce!
		if let Err(e) = self.state.inc_nonce(caller) {
			return (e.into(), Vec::new());
		}

		let transaction_cost = gasometer::call_transaction_cost(&data, &access_list);
		let gasometer = &mut self.state.metadata_mut().gasometer;
		match gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => return emit_exit!(e.into(), Vec::new()),
		}

		// Initialize initial addresses for EIP-2929
		if self.config.increase_state_access_gas {
			if self.config.warm_coinbase_address {
				// Warm coinbase address for EIP-3651
				let addresses = core::iter::once(caller)
					.chain(core::iter::once(address))
					.chain(core::iter::once(self.block_coinbase()));
				self.state.metadata_mut().access_addresses(addresses);
			} else {
				let addresses = core::iter::once(caller).chain(core::iter::once(address));
				self.state.metadata_mut().access_addresses(addresses);
			}

			self.initialize_with_access_list(access_list);
		}
		if let Err(e) = self.record_external_operation(crate::ExternalOperation::AccountBasicRead) {
			return (e.into(), Vec::new());
		}

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
			Capture::Trap(rt) => {
				let mut cs = Vec::with_capacity(DEFAULT_CALL_STACK_CAPACITY);
				cs.push(rt.0);
				let (s, _, v) = self.execute_with_call_stack(&mut cs);
				emit_exit!(s, v)
			}
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
		U256::from(used_gas).saturating_mul(price)
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
				hasher.update([0xff]);
				hasher.update(&caller[..]);
				hasher.update(&salt[..]);
				hasher.update(&code_hash[..]);
				H256::from_slice(hasher.finalize().as_slice()).into()
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
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), StackExecutorCreateInterrupt<'static>> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => return Capture::Exit((e.into(), None, Vec::new())),
				}
			};
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

		if let Err(e) = self.record_external_operation(crate::ExternalOperation::AccountBasicRead) {
			return Capture::Exit((ExitReason::Error(e), None, Vec::new()));
		}
		if let Err(e) = self.state.inc_nonce(caller) {
			return Capture::Exit((e.into(), None, Vec::new()));
		}

		self.state.set_created(address);

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

		self.enter_substate(gas_limit, false);

		{
			if let Err(e) =
				self.record_external_operation(crate::ExternalOperation::AddressCodeRead(address))
			{
				let _ = self.exit_substate(StackExitKind::Failed);
				return Capture::Exit((ExitReason::Error(e), None, Vec::new()));
			}
			let code_size = self.code_size(address);
			if code_size != U256::zero() {
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
			if let Err(e) =
				self.record_external_operation(crate::ExternalOperation::AccountBasicRead)
			{
				let _ = self.exit_substate(StackExitKind::Failed);
				return Capture::Exit((ExitReason::Error(e), None, Vec::new()));
			}
			if let Err(e) = self.state.inc_nonce(address) {
				return Capture::Exit((e.into(), None, Vec::new()));
			}
		}

		let runtime = Runtime::new(
			Rc::new(init_code),
			Rc::new(Vec::new()),
			context,
			self.config.stack_limit,
			self.config.memory_limit,
		);

		Capture::Trap(StackExecutorCreateInterrupt(TaggedRuntime {
			kind: RuntimeKind::Create(address),
			inner: MaybeBorrowed::Owned(runtime),
		}))
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
	) -> Capture<(ExitReason, Vec<u8>), StackExecutorCallInterrupt<'static>> {
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

		self.enter_substate(gas_limit, is_static);
		self.state.touch(context.address);

		if let Err(e) =
			self.record_external_operation(crate::ExternalOperation::AddressCodeRead(code_address))
		{
			let _ = self.exit_substate(StackExitKind::Failed);
			return Capture::Exit((ExitReason::Error(e), Vec::new()));
		}
		let code = self.code(code_address);
		if let Some(depth) = self.state.metadata().depth {
			if depth > self.config.call_stack_limit {
				let _ = self.exit_substate(StackExitKind::Reverted);
				return Capture::Exit((ExitError::CallTooDeep.into(), Vec::new()));
			}
		}

		if let Some(transfer) = transfer {
			if let Err(e) =
				self.record_external_operation(crate::ExternalOperation::AccountBasicRead)
			{
				let _ = self.exit_substate(StackExitKind::Failed);
				return Capture::Exit((ExitReason::Error(e), Vec::new()));
			}
			match self.state.transfer(transfer) {
				Ok(()) => (),
				Err(e) => {
					let _ = self.exit_substate(StackExitKind::Reverted);
					return Capture::Exit((ExitReason::Error(e), Vec::new()));
				}
			}
		}

		// At this point, the state has been modified in enter_substate to
		// reflect both the is_static parameter of this call and the is_static
		// of the caller context.
		let precompile_is_static = self.state.metadata().is_static();
		if let Some(result) = self.precompile_set.execute(&mut StackExecutorHandle {
			executor: self,
			code_address,
			input: &input,
			gas_limit: Some(gas_limit),
			context: &context,
			is_static: precompile_is_static,
		}) {
			return match result {
				Ok(PrecompileOutput {
					exit_status,
					output,
				}) => {
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
				}) => {
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

		let runtime = Runtime::new(
			Rc::new(code),
			Rc::new(input),
			context,
			self.config.stack_limit,
			self.config.memory_limit,
		);

		Capture::Trap(StackExecutorCallInterrupt(TaggedRuntime {
			kind: RuntimeKind::Call(code_address),
			inner: MaybeBorrowed::Owned(runtime),
		}))
	}

	fn cleanup_for_create(
		&mut self,
		created_address: H160,
		reason: ExitReason,
		return_data: Vec<u8>,
	) -> (ExitReason, Option<H160>, Vec<u8>) {
		fn check_first_byte(config: &Config, code: &[u8]) -> Result<(), ExitError> {
			if config.disallow_executable_format && Some(&Opcode::EOFMAGIC.as_u8()) == code.first()
			{
				return Err(ExitError::InvalidCode(Opcode::EOFMAGIC));
			}
			Ok(())
		}

		log::debug!(target: "evm", "Create execution using address {}: {:?}", created_address, reason);

		match reason {
			ExitReason::Succeed(s) => {
				let out = return_data;
				let address = created_address;
				// As of EIP-3541 code starting with 0xef cannot be deployed
				if let Err(e) = check_first_byte(self.config, &out) {
					self.state.metadata_mut().gasometer.fail();
					let _ = self.exit_substate(StackExitKind::Failed);
					return (e.into(), None, Vec::new());
				}

				if let Some(limit) = self.config.create_contract_limit {
					if out.len() > limit {
						self.state.metadata_mut().gasometer.fail();
						let _ = self.exit_substate(StackExitKind::Failed);
						return (ExitError::CreateContractLimit.into(), None, Vec::new());
					}
				}

				match self
					.state
					.metadata_mut()
					.gasometer
					.record_deposit(out.len())
				{
					Ok(()) => {
						if let Err(e) = self.record_external_operation(
							crate::ExternalOperation::Write(U256::from(out.len())),
						) {
							self.state.metadata_mut().gasometer.fail();
							let _ = self.exit_substate(StackExitKind::Failed);
							return (e.into(), None, Vec::new());
						}
						let exit_result = self.exit_substate(StackExitKind::Succeeded);
						self.state.set_code(address, out);
						if let Err(e) = exit_result {
							return (e.into(), None, Vec::new());
						}
						(ExitReason::Succeed(s), Some(address), Vec::new())
					}
					Err(e) => {
						let _ = self.exit_substate(StackExitKind::Failed);
						(ExitReason::Error(e), None, Vec::new())
					}
				}
			}
			ExitReason::Error(e) => {
				self.state.metadata_mut().gasometer.fail();
				let _ = self.exit_substate(StackExitKind::Failed);
				(ExitReason::Error(e), None, Vec::new())
			}
			ExitReason::Revert(e) => {
				let _ = self.exit_substate(StackExitKind::Reverted);
				(ExitReason::Revert(e), None, return_data)
			}
			ExitReason::Fatal(e) => {
				self.state.metadata_mut().gasometer.fail();
				let _ = self.exit_substate(StackExitKind::Failed);
				(ExitReason::Fatal(e), None, Vec::new())
			}
		}
	}

	fn cleanup_for_call(
		&mut self,
		code_address: H160,
		reason: &ExitReason,
		return_data: Vec<u8>,
	) -> Vec<u8> {
		log::debug!(target: "evm", "Call execution using address {}: {:?}", code_address, reason);
		match reason {
			ExitReason::Succeed(_) => {
				let _ = self.exit_substate(StackExitKind::Succeeded);
				return_data
			}
			ExitReason::Error(_) => {
				let _ = self.exit_substate(StackExitKind::Failed);
				Vec::new()
			}
			ExitReason::Revert(_) => {
				let _ = self.exit_substate(StackExitKind::Reverted);
				return_data
			}
			ExitReason::Fatal(_) => {
				self.state.metadata_mut().gasometer.fail();
				let _ = self.exit_substate(StackExitKind::Failed);
				Vec::new()
			}
		}
	}
}

pub struct StackExecutorCallInterrupt<'borrow>(TaggedRuntime<'borrow>);
pub struct StackExecutorCreateInterrupt<'borrow>(TaggedRuntime<'borrow>);

impl<'config, 'precompiles, S: StackState<'config>, P: PrecompileSet> Handler
	for StackExecutor<'config, 'precompiles, S, P>
{
	type CreateInterrupt = StackExecutorCreateInterrupt<'static>;
	type CreateFeedback = Infallible;
	type CallInterrupt = StackExecutorCallInterrupt<'static>;
	type CallFeedback = Infallible;

	fn balance(&self, address: H160) -> U256 {
		self.state.basic(address).balance
	}

	fn code_size(&self, address: H160) -> U256 {
		// EIP-7702: For code size, we need to get the delegated code size
		let code = self.code(address);
		U256::from(code.len())
	}

	fn code_hash(&self, address: H160) -> H256 {
		if !self.exists(address) {
			return H256::default();
		}

		// EIP-7702: For code hash, we need to get the hash of the delegated code
		let code = self.code(address);
		if code.is_empty() {
			H256::default()
		} else {
			use sha3::{Digest, Keccak256};
			H256::from_slice(&Keccak256::digest(&code))
		}
	}

	fn code(&self, address: H160) -> Vec<u8> {
		let code = self.state.code(address);

		// EIP-7702: Check for delegation indicator
		if self.config.has_eip_7702 {
			if let Some(delegation_target) = self.state.delegation(address) {
				// If there's a delegation, return the code from the delegated address
				return self.state.code(delegation_target);
			}

			// Check if the code itself contains a delegation indicator (0xef0100 + address)
			if code.len() >= 23 && code[0] == 0xef && code[1] == 0x01 && code[2] == 0x00 {
				// Extract the address from the delegation indicator
				let mut addr_bytes = [0u8; 20];
				addr_bytes.copy_from_slice(&code[3..23]);
				let delegation_address = H160::from(addr_bytes);
				return self.state.code(delegation_address);
			}
		}

		code
	}

	fn storage(&self, address: H160, index: H256) -> H256 {
		self.state.storage(address, index)
	}

	fn transient_storage(&self, address: H160, index: H256) -> H256 {
		self.state.transient_storage(address, index)
	}

	fn original_storage(&self, address: H160, index: H256) -> H256 {
		self.state
			.original_storage(address, index)
			.unwrap_or_default()
	}

	fn delegation(&self, address: H160) -> Option<H160> {
		self.state.delegation(address)
	}

	fn exists(&self, address: H160) -> bool {
		if self.config.empty_considered_exists {
			self.state.exists(address)
		} else {
			self.state.exists(address) && !self.state.is_empty(address)
		}
	}

	fn is_cold(&mut self, address: H160, maybe_index: Option<H256>) -> Result<bool, ExitError> {
		Ok(match maybe_index {
			None => {
				let is_precompile = match self
					.precompile_set
					.is_precompile(address, self.state.metadata().gasometer.gas())
				{
					IsPrecompileResult::Answer {
						is_precompile,
						extra_cost,
					} => {
						self.state
							.metadata_mut()
							.gasometer
							.record_cost(extra_cost)?;
						is_precompile
					}
					IsPrecompileResult::OutOfGas => return Err(ExitError::OutOfGas),
				};

				!is_precompile && self.state.is_cold(address)
			}
			Some(index) => self.state.is_storage_cold(address, index),
		})
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
	fn block_randomness(&self) -> Option<H256> {
		self.state.block_randomness()
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

	fn set_transient_storage(
		&mut self,
		address: H160,
		index: H256,
		value: H256,
	) -> Result<(), ExitError> {
		self.state.set_transient_storage(address, index, value);
		Ok(())
	}

	fn set_delegation(&mut self, address: H160, target: Option<H160>) -> Result<(), ExitError> {
		self.state.set_delegation(address, target);
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

		if self.config.has_eip_6780 && !self.state.created(address) {
			if address != target {
				self.state.transfer(Transfer {
					source: address,
					target,
					value: balance,
				})?;
			}
		} else {
			self.state.transfer(Transfer {
				source: address,
				target,
				value: balance,
			})?;
			self.state.reset_balance(address);
			self.state.set_deleted(address);
		}

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
		if let Err(e) = self.maybe_record_init_code_cost(&init_code) {
			let reason: ExitReason = e.into();
			emit_exit!(reason.clone());
			return Capture::Exit((reason, None, Vec::new()));
		}

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
		if let Err(e) = self.maybe_record_init_code_cost(&init_code) {
			let reason: ExitReason = e.into();
			emit_exit!(reason.clone());
			return Capture::Exit((reason, None, Vec::new()));
		}

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

			self.state
				.record_external_dynamic_opcode_cost(opcode, gas_cost, target)?;

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

	fn record_external_operation(&mut self, op: crate::ExternalOperation) -> Result<(), ExitError> {
		self.state.record_external_operation(op)
	}
}

struct StackExecutorHandle<'inner, 'config, 'precompiles, S, P> {
	executor: &'inner mut StackExecutor<'config, 'precompiles, S, P>,
	code_address: H160,
	input: &'inner [u8],
	gas_limit: Option<u64>,
	context: &'inner Context,
	is_static: bool,
}

impl<'inner, 'config, 'precompiles, S: StackState<'config>, P: PrecompileSet> PrecompileHandle
	for StackExecutorHandle<'inner, 'config, 'precompiles, S, P>
{
	// Perform subcall in provided context.
	/// Precompile specifies in which context the subcall is executed.
	fn call(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		gas_limit: Option<u64>,
		is_static: bool,
		context: &Context,
	) -> (ExitReason, Vec<u8>) {
		// For normal calls the cost is recorded at opcode level.
		// Since we don't go through opcodes we need manually record the call
		// cost. Not doing so will make the code panic as recording the call stipend
		// will do an underflow.
		let target_is_cold = match self.executor.is_cold(code_address, None) {
			Ok(x) => x,
			Err(err) => return (ExitReason::Error(err), Vec::new()),
		};

		let target_exists = self.executor.exists(code_address);

		let gas_cost = crate::gasometer::GasCost::Call {
			value: transfer.clone().map(|x| x.value).unwrap_or_else(U256::zero),
			gas: U256::from(gas_limit.unwrap_or(u64::MAX)),
			target_is_cold,
			target_exists,
		};

		// We record the length of the input.
		let memory_cost = Some(crate::gasometer::MemoryCost {
			offset: U256::zero(),
			len: input.len().into(),
		});

		if let Err(error) = self
			.executor
			.state
			.metadata_mut()
			.gasometer
			.record_dynamic_cost(gas_cost, memory_cost)
		{
			return (ExitReason::Error(error), Vec::new());
		}

		event!(PrecompileSubcall {
			code_address,
			transfer: &transfer,
			input: &input,
			target_gas: gas_limit,
			is_static,
			context
		});

		// Perform the subcall
		match Handler::call(
			self.executor,
			code_address,
			transfer,
			input,
			gas_limit,
			is_static,
			context.clone(),
		) {
			Capture::Exit((s, v)) => (s, v),
			Capture::Trap(rt) => {
				// Ideally this would pass the interrupt back to the executor so it could be
				// handled like any other call, however the type signature of this function does
				// not allow it. For now we'll make a recursive call instead of making a breaking
				// change to the precompile API. But this means a custom precompile could still
				// potentially cause a stack overflow if you're not careful.
				let mut call_stack = Vec::with_capacity(DEFAULT_CALL_STACK_CAPACITY);
				call_stack.push(rt.0);
				let (reason, _, return_data) =
					self.executor.execute_with_call_stack(&mut call_stack);
				emit_exit!(reason, return_data)
			}
		}
	}

	/// Record cost to the Runtime gasometer.
	fn record_cost(&mut self, cost: u64) -> Result<(), ExitError> {
		self.executor
			.state
			.metadata_mut()
			.gasometer
			.record_cost(cost)
	}

	/// Record Substrate specific cost.
	fn record_external_cost(
		&mut self,
		ref_time: Option<u64>,
		proof_size: Option<u64>,
		storage_growth: Option<u64>,
	) -> Result<(), ExitError> {
		self.executor
			.state
			.record_external_cost(ref_time, proof_size, storage_growth)
	}

	/// Refund Substrate specific cost.
	fn refund_external_cost(&mut self, ref_time: Option<u64>, proof_size: Option<u64>) {
		self.executor
			.state
			.refund_external_cost(ref_time, proof_size);
	}

	/// Retreive the remaining gas.
	fn remaining_gas(&self) -> u64 {
		self.executor.state.metadata().gasometer.gas()
	}

	/// Record a log.
	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError> {
		Handler::log(self.executor, address, topics, data)
	}

	/// Retreive the code address (what is the address of the precompile being called).
	fn code_address(&self) -> H160 {
		self.code_address
	}

	/// Retreive the input data the precompile is called with.
	fn input(&self) -> &[u8] {
		self.input
	}

	/// Retreive the context in which the precompile is executed.
	fn context(&self) -> &Context {
		self.context
	}

	/// Is the precompile call is done statically.
	fn is_static(&self) -> bool {
		self.is_static
	}

	/// Retreive the gas limit of this call.
	fn gas_limit(&self) -> Option<u64> {
		self.gas_limit
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::backend::{MemoryBackend, MemoryVicinity};
	use crate::executor::stack::{MemoryStackState, StackSubstateMetadata};
	use crate::{Config, Handler};
	use primitive_types::{H160, U256};
	use std::collections::BTreeMap;

	#[test]
	fn test_eip7702_delegation() {
		// Create a test environment with EIP-7702 enabled
		let config = Config::pectra();

		let vicinity = MemoryVicinity {
			gas_price: U256::zero(),
			origin: H160::default(),
			chain_id: U256::one(),
			block_hashes: vec![],
			block_number: U256::zero(),
			block_coinbase: H160::default(),
			block_timestamp: U256::zero(),
			block_difficulty: U256::zero(),
			block_gas_limit: U256::from(10_000_000u64),
			block_base_fee_per_gas: U256::zero(),
			block_randomness: None,
		};

		let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());
		let metadata = StackSubstateMetadata::new(u64::MAX, &config);
		let state = MemoryStackState::new(metadata, &mut backend);
		let precompiles = ();
		let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

		// Test addresses
		let authorizing_address = H160::from_low_u64_be(1);
		let delegate_address = H160::from_low_u64_be(2);

		// Set some actual code on the delegate address
		let delegate_code = vec![0x60, 0x01, 0x60, 0x01, 0x01]; // Simple bytecode
		executor
			.state_mut()
			.set_code(delegate_address, delegate_code.clone());

		// For testing, directly set up delegation without going through authorization processing
		// since signature recovery is the responsibility of frontier
		executor
			.state_mut()
			.set_delegation(authorizing_address, Some(delegate_address));

		// Set delegation code manually (this is what's stored in the authorizing account)
		let mut delegation_code = vec![0xef, 0x01, 0x00];
		delegation_code.extend_from_slice(delegate_address.as_bytes());
		executor
			.state_mut()
			.set_code(authorizing_address, delegation_code.clone());

		// Check that delegation was set
		let delegation = executor.delegation(authorizing_address);
		assert_eq!(delegation, Some(delegate_address));

		// According to EIP-7702, calling code() should return the delegate's code, not the delegation indicator
		let code = executor.code(authorizing_address);
		assert_eq!(
			code, delegate_code,
			"Should return delegate's code, not delegation indicator"
		);
	}

	#[test]
	fn test_eip7702_delegation_code_resolution() {
		// Create a test environment with EIP-7702 enabled
		let config = Config::pectra();

		let vicinity = MemoryVicinity {
			gas_price: U256::zero(),
			origin: H160::default(),
			chain_id: U256::one(),
			block_hashes: vec![],
			block_number: U256::zero(),
			block_coinbase: H160::default(),
			block_timestamp: U256::zero(),
			block_difficulty: U256::zero(),
			block_gas_limit: U256::from(10_000_000u64),
			block_base_fee_per_gas: U256::zero(),
			block_randomness: None,
		};

		let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());
		let metadata = StackSubstateMetadata::new(u64::MAX, &config);
		let state = MemoryStackState::new(metadata, &mut backend);
		let precompiles = ();
		let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

		// Test addresses
		let authorizing_address = H160::from_low_u64_be(1);
		let delegate_address = H160::from_low_u64_be(2);

		// Set some code on the delegate address
		let delegate_code = vec![0x60, 0x01, 0x60, 0x01, 0x01]; // Simple bytecode
		executor
			.state_mut()
			.set_code(delegate_address, delegate_code.clone());

		// Create delegation indicator code manually (0xef0100 + address)
		let mut delegation_code = vec![0xef, 0x01, 0x00];
		delegation_code.extend_from_slice(delegate_address.as_bytes());
		executor
			.state_mut()
			.set_code(authorizing_address, delegation_code);

		// When getting code from authorizing_address, it should return the delegate's code
		let resolved_code = executor.code(authorizing_address);
		assert_eq!(resolved_code, delegate_code);

		// Code size should also be resolved
		let code_size = executor.code_size(authorizing_address);
		assert_eq!(code_size, U256::from(delegate_code.len()));
	}

	#[test]
	fn test_eip7702_disabled() {
		// Create a test environment without EIP-7702
		let config = Config::cancun(); // Use Cancun config which doesn't have EIP-7702

		let vicinity = MemoryVicinity {
			gas_price: U256::zero(),
			origin: H160::default(),
			chain_id: U256::one(),
			block_hashes: vec![],
			block_number: U256::zero(),
			block_coinbase: H160::default(),
			block_timestamp: U256::zero(),
			block_difficulty: U256::zero(),
			block_gas_limit: U256::from(10_000_000u64),
			block_base_fee_per_gas: U256::zero(),
			block_randomness: None,
		};

		let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());
		let metadata = StackSubstateMetadata::new(u64::MAX, &config);
		let state = MemoryStackState::new(metadata, &mut backend);
		let precompiles = ();
		let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

		// Test addresses
		let authorizing_address = H160::from_low_u64_be(999);
		let delegate_address = H160::from_low_u64_be(2);

		// Create an authorization
		let authorization = Authorization {
			chain_id: U256::one(),
			address: delegate_address,
			nonce: U256::zero(),
			authorizing_address,
		};

		// Process the authorization - should succeed but not set any delegation since EIP-7702 is disabled
		let result = executor.process_authorizations(&[authorization]);
		assert!(
			result.is_ok(),
			"Authorization processing should succeed when EIP-7702 is disabled"
		);

		// Verify no delegation was set
		let delegation = executor.delegation(authorizing_address);
		assert_eq!(
			delegation, None,
			"No delegation should be set when EIP-7702 is disabled"
		);
	}

	#[test]
	fn test_eip7702_authorization_processing() {
		// Create a test environment with EIP-7702 enabled
		let config = Config::pectra();

		let vicinity = MemoryVicinity {
			gas_price: U256::zero(),
			origin: H160::default(),
			chain_id: U256::one(),
			block_hashes: vec![],
			block_number: U256::zero(),
			block_coinbase: H160::default(),
			block_timestamp: U256::zero(),
			block_difficulty: U256::zero(),
			block_gas_limit: U256::from(10_000_000u64),
			block_base_fee_per_gas: U256::zero(),
			block_randomness: None,
		};

		let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());
		let metadata = StackSubstateMetadata::new(u64::MAX, &config);
		let state = MemoryStackState::new(metadata, &mut backend);
		let precompiles = ();
		let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

		// Test addresses
		let authorizing_address = H160::from_low_u64_be(1);
		let delegate_address = H160::from_low_u64_be(2);

		// Set some code on the delegate address so it can be retrieved
		let delegate_code = vec![0x60, 0x01, 0x60, 0x01, 0x01]; // Simple bytecode
		executor
			.state_mut()
			.set_code(delegate_address, delegate_code.clone());

		// Create an authorization
		let authorization = Authorization {
			chain_id: U256::one(),
			address: delegate_address,
			nonce: U256::zero(),
			authorizing_address,
		};

		// Process the authorization
		let result = executor.process_authorizations(&[authorization]);
		assert!(result.is_ok(), "Authorization processing should succeed");

		// Check that delegation was set for the authorizing address
		let delegation = executor.delegation(authorizing_address);
		assert_eq!(
			delegation,
			Some(delegate_address),
			"Delegation should be set for the authorizing address"
		);

		// Check that the delegation code resolution works (should return delegate's code)
		let code = executor.code(authorizing_address);
		assert_eq!(
			code, delegate_code,
			"Should return delegate's code when delegation is set"
		);

		// Check that nonce was incremented for the authorizing address
		let basic = executor.state().basic(authorizing_address);
		assert_eq!(
			basic.nonce,
			U256::one(),
			"Nonce should be incremented for the authorizing address"
		);
	}

	#[test]
	fn test_eip7702_authorization_with_different_chain_ids() {
		// Test that authorizations for different chain IDs are handled correctly
		let config = Config::pectra();

		let vicinity = MemoryVicinity {
			gas_price: U256::zero(),
			origin: H160::default(),
			chain_id: U256::from(5), // Goerli chain ID
			block_hashes: vec![],
			block_number: U256::zero(),
			block_coinbase: H160::default(),
			block_timestamp: U256::zero(),
			block_difficulty: U256::zero(),
			block_gas_limit: U256::from(10_000_000u64),
			block_base_fee_per_gas: U256::zero(),
			block_randomness: None,
		};

		let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());
		let metadata = StackSubstateMetadata::new(u64::MAX, &config);
		let state = MemoryStackState::new(metadata, &mut backend);
		let precompiles = ();
		let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

		let authorizing_address = H160::from_low_u64_be(1);
		let delegate_address = H160::from_low_u64_be(2);

		// Create authorizations for different chain IDs
		let valid_auth = Authorization {
			chain_id: U256::from(5), // Matches vicinity chain_id
			address: delegate_address,
			nonce: U256::zero(),
			authorizing_address,
		};

		let invalid_auth = Authorization {
			chain_id: U256::from(1), // Different chain_id
			address: delegate_address,
			nonce: U256::zero(),
			authorizing_address,
		};

		let zero_chain_auth = Authorization {
			chain_id: U256::zero(), // Zero chain_id (should be accepted)
			address: delegate_address,
			nonce: U256::zero(),
			authorizing_address,
		};

		// Process authorizations
		let result = executor.process_authorizations(&[valid_auth, invalid_auth, zero_chain_auth]);
		assert!(result.is_ok(), "Authorization processing should succeed");

		// Check that delegation was set for the valid authorizations
		let delegation = executor.delegation(authorizing_address);
		assert_eq!(
			delegation,
			Some(delegate_address),
			"Delegation should be set for valid authorizations"
		);
	}

	#[test]
	fn test_eip7702_authorization_with_wrong_nonce() {
		// Test that authorization with wrong nonce is ignored
		let config = Config::pectra();

		let vicinity = MemoryVicinity {
			gas_price: U256::zero(),
			origin: H160::default(),
			chain_id: U256::one(),
			block_hashes: vec![],
			block_number: U256::zero(),
			block_coinbase: H160::default(),
			block_timestamp: U256::zero(),
			block_difficulty: U256::zero(),
			block_gas_limit: U256::from(10_000_000u64),
			block_base_fee_per_gas: U256::zero(),
			block_randomness: None,
		};

		let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());
		let metadata = StackSubstateMetadata::new(u64::MAX, &config);
		let state = MemoryStackState::new(metadata, &mut backend);
		let precompiles = ();
		let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

		let authorizing_address = H160::from_low_u64_be(1);
		let delegate_address = H160::from_low_u64_be(2);

		// Set initial nonce to 5 for the authorizing address
		executor.state_mut().inc_nonce(authorizing_address).unwrap();
		executor.state_mut().inc_nonce(authorizing_address).unwrap();
		executor.state_mut().inc_nonce(authorizing_address).unwrap();
		executor.state_mut().inc_nonce(authorizing_address).unwrap();
		executor.state_mut().inc_nonce(authorizing_address).unwrap();

		// Create authorization with wrong nonce (0 instead of 5)
		let wrong_nonce_auth = Authorization {
			chain_id: U256::one(),
			address: delegate_address,
			nonce: U256::zero(), // Wrong nonce
			authorizing_address,
		};

		// Create authorization with correct nonce (5)
		let correct_nonce_auth = Authorization {
			chain_id: U256::one(),
			address: delegate_address,
			nonce: U256::from(5), // Correct nonce
			authorizing_address,
		};

		// Process authorizations
		let result = executor.process_authorizations(&[wrong_nonce_auth, correct_nonce_auth]);
		assert!(result.is_ok(), "Authorization processing should succeed");

		// Check that only the correct nonce authorization was processed
		let delegation = executor.delegation(authorizing_address);
		assert_eq!(
			delegation,
			Some(delegate_address),
			"Delegation should be set for correct nonce authorization"
		);

		// Check that nonce was incremented from 5 to 6
		let basic = executor.state().basic(authorizing_address);
		assert_eq!(
			basic.nonce,
			U256::from(6),
			"Nonce should be incremented to 6"
		);
	}

	#[test]
	fn test_eip7702_authorization_validation() {
		// Test authorization processing validation
		let config = Config::pectra();

		let vicinity = MemoryVicinity {
			gas_price: U256::zero(),
			origin: H160::default(),
			chain_id: U256::one(),
			block_hashes: vec![],
			block_number: U256::zero(),
			block_coinbase: H160::default(),
			block_timestamp: U256::zero(),
			block_difficulty: U256::zero(),
			block_gas_limit: U256::from(10_000_000u64),
			block_base_fee_per_gas: U256::zero(),
			block_randomness: None,
		};

		let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());
		let metadata = StackSubstateMetadata::new(u64::MAX, &config);
		let state = MemoryStackState::new(metadata, &mut backend);
		let precompiles = ();
		let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

		let delegate_address = H160::from_low_u64_be(2);
		let authorizing_address = H160::from_low_u64_be(1);

		// Create a valid authorization (signature recovery would happen at higher level)
		let valid_auth = Authorization {
			chain_id: U256::one(),
			address: delegate_address,
			nonce: U256::zero(),
			authorizing_address,
		};

		// This should succeed since signature validation would be done at higher level
		let result = executor.process_authorizations(&[valid_auth]);
		assert!(result.is_ok(), "Should process valid authorization");
	}

	#[test]
	fn test_eip7702_multiple_authorizations_same_address() {
		// Test multiple authorizations from the same address
		let config = Config::pectra();

		let vicinity = MemoryVicinity {
			gas_price: U256::zero(),
			origin: H160::default(),
			chain_id: U256::one(),
			block_hashes: vec![],
			block_number: U256::zero(),
			block_coinbase: H160::default(),
			block_timestamp: U256::zero(),
			block_difficulty: U256::zero(),
			block_gas_limit: U256::from(10_000_000u64),
			block_base_fee_per_gas: U256::zero(),
			block_randomness: None,
		};

		let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());
		let metadata = StackSubstateMetadata::new(u64::MAX, &config);
		let state = MemoryStackState::new(metadata, &mut backend);
		let precompiles = ();
		let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

		let authorizing_address = H160::from_low_u64_be(1);

		let delegate_address_1 = H160::from_low_u64_be(2);
		let delegate_address_2 = H160::from_low_u64_be(3);

		// Set code on delegate addresses
		let delegate_code_1 = vec![0x60, 0x01];
		let delegate_code_2 = vec![0x60, 0x02];
		executor
			.state_mut()
			.set_code(delegate_address_1, delegate_code_1.clone());
		executor
			.state_mut()
			.set_code(delegate_address_2, delegate_code_2.clone());

		// Create first authorization (nonce 0)
		let auth1 = Authorization {
			chain_id: U256::one(),
			address: delegate_address_1,
			nonce: U256::zero(),
			authorizing_address,
		};

		// Create second authorization (nonce 1)
		let auth2 = Authorization {
			chain_id: U256::one(),
			address: delegate_address_2,
			nonce: U256::one(),
			authorizing_address,
		};

		// Process first authorization
		let result1 = executor.process_authorizations(&[auth1]);
		assert!(result1.is_ok(), "First authorization should succeed");

		// Check first delegation
		let delegation_1 = executor.delegation(authorizing_address);
		assert_eq!(delegation_1, Some(delegate_address_1));
		let code_1 = executor.code(authorizing_address);
		assert_eq!(code_1, delegate_code_1);

		// Process second authorization
		let result2 = executor.process_authorizations(&[auth2]);
		assert!(result2.is_ok(), "Second authorization should succeed");

		// Check that delegation was updated to second address
		let delegation_2 = executor.delegation(authorizing_address);
		assert_eq!(delegation_2, Some(delegate_address_2));
		let code_2 = executor.code(authorizing_address);
		assert_eq!(code_2, delegate_code_2);

		// Check that nonce was incremented twice
		let basic = executor.state().basic(authorizing_address);
		assert_eq!(basic.nonce, U256::from(2));
	}
}
