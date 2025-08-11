use alloc::vec::Vec;
use evm_interpreter::{Capture, ExitError, ExitResult, Interpreter};

/// Control for an invoker.
pub enum InvokerControl<I, S> {
	/// Pushing the call stack.
	Enter(I),
	/// Directly exit, not pushing the call stack.
	DirectExit(InvokerExit<S>),
}

pub struct InvokerExit<S> {
	pub result: ExitResult,
	pub substate: Option<S>,
	pub retval: Vec<u8>,
}

/// An invoker, responsible for pushing/poping values in the call stack.
pub trait Invoker<H> {
	/// Interpreter type.
	type Interpreter: Interpreter<H>;
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
			InvokerControl<Self::Interpreter, <Self::Interpreter as Interpreter<H>>::State>,
		),
		ExitError,
	>;

	/// Finalize a transaction.
	fn finalize_transact(
		&self,
		invoke: &Self::TransactInvoke,
		exit: InvokerExit<<Self::Interpreter as Interpreter<H>>::State>,
		handler: &mut H,
	) -> Result<Self::TransactValue, ExitError>;

	/// Enter a sub-layer call stack.
	#[allow(clippy::type_complexity)]
	fn enter_substack(
		&self,
		trap: <Self::Interpreter as Interpreter<H>>::Trap,
		machine: &mut Self::Interpreter,
		handler: &mut H,
		depth: usize,
	) -> Capture<
		Result<
			(
				Self::SubstackInvoke,
				InvokerControl<Self::Interpreter, <Self::Interpreter as Interpreter<H>>::State>,
			),
			ExitError,
		>,
		Self::Interrupt,
	>;

	/// Exit a sub-layer call stack.
	fn exit_substack(
		&self,
		trap_data: Self::SubstackInvoke,
		exit: InvokerExit<<Self::Interpreter as Interpreter<H>>::State>,
		parent: &mut Self::Interpreter,
		handler: &mut H,
	) -> Result<(), ExitError>;
}
