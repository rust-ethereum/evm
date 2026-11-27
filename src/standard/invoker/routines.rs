//! Internal routines that can be reused.

use alloc::vec::Vec;
use evm_interpreter::uint::H160;
use evm_interpreter::{
	CallInput, ExitError, ExitException, Interpreter, Opcode,
	runtime::{
		RuntimeBackend, RuntimeEnvironment, RuntimeState, SetCodeOrigin, TouchKind, Transfer,
	},
	trap::{CallScheme, CallTrap, CreateTrap},
};

use crate::{
	backend::TransactionalBackend,
	invoker::{InvokerControl, InvokerExit},
	standard::{Config, InvokerState, Resolver, ResolverOrigin, SubstackInvoke},
};

/// Mark the current address as hot, and attempt the given transfer.
fn mark_hot_and_transfer<H, S>(
	state: &S,
	transfer: Option<Transfer>,
	handler: &mut H,
) -> Result<(), ExitError>
where
	S: AsRef<RuntimeState>,
	H: RuntimeBackend + TransactionalBackend,
{
	handler.mark_hot(
		AsRef::<RuntimeState>::as_ref(state).context.address,
		TouchKind::StateChange,
	);

	if let Some(transfer) = transfer {
		handler.transfer(transfer)?;
	}

	Ok(())
}

/// Routine for making a call interpreter.
///
/// It will:
/// * Mark the current address as hot.
/// * Attempt transfer.
/// * Use [Resolver] to resolve the call into a new interpreter in the sub call stack.
#[allow(clippy::too_many_arguments)]
pub fn make_enter_call_machine<H, R>(
	origin: ResolverOrigin,
	resolver: &R,
	scheme: CallScheme,
	code_address: H160,
	input: Vec<u8>,
	transfer: Option<Transfer>,
	state: <R::Interpreter as Interpreter<H>>::State,
	handler: &mut H,
) -> Result<InvokerControl<R::Interpreter, R::State>, ExitError>
where
	R::State: AsRef<RuntimeState>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
{
	if let Err(err) = mark_hot_and_transfer(&state, transfer, handler) {
		return Ok(InvokerControl::DirectExit(InvokerExit {
			result: Err(err),
			substate: Some(state),
			retval: Vec::new(),
		}));
	}

	resolver.resolve_call(origin, scheme, code_address, input, state, handler)
}

/// Routine for making a call interpreter, with the call input as a
/// [`CallInput`].
///
/// It does what [make_enter_call_machine] does, and resolves the call with
/// [Resolver::resolve_call_with_input], so that a resolver that accepts a
/// [`CallInput`] does not have to copy the input.
#[allow(clippy::too_many_arguments)]
pub fn make_enter_call_machine_with_input<H, R>(
	origin: ResolverOrigin,
	resolver: &R,
	scheme: CallScheme,
	code_address: H160,
	input: CallInput,
	transfer: Option<Transfer>,
	state: <R::Interpreter as Interpreter<H>>::State,
	handler: &mut H,
) -> Result<InvokerControl<R::Interpreter, R::State>, ExitError>
where
	R::State: AsRef<RuntimeState>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
{
	if let Err(err) = mark_hot_and_transfer(&state, transfer, handler) {
		return Ok(InvokerControl::DirectExit(InvokerExit {
			result: Err(err),
			substate: Some(state),
			retval: Vec::new(),
		}));
	}

	resolver.resolve_call_with_input(origin, scheme, code_address, input, state, handler)
}

/// Routine for making a create interpreter.
///
/// It will:
/// * Mark the current address as hot.
/// * Check for init size limit.
/// * Attempt transfer.
/// * Check create collision.
/// * Increase nonce of the current address.
/// * Reset state of the current address.
/// * And finally, use [Resolver] to resolve the call into a new interpreter in the sub call stack.
#[allow(clippy::too_many_arguments)]
pub fn make_enter_create_machine<H, R>(
	origin: ResolverOrigin,
	resolver: &R,
	_caller: H160,
	init_code: Vec<u8>,
	transfer: Transfer,
	state: <R::Interpreter as Interpreter<H>>::State,
	handler: &mut H,
) -> Result<InvokerControl<R::Interpreter, R::State>, ExitError>
where
	R::State: AsRef<RuntimeState> + AsRef<Config>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
{
	handler.mark_hot(
		AsRef::<RuntimeState>::as_ref(&state).context.address,
		TouchKind::StateChange,
	);

	match handler.transfer(transfer) {
		Ok(()) => (),
		Err(err) => {
			return Ok(InvokerControl::DirectExit(InvokerExit {
				result: Err(err),
				substate: Some(state),
				retval: Vec::new(),
			}));
		}
	}

	if !handler.can_create(AsRef::<RuntimeState>::as_ref(&state).context.address) {
		return Ok(InvokerControl::DirectExit(InvokerExit {
			result: Err(ExitException::CreateCollision.into()),
			substate: Some(state),
			retval: Vec::new(),
		}));
	}

	if AsRef::<Config>::as_ref(&state).eip161_create_increase_nonce {
		match handler.inc_nonce(AsRef::<RuntimeState>::as_ref(&state).context.address) {
			Ok(()) => (),
			Err(err) => {
				return Ok(InvokerControl::DirectExit(InvokerExit {
					result: Err(err),
					substate: Some(state),
					retval: Vec::new(),
				}));
			}
		}
	}

	handler.reset_storage(AsRef::<RuntimeState>::as_ref(&state).context.address);
	handler.mark_create(AsRef::<RuntimeState>::as_ref(&state).context.address);

	resolver.resolve_create(origin, init_code, state, handler)
}

