//! # Standard machines and gasometers
//!
//! This module implements the standard configurations of the interpreter, like how it works on
//! Ethereum mainnet. Most of them can still be customized to add additional functionality, by
//! wrapping them or replacing the generic parameters.

mod config;
mod gasometer;
mod invoker;

use alloc::vec::Vec;
use core::marker::PhantomData;

use evm_interpreter::{
	Control, ExitError, etable, eval,
	runtime::{GasState, RuntimeBackend, RuntimeConfig, RuntimeEnvironment, RuntimeState},
	trap::CallCreateTrap,
};
use primitive_types::{H160, H256, U256};

pub use self::{
	config::Config,
	gasometer::{GasometerState, eval as eval_gasometer},
	invoker::{
		EtableResolver, Invoker, InvokerState, PrecompileSet, Resolver, ResolverOrigin,
		SubstackInvoke, TransactArgs, TransactArgsCallCreate, TransactGasPrice, TransactInvoke,
		TransactValue, TransactValueCallCreate, routines,
	},
};
use crate::{MergeStrategy, gasometer::GasMutState};

/// Standard machine.
pub type Machine<'config> = evm_interpreter::Machine<State<'config>>;

/// Standard Etable opcode handle function.
pub type Efn<'config, H> = etable::Efn<State<'config>, H, CallCreateTrap>;

/// Standard Etable.
pub type DispatchEtable<'config, H, F = Efn<'config, H>> =
	etable::DispatchEtable<State<'config>, H, CallCreateTrap, F>;

/// Gasometer Etable.
pub struct GasometerEtable<'config>(PhantomData<&'config ()>);

impl<'config> GasometerEtable<'config> {
	/// Create a new gasometer etable.
	pub const fn new() -> Self {
		Self(PhantomData)
	}
}

impl<'config> Default for GasometerEtable<'config> {
	fn default() -> Self {
		Self::new()
	}
}

impl<'config, H> etable::Etable<H> for GasometerEtable<'config>
where
	H: RuntimeBackend,
{
	type State = State<'config>;
	type Trap = CallCreateTrap;

	fn eval(
		&self,
		machine: &mut Machine<'config>,
		handle: &mut H,
		position: usize,
	) -> Control<Self::Trap> {
		eval_gasometer(machine, handle, position)
	}
}

/// Execution etable.
pub struct ExecutionEtable<'config>(PhantomData<&'config ()>);

impl<'config> ExecutionEtable<'config> {
	/// Create a new execution etable.
	pub const fn new() -> Self {
		Self(PhantomData)
	}
}

impl<'config> Default for ExecutionEtable<'config> {
	fn default() -> Self {
		Self::new()
	}
}

impl<'config, H> etable::Etable<H> for ExecutionEtable<'config>
where
	H: RuntimeBackend + RuntimeEnvironment,
{
	type State = State<'config>;
	type Trap = CallCreateTrap;

	fn eval(
		&self,
		machine: &mut Machine<'config>,
		handle: &mut H,
		position: usize,
	) -> Control<Self::Trap> {
		eval::eval_any(machine, handle, position)
	}
}

/// Standard state.
pub struct State<'config> {
	/// Runtime state.
	pub runtime: RuntimeState,
	/// Gasometer state.
	pub gasometer: GasometerState,
	/// Current config.
	pub config: &'config Config,
}

impl<'config> AsRef<RuntimeState> for State<'config> {
	fn as_ref(&self) -> &RuntimeState {
		&self.runtime
	}
}

impl<'config> AsMut<RuntimeState> for State<'config> {
	fn as_mut(&mut self) -> &mut RuntimeState {
		&mut self.runtime
	}
}

impl<'config> AsRef<GasometerState> for State<'config> {
	fn as_ref(&self) -> &GasometerState {
		&self.gasometer
	}
}

impl<'config> AsMut<GasometerState> for State<'config> {
	fn as_mut(&mut self) -> &mut GasometerState {
		&mut self.gasometer
	}
}

impl<'config> AsRef<Config> for State<'config> {
	fn as_ref(&self) -> &Config {
		self.config
	}
}

impl<'config> AsRef<RuntimeConfig> for State<'config> {
	fn as_ref(&self) -> &RuntimeConfig {
		&self.config.runtime
	}
}

impl<'config> GasState for State<'config> {
	fn gas(&self) -> U256 {
		self.gasometer.gas()
	}
}

impl<'config> GasMutState for State<'config> {
	fn record_gas(&mut self, gas: U256) -> Result<(), ExitError> {
		self.gasometer.record_gas(gas)
	}
}

impl<'config> InvokerState for State<'config> {
	type TransactArgs = TransactArgs<'config>;

	fn new_transact_call(
		runtime: RuntimeState,
		gas_limit: U256,
		data: &[u8],
		access_list: &[(H160, Vec<H256>)],
		args: &TransactArgs<'config>,
	) -> Result<Self, ExitError> {
		let config = args.config;
		Ok(Self {
			runtime,
			gasometer: GasometerState::new_transact_call(gas_limit, data, access_list, config)?,
			config,
		})
	}

	fn new_transact_create(
		runtime: RuntimeState,
		gas_limit: U256,
		code: &[u8],
		access_list: &[(H160, Vec<H256>)],
		args: &TransactArgs<'config>,
	) -> Result<Self, ExitError> {
		let config = args.config;
		Ok(Self {
			runtime,
			gasometer: GasometerState::new_transact_create(gas_limit, code, access_list, config)?,
			config,
		})
	}

	fn substate(
		&mut self,
		runtime: RuntimeState,
		gas_limit: U256,
		is_static: bool,
		call_has_value: bool,
	) -> Result<Self, ExitError> {
		Ok(Self {
			runtime,
			gasometer: self.gasometer.submeter(
				gas_limit,
				is_static,
				call_has_value,
				self.config,
			)?,
			config: self.config,
		})
	}

	fn merge(&mut self, substate: Self, strategy: MergeStrategy) {
		self.gasometer.merge(substate.gasometer, strategy)
	}

	fn record_codedeposit(&mut self, len: usize) -> Result<(), ExitError> {
		self.gasometer.record_codedeposit(len)
	}

	fn is_static(&self) -> bool {
		self.gasometer.is_static
	}

	fn effective_gas(&self, with_refund: bool) -> U256 {
		self.gasometer.effective_gas(with_refund, self.config)
	}
}
