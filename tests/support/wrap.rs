//! An interpreter and a resolver that wrap the standard ones.
//!
//! The pipeline tests use them to show that an interpreter has to expose its
//! machine for the input of a call to be shared, and that a resolver has to
//! accept the input as a [`CallInput`] to avoid copying it.

#![allow(dead_code)]

use evm::{
	InvokerControl,
	interpreter::{
		CallInput, Capture, EtableInterpreter, ExitError, ExitResult, FeedbackInterpreter,
		Interpreter, Machine,
		etable::Etable,
		runtime::{RuntimeBackend, RuntimeState},
		trap::{CallFeedback, CallScheme, CreateFeedback},
	},
	standard::{Config, EtableResolver, PrecompileSet, Resolver, ResolverOrigin},
	uint::H160,
};

/// An interpreter that wraps an [`EtableInterpreter`].
///
/// With `FORWARD`, [`Interpreter::machine_mut`] forwards to the wrapped
/// interpreter, so that an invoker reaches the machine. Without it, the method
/// keeps its default of `None`.
pub struct WrappedInterpreter<'etable, S, ES, const FORWARD: bool> {
	inner: EtableInterpreter<'etable, S, ES>,
}

impl<'etable, S, ES, const FORWARD: bool> WrappedInterpreter<'etable, S, ES, FORWARD> {
	/// Wrap the given interpreter.
	pub fn new(inner: EtableInterpreter<'etable, S, ES>) -> Self {
		Self { inner }
	}

	/// The wrapped interpreter.
	pub fn into_inner(self) -> EtableInterpreter<'etable, S, ES> {
		self.inner
	}
}

impl<'etable, S, H, ES, const FORWARD: bool> Interpreter<H>
	for WrappedInterpreter<'etable, S, ES, FORWARD>
where
	ES: Etable<H, State = S>,
{
	type State = S;
	type Trap = ES::Trap;

	fn deconstruct(self) -> (S, Vec<u8>) {
		Interpreter::deconstruct(self.inner)
	}

	fn state(&self) -> &S {
		self.inner.state()
	}

	fn state_mut(&mut self) -> &mut S {
		self.inner.state_mut()
	}

	fn machine_mut(&mut self) -> Option<&mut Machine<S>> {
		if FORWARD {
			self.inner.machine_mut()
		} else {
			None
		}
	}

	fn run(&mut self, handle: &mut H) -> Capture<ExitResult, Self::Trap> {
		self.inner.run(handle)
	}
}

impl<'etable, S, H, ES, const FORWARD: bool> FeedbackInterpreter<H, CallFeedback>
	for WrappedInterpreter<'etable, S, ES, FORWARD>
where
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	ES: Etable<H, State = S>,
{
	fn feedback(&mut self, feedback: CallFeedback, handler: &mut H) -> Result<(), ExitError> {
		self.inner.feedback(feedback, handler)
	}
}

impl<'etable, S, H, ES, const FORWARD: bool> FeedbackInterpreter<H, CreateFeedback>
	for WrappedInterpreter<'etable, S, ES, FORWARD>
where
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	ES: Etable<H, State = S>,
{
	fn feedback(&mut self, feedback: CreateFeedback, handler: &mut H) -> Result<(), ExitError> {
		self.inner.feedback(feedback, handler)
	}
}

/// Turn a control of a machine into a control of a wrapped one.
fn wrap_control<'etable, S, ES, const FORWARD: bool>(
	control: InvokerControl<EtableInterpreter<'etable, S, ES>, S>,
) -> InvokerControl<WrappedInterpreter<'etable, S, ES, FORWARD>, S> {
	match control {
		InvokerControl::Enter(interpreter) => {
			InvokerControl::Enter(WrappedInterpreter::new(interpreter))
		}
		InvokerControl::DirectExit(exit) => InvokerControl::DirectExit(exit),
	}
}

/// A resolver that wraps every interpreter of an [`EtableResolver`], and
/// forwards both ways of resolving a call.
pub struct WrappedResolver<'a, 'precompile, 'etable, Pre, ES, const FORWARD: bool> {
	inner: &'a EtableResolver<'precompile, 'etable, Pre, ES>,
}

