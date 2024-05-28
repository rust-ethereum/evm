use alloc::vec::Vec;

use evm_interpreter::{
	error::ExitError,
	runtime::{GasState, RuntimeState},
};
use primitive_types::{H160, H256, U256};

use crate::{standard::Config, MergeStrategy};

pub trait InvokerState<'config>: GasState + Sized {
	fn new_transact_call(
		runtime: RuntimeState,
		gas_limit: U256,
		data: &[u8],
		access_list: &[(H160, Vec<H256>)],
		config: &'config Config,
	) -> Result<Self, ExitError>;
	fn new_transact_create(
		runtime: RuntimeState,
		gas_limit: U256,
		code: &[u8],
		access_list: &[(H160, Vec<H256>)],
		config: &'config Config,
	) -> Result<Self, ExitError>;

	fn substate(
		&mut self,
		runtime: RuntimeState,
		gas_limit: U256,
		is_static: bool,
		call_has_value: bool,
	) -> Result<Self, ExitError>;
	fn merge(&mut self, substate: Self, strategy: MergeStrategy);

	fn record_codedeposit(&mut self, len: usize) -> Result<(), ExitError>;

	fn is_static(&self) -> bool;
	fn effective_gas(&self) -> U256;
	fn config(&self) -> &Config;
}
