//! An invoker that records the input of every call trap that it sees.

#![allow(dead_code)]

use std::cell::{Cell, RefCell};

use evm::{
	InvokerControl, InvokerExit,
	backend::{RuntimeBackend, RuntimeEnvironment, TransactionalBackend},
	interpreter::{
		Capture, ExitError, FeedbackInterpreter, Interpreter,
		runtime::RuntimeState,
		trap::{CallCreateTrap, CallFeedback, CreateFeedback},
	},
	standard::{
		Config, Invoker as StandardInvoker, InvokerState, Resolver, SubstackInvoke, TransactInvoke,
		TransactValue,
	},
};

/// An invoker that wraps the standard invoker, and records the input of every
/// call trap that it sees, and the calls that unwind it.
pub struct RecordingInvoker<'a, 'config, 'resolver, R> {
	/// The standard invoker that handles every trap.
	standard: &'a StandardInvoker<'config, 'resolver, R>,
	/// Whether this invoker declares that it shares call input.
	forward: bool,
	/// The input of every call trap that this invoker saw.
	seen: RefCell<Vec<Vec<u8>>>,
	/// The number of sub-layers that this invoker was asked to exit.
	exits: Cell<usize>,
	/// Whether this invoker was asked to finalize the transaction.
	finalized: Cell<bool>,
}

impl<'a, 'config, 'resolver, R> RecordingInvoker<'a, 'config, 'resolver, R> {
	/// Wrap the given standard invoker.
	pub fn new(standard: &'a StandardInvoker<'config, 'resolver, R>, forward: bool) -> Self {
		Self {
			standard,
			forward,
			seen: RefCell::new(Vec::new()),
			exits: Cell::new(0),
			finalized: Cell::new(false),
		}
	}

	/// The input of every call trap that this invoker saw.
	pub fn seen(&self) -> Vec<Vec<u8>> {
		self.seen.borrow().clone()
	}

	/// The number of sub-layers that this invoker was asked to exit.
	pub fn exits(&self) -> usize {
		self.exits.get()
	}

	/// Whether this invoker was asked to finalize the transaction.
	pub fn finalized(&self) -> bool {
		self.finalized.get()
	}
}

impl<'a, 'config, 'resolver, H, R> evm::Invoker<H> for RecordingInvoker<'a, 'config, 'resolver, R>
where
	R::State: InvokerState + AsRef<RuntimeState> + AsMut<RuntimeState> + AsRef<Config>,
	<R::State as InvokerState>::TransactArgs: AsRef<evm::standard::TransactArgs<'config>>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
	R::Interpreter: Interpreter<H, Trap = CallCreateTrap>
		+ FeedbackInterpreter<H, CallFeedback>
		+ FeedbackInterpreter<H, CreateFeedback>,
{
	type State = R::State;
	type Interpreter = R::Interpreter;
	type Interrupt = std::convert::Infallible;
	type TransactArgs = <R::State as InvokerState>::TransactArgs;
	type TransactValue = TransactValue;
	type TransactInvoke = TransactInvoke<'config>;
	type SubstackInvoke = SubstackInvoke;

	fn shares_call_input(&self) -> bool {
		self.forward
	}

	fn new_transact(
		&self,
		args: Self::TransactArgs,
		handler: &mut H,
	) -> Result<
		(
			Self::TransactInvoke,
			InvokerControl<Self::Interpreter, Self::State>,
		),
		ExitError,
	> {
		self.standard.new_transact(args, handler)
	}

	fn finalize_transact(
		&self,
		invoke: &Self::TransactInvoke,
		exit: InvokerExit<Self::State>,
		handler: &mut H,
	) -> Result<Self::TransactValue, ExitError> {
		self.finalized.set(true);

		self.standard.finalize_transact(invoke, exit, handler)
	}

	#[allow(deprecated)]
	fn enter_substack(
		&self,
		trap: CallCreateTrap,
		machine: &mut Self::Interpreter,
		handler: &mut H,
		depth: usize,
	) -> Capture<
		Result<
			(
				Self::SubstackInvoke,
				InvokerControl<Self::Interpreter, Self::State>,
			),
			ExitError,
		>,
		Self::Interrupt,
	> {
		if let CallCreateTrap::Call(call) = &trap {
			self.seen.borrow_mut().push(call.input.clone());
		}

		self.standard.enter_substack(trap, machine, handler, depth)
	}

	fn exit_substack(
		&self,
		trap_data: Self::SubstackInvoke,
		exit: InvokerExit<Self::State>,
		parent: &mut Self::Interpreter,
		handler: &mut H,
	) -> Result<(), ExitError> {
		self.exits.set(self.exits.get() + 1);

		self.standard
			.exit_substack(trap_data, exit, parent, handler)
	}
}