impl<'a, 'precompile, 'etable, Pre, ES, const FORWARD: bool>
	WrappedResolver<'a, 'precompile, 'etable, Pre, ES, FORWARD>
{
	/// Wrap the given resolver.
	pub fn new(inner: &'a EtableResolver<'precompile, 'etable, Pre, ES>) -> Self {
		Self { inner }
	}
}

impl<'a, 'precompile, 'etable, H, Pre, ES, const FORWARD: bool> Resolver<H>
	for WrappedResolver<'a, 'precompile, 'etable, Pre, ES, FORWARD>
where
	ES::State: AsRef<RuntimeState> + AsMut<RuntimeState> + AsRef<Config>,
	H: RuntimeBackend,
	Pre: PrecompileSet<ES::State, H>,
	ES: Etable<H>,
{
	type State = ES::State;
	type Interpreter = WrappedInterpreter<'etable, ES::State, ES, FORWARD>;

	fn resolve_call(
		&self,
		origin: ResolverOrigin,
		scheme: CallScheme,
		code_address: H160,
		input: Vec<u8>,
		state: Self::State,
		handler: &mut H,
	) -> Result<InvokerControl<Self::Interpreter, Self::State>, ExitError> {
		Ok(wrap_control(self.inner.resolve_call(
			origin,
			scheme,
			code_address,
			input,
			state,
			handler,
		)?))
	}

	fn resolve_call_with_input(
		&self,
		origin: ResolverOrigin,
		scheme: CallScheme,
		code_address: H160,
		input: CallInput,
		state: Self::State,
		handler: &mut H,
	) -> Result<InvokerControl<Self::Interpreter, Self::State>, ExitError> {
		Ok(wrap_control(self.inner.resolve_call_with_input(
			origin,
			scheme,
			code_address,
			input,
			state,
			handler,
		)?))
	}

	fn resolve_create(
		&self,
		origin: ResolverOrigin,
		init_code: Vec<u8>,
		state: Self::State,
		handler: &mut H,
	) -> Result<InvokerControl<Self::Interpreter, Self::State>, ExitError> {
		Ok(wrap_control(
			self.inner
				.resolve_create(origin, init_code, state, handler)?,
		))
	}
}

/// A resolver that wraps every interpreter of an [`EtableResolver`], and
/// implements only [`Resolver::resolve_call`], so that the default of
/// [`Resolver::resolve_call_with_input`] copies the input.
pub struct CopyOnlyResolver<'a, 'precompile, 'etable, Pre, ES, const FORWARD: bool> {
	inner: &'a EtableResolver<'precompile, 'etable, Pre, ES>,
}

impl<'a, 'precompile, 'etable, Pre, ES, const FORWARD: bool>
	CopyOnlyResolver<'a, 'precompile, 'etable, Pre, ES, FORWARD>
{
	/// Wrap the given resolver.
	pub fn new(inner: &'a EtableResolver<'precompile, 'etable, Pre, ES>) -> Self {
		Self { inner }
	}
}

impl<'a, 'precompile, 'etable, H, Pre, ES, const FORWARD: bool> Resolver<H>
	for CopyOnlyResolver<'a, 'precompile, 'etable, Pre, ES, FORWARD>
where
	ES::State: AsRef<RuntimeState> + AsMut<RuntimeState> + AsRef<Config>,
	H: RuntimeBackend,
	Pre: PrecompileSet<ES::State, H>,
	ES: Etable<H>,
{
	type State = ES::State;
	type Interpreter = WrappedInterpreter<'etable, ES::State, ES, FORWARD>;

	fn resolve_call(
		&self,
		origin: ResolverOrigin,
		scheme: CallScheme,
		code_address: H160,
		input: Vec<u8>,
		state: Self::State,
		handler: &mut H,
	) -> Result<InvokerControl<Self::Interpreter, Self::State>, ExitError> {
		Ok(wrap_control(self.inner.resolve_call(
			origin,
			scheme,
			code_address,
			input,
			state,
			handler,
		)?))
	}

	fn resolve_create(
		&self,
		origin: ResolverOrigin,
		init_code: Vec<u8>,
		state: Self::State,
		handler: &mut H,
	) -> Result<InvokerControl<Self::Interpreter, Self::State>, ExitError> {
		Ok(wrap_control(
			self.inner
				.resolve_create(origin, init_code, state, handler)?,
		))
	}
}
