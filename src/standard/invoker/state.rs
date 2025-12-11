use alloc::vec::Vec;

use evm_interpreter::{
	ExitError,
	runtime::{GasState, RuntimeState},
};
use evm_interpreter::uint::{H160, H256, U256};

use crate::MergeStrategy;

/// Trait to be implemented by any state wishing to use [crate::standard::Invoker].
pub trait InvokerState: GasState + Sized {
	/// Type of the transaction argument.
	type TransactArgs;

	/// Create a new state from a call transaction.
	fn new_transact_call(
		runtime: RuntimeState,
		gas_limit: U256,
		data: &[u8],
		access_list: &[(H160, Vec<H256>)],
		args: &Self::TransactArgs,
	) -> Result<Self, ExitError>;
	/// Create a new state from a create transaction.
	fn new_transact_create(
		runtime: RuntimeState,
		gas_limit: U256,
		code: &[u8],
		access_list: &[(H160, Vec<H256>)],
		args: &Self::TransactArgs,
	) -> Result<Self, ExitError>;

	/// Create a substate from the current state.
	fn substate(
		&mut self,
		runtime: RuntimeState,
		gas_limit: U256,
		is_static: bool,
		call_has_value: bool,
	) -> Result<Self, ExitError>;
	/// Merge a substate to the current state using the given merge strategy.
	fn merge(&mut self, substate: Self, strategy: MergeStrategy);

	/// Record a code deposit.
	fn record_codedeposit(&mut self, len: usize) -> Result<(), ExitError>;
	/// Whether the current state is in the static frame.
	fn is_static(&self) -> bool;
	/// Effective gas. The final used gas as reported by the transaction.
	fn effective_gas(&self, with_refund: bool) -> U256;
}
