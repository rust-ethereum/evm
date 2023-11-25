use crate::{Capture, ExitError, ExitResult};

/// Control for an invoker.
pub enum InvokerControl<VE, VD> {
	/// Pushing the call stack.
	Enter(VE),
	/// Directly exit, not pushing the call stack.
	DirectExit(VD),
}

/// A machine that is put onto the call stack.
pub trait InvokerMachine<H, Tr> {
	/// Deconstruct value of the machine.
	///
	/// This type is needed bacause an invoker may not push a value onto the
	/// call stack, but directly exit. In the latter case, it should return the
	/// deconstruct value. When popping from the call stack, we also deconstruct
	/// the machine to the deconstruct value, thus unifying the types.
	type Deconstruct;

	/// Step the machine using a handler.
	fn step(&mut self, handler: &mut H) -> Result<(), Capture<ExitResult, Tr>>;
	/// Run the machine until it returns.
	fn run(&mut self, handler: &mut H) -> Capture<ExitResult, Tr>;
	/// Deconstruct the machine to its deconstruct value.
	fn deconstruct(self) -> Self::Deconstruct;
}

/// An invoker, responsible for pushing/poping values in the call stack.
pub trait Invoker<H, Tr> {
	/// Machine type on the call stack.
	type Machine: InvokerMachine<H, Tr>;
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
	fn new_transact(
		&self,
		args: Self::TransactArgs,
		handler: &mut H,
	) -> Result<
		(
			Self::TransactInvoke,
			InvokerControl<
				Self::Machine,
				(
					ExitResult,
					<Self::Machine as InvokerMachine<H, Tr>>::Deconstruct,
				),
			>,
		),
		ExitError,
	>;

	/// Finalize a transaction.
	fn finalize_transact(
		&self,
		invoke: &Self::TransactInvoke,
		exit: ExitResult,
		machine: <Self::Machine as InvokerMachine<H, Tr>>::Deconstruct,
		handler: &mut H,
	) -> Result<Self::TransactValue, ExitError>;

	/// Enter a sub-layer call stack.
	fn enter_substack(
		&self,
		trap: Tr,
		machine: &mut Self::Machine,
		handler: &mut H,
		depth: usize,
	) -> Capture<
		Result<
			(
				Self::SubstackInvoke,
				InvokerControl<
					Self::Machine,
					(
						ExitResult,
						<Self::Machine as InvokerMachine<H, Tr>>::Deconstruct,
					),
				>,
			),
			ExitError,
		>,
		Self::Interrupt,
	>;

	/// Exit a sub-layer call stack.
	fn exit_substack(
		&self,
		result: ExitResult,
		child: <Self::Machine as InvokerMachine<H, Tr>>::Deconstruct,
		trap_data: Self::SubstackInvoke,
		parent: &mut Self::Machine,
		handler: &mut H,
	) -> Result<(), ExitError>;
}