/// Enter a call substack, resolving the call with the given routine.
#[allow(clippy::type_complexity)]
fn enter_call_substack_inner<H, R, F>(
	resolver: &R,
	trap_data: CallTrap,
	state: R::State,
	handler: &mut H,
	enter: F,
) -> Result<(SubstackInvoke, InvokerControl<R::Interpreter, R::State>), ExitError>
where
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
	F: FnOnce(
		&R,
		&CallTrap,
		R::State,
		&mut H,
	) -> Result<InvokerControl<R::Interpreter, R::State>, ExitError>,
{
	handler.push_substate();

	let res = enter(resolver, &trap_data, state, handler);
	let invoke = SubstackInvoke::Call { trap: trap_data };

	match res {
		Ok(machine) => Ok((invoke, machine)),
		Err(err) => Ok((
			invoke,
			InvokerControl::DirectExit(InvokerExit {
				result: Err(err),
				substate: None,
				retval: Vec::new(),
			}),
		)),
	}
}

/// Enter a call substack, which internally calls [make_enter_call_machine].
///
/// The trap takes its input from `trap_data.input`, so this must not be used
/// for a trap from a machine that shares call input, whose `input` is empty.
/// Use [enter_call_substack_with_input] for such a trap.
#[allow(clippy::type_complexity)]
pub fn enter_call_substack<H, R>(
	resolver: &R,
	trap_data: CallTrap,
	code_address: H160,
	state: R::State,
	handler: &mut H,
) -> Result<(SubstackInvoke, InvokerControl<R::Interpreter, R::State>), ExitError>
where
	R::State: AsRef<RuntimeState>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
{
	#[allow(deprecated)]
	let input = trap_data.input.clone();

	enter_call_substack_inner(
		resolver,
		trap_data,
		state,
		handler,
		|resolver, trap_data, state, handler| {
			make_enter_call_machine(
				ResolverOrigin::Substack,
				resolver,
				trap_data.scheme,
				code_address,
				input,
				trap_data.transfer.clone(),
				state,
				handler,
			)
		},
	)
}

/// Enter a call substack with the given call input, which internally calls
/// [make_enter_call_machine_with_input].
///
/// The trap is kept as it is, so its `input` stays empty for a trap from a
/// machine that shares call input.
#[allow(clippy::type_complexity)]
pub fn enter_call_substack_with_input<H, R>(
	resolver: &R,
	trap_data: CallTrap,
	input: CallInput,
	code_address: H160,
	state: R::State,
	handler: &mut H,
) -> Result<(SubstackInvoke, InvokerControl<R::Interpreter, R::State>), ExitError>
where
	R::State: AsRef<RuntimeState>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
{
	enter_call_substack_inner(
		resolver,
		trap_data,
		state,
		handler,
		|resolver, trap_data, state, handler| {
			make_enter_call_machine_with_input(
				ResolverOrigin::Substack,
				resolver,
				trap_data.scheme,
				code_address,
				input,
				trap_data.transfer.clone(),
				state,
				handler,
			)
		},
	)
}

/// Enter a create substack, which internally calls [make_enter_create_machine].
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn enter_create_substack<H, R>(
	resolver: &R,
	code: Vec<u8>,
	trap_data: CreateTrap,
	address: H160,
	state: R::State,
	handler: &mut H,
) -> Result<(SubstackInvoke, InvokerControl<R::Interpreter, R::State>), ExitError>
where
	R::State: AsRef<RuntimeState> + AsRef<Config>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
{
	handler.push_substate();

	let scheme = trap_data.scheme;
	let value = trap_data.value;
	let caller = scheme.caller();

	let transfer = Transfer {
		source: caller,
		target: address,
		value,
	};

	let invoke = SubstackInvoke::Create {
		address,
		trap: trap_data,
	};
	let machine = make_enter_create_machine(
		ResolverOrigin::Substack,
		resolver,
		caller,
		code,
		transfer,
		state,
		handler,
	)?;

	Ok((invoke, machine))
}

fn check_first_byte(config: &Config, code: &[u8]) -> Result<(), ExitError> {
	if config.eip3541_disallow_executable_format && Some(&Opcode::EOFMAGIC.as_u8()) == code.first()
	{
		return Err(ExitException::InvalidOpcode(Opcode::EOFMAGIC).into());
	}
	Ok(())
}

/// Routine for deploying the create code.
pub fn deploy_create_code<S, H>(
	address: H160,
	retbuf: Vec<u8>,
	state: &mut S,
	handler: &mut H,
	origin: SetCodeOrigin,
) -> Result<(), ExitError>
where
	S: InvokerState + AsRef<Config>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	check_first_byte(state.as_ref(), &retbuf[..])?;

	if let Some(limit) = AsRef::<Config>::as_ref(state).create_contract_limit()
		&& retbuf.len() > limit
	{
		return Err(ExitException::CreateContractLimit.into());
	}

	state.record_codedeposit(retbuf.len())?;

	handler.set_code(address, retbuf, origin)?;

	Ok(())
}
