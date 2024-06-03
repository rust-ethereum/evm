use alloc::vec::Vec;

use evm_interpreter::{
	error::{Capture, ExitError, ExitResult},
	Interpreter,
};

/// Control for an invoker.
pub enum InvokerControl<VE, VD> {
	/// Pushing the call stack.
	Enter(VE),
	/// Directly exit, not pushing the call stack.
	DirectExit(VD),
}

/// An invoker, responsible for pushing/poping values in the call stack.
pub trait Invoker<H, Tr> {
	type State;
	type Interpreter: Interpreter<State = Self::State>;
	/// Possible interrupt type that may be returned by the call stack.
	type Interrupt;

	/// Type for transaction arguments.
	type TransactArgs;
	/// The invoke of a top-layer transaction call stack. When finalizing a
	/// transaction, this invoke is used to figure out the finalization routine.
	type TransactInvoke;
	/// The returned value of the transaction.
	type TransactValue;
	/// The invoke of a sub-layer call stack. When exiting a call stack, this
	/// invoke is used to figure out the exit routine.
	type SubstackInvoke;

	/// Create a new transaction with the given transaction arguments.
	#[allow(clippy::type_complexity)]
	fn new_transact(
		&self,
		args: Self::TransactArgs,
		handler: &mut H,
	) -> Result<
		(
			Self::TransactInvoke,
			InvokerControl<Self::Interpreter, (ExitResult, (Self::State, Vec<u8>))>,
		),
		ExitError,
	>;

	/// Finalize a transaction.
	fn finalize_transact(
		&self,
		invoke: &Self::TransactInvoke,
		exit: ExitResult,
		machine: (Self::State, Vec<u8>),
		handler: &mut H,
	) -> Result<Self::TransactValue, ExitError>;

	/// Enter a sub-layer call stack.
	#[allow(clippy::type_complexity)]
	fn enter_substack(
		&self,
		trap: Tr,
		machine: &mut Self::Interpreter,
		handler: &mut H,
		depth: usize,
	) -> Capture<
		Result<
			(
				Self::SubstackInvoke,
				InvokerControl<Self::Interpreter, (ExitResult, (Self::State, Vec<u8>))>,
			),
			ExitError,
		>,
		Self::Interrupt,
	>;

	/// Exit a sub-layer call stack.
	fn exit_substack(
		&self,
		result: ExitResult,
		child: (Self::State, Vec<u8>),
		trap_data: Self::SubstackInvoke,
		parent: &mut Self::Interpreter,
		handler: &mut H,
	) -> Result<(), ExitError>;
}
