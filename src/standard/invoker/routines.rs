use alloc::vec::Vec;
use evm_interpreter::{
	error::{CallScheme, CallTrap, CreateTrap, ExitError, ExitException},
	opcode::Opcode,
	Interpreter,
	runtime::{
		RuntimeBackend, RuntimeEnvironment, RuntimeState, SetCodeOrigin, TouchKind, Transfer,
	},
};
use primitive_types::{H160, U256};

use crate::{
	backend::TransactionalBackend,
	invoker::{InvokerControl, InvokerExit},
	standard::{Config, InvokerState, Resolver, SubstackInvoke},
};

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn make_enter_call_machine<H, R>(
	_config: &Config,
	resolver: &R,
	scheme: CallScheme,
	code_address: H160,
	input: Vec<u8>,
	transfer: Option<Transfer>,
	state: <R::Interpreter as Interpreter<H>>::State,
	handler: &mut H,
) -> Result<InvokerControl<R::Interpreter, <R::Interpreter as Interpreter<H>>::State>, ExitError>
where
	<R::Interpreter as Interpreter<H>>::State: AsRef<RuntimeState>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
{
	handler.mark_hot(state.as_ref().context.address, TouchKind::StateChange);

	if let Some(transfer) = transfer {
		match handler.transfer(transfer) {
			Ok(()) => (),
			Err(err) => {
				return Ok(InvokerControl::DirectExit(InvokerExit {
					result: Err(err),
					substate: Some(state),
					retval: Vec::new(),
				}))
			}
		}
	}

	resolver.resolve_call(scheme, code_address, input, state, handler)
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn make_enter_create_machine<H, R>(
	config: &Config,
	resolver: &R,
	_caller: H160,
	init_code: Vec<u8>,
	transfer: Transfer,
	state: <R::Interpreter as Interpreter<H>>::State,
	handler: &mut H,
) -> Result<InvokerControl<R::Interpreter, <R::Interpreter as Interpreter<H>>::State>, ExitError>
where
	<R::Interpreter as Interpreter<H>>::State: AsRef<RuntimeState>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
{
	handler.mark_hot(state.as_ref().context.address, TouchKind::StateChange);

	if let Some(limit) = config.max_initcode_size {
		if init_code.len() > limit {
			return Ok(InvokerControl::DirectExit(InvokerExit {
				result: Err(ExitException::CreateContractLimit.into()),
				substate: Some(state),
				retval: Vec::new(),
			}));
		}
	}

	match handler.transfer(transfer) {
		Ok(()) => (),
		Err(err) => {
			return Ok(InvokerControl::DirectExit(InvokerExit {
				result: Err(err),
				substate: Some(state),
				retval: Vec::new(),
			}))
		}
	}

	if handler.code_size(state.as_ref().context.address) != U256::zero()
		|| handler.nonce(state.as_ref().context.address) > U256::zero()
	{
		return Ok(InvokerControl::DirectExit(InvokerExit {
			result: Err(ExitException::CreateCollision.into()),
			substate: Some(state),
			retval: Vec::new(),
		}));
	}

	if config.create_increase_nonce {
		match handler.inc_nonce(state.as_ref().context.address) {
			Ok(()) => (),
			Err(err) => {
				return Ok(InvokerControl::DirectExit(InvokerExit {
					result: Err(err),
					substate: Some(state),
					retval: Vec::new(),
				}))
			}
		}
	}

	handler.reset_storage(state.as_ref().context.address);
	handler.mark_create(state.as_ref().context.address);

	resolver.resolve_create(init_code, state, handler)
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn enter_call_substack<H, R>(
	config: &Config,
	resolver: &R,
	trap_data: CallTrap,
	code_address: H160,
	state: <R::Interpreter as Interpreter<H>>::State,
	handler: &mut H,
) -> Result<(SubstackInvoke, InvokerControl<R::Interpreter, <R::Interpreter as Interpreter<H>>::State>), ExitError>
where
	<R::Interpreter as Interpreter<H>>::State: AsRef<RuntimeState>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
{
	handler.push_substate();

	let work = || -> Result<_, ExitError> {
		let machine = make_enter_call_machine(
			config,
			resolver,
			trap_data.scheme,
			code_address,
			trap_data.input.clone(),
			trap_data.transfer.clone(),
			state,
			handler,
		)?;

		Ok(machine)
	};

	let res = work();
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

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn enter_create_substack<H, R>(
	config: &Config,
	resolver: &R,
	code: Vec<u8>,
	trap_data: CreateTrap,
	address: H160,
	state: <R::Interpreter as Interpreter<H>>::State,
	handler: &mut H,
) -> Result<(SubstackInvoke, InvokerControl<R::Interpreter, <R::Interpreter as Interpreter<H>>::State>), ExitError>
where
	<R::Interpreter as Interpreter<H>>::State: AsRef<RuntimeState>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
{
	handler.push_substate();

	let scheme = trap_data.scheme.clone();
	let value = trap_data.value.clone();
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
	let machine =
		make_enter_create_machine(config, resolver, caller, code, transfer, state, handler)?;

	Ok((invoke, machine))
}

fn check_first_byte(config: &Config, code: &[u8]) -> Result<(), ExitError> {
	if config.disallow_executable_format && Some(&Opcode::EOFMAGIC.as_u8()) == code.first() {
		return Err(ExitException::InvalidOpcode(Opcode::EOFMAGIC).into());
	}
	Ok(())
}

pub fn deploy_create_code<'config, S, H>(
	config: &Config,
	address: H160,
	retbuf: Vec<u8>,
	state: &mut S,
	handler: &mut H,
	origin: SetCodeOrigin,
) -> Result<(), ExitError>
where
	S: InvokerState<'config>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	check_first_byte(config, &retbuf[..])?;

	if let Some(limit) = config.create_contract_limit {
		if retbuf.len() > limit {
			return Err(ExitException::CreateContractLimit.into());
		}
	}

	state.record_codedeposit(retbuf.len())?;

	handler.set_code(address, retbuf, origin)?;

	Ok(())
}
