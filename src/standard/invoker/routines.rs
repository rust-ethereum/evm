use alloc::vec::Vec;

use evm_interpreter::{
	error::{CallTrapData, CreateTrapData, ExitError, ExitException, ExitResult},
	opcode::Opcode,
	runtime::{RuntimeBackend, RuntimeEnvironment, RuntimeState, Transfer},
};
use primitive_types::{H160, U256};

use crate::{
	backend::TransactionalBackend,
	invoker::InvokerControl,
	standard::{Config, InvokerState, Resolver, SubstackInvoke},
	MergeStrategy,
};

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn make_enter_call_machine<H, R>(
	_config: &Config,
	resolver: &R,
	code_address: H160,
	input: Vec<u8>,
	transfer: Option<Transfer>,
	state: R::State,
	handler: &mut H,
) -> Result<InvokerControl<R::Interpreter, (ExitResult, (R::State, Vec<u8>))>, ExitError>
where
	R::State: AsRef<RuntimeState>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
{
	handler.mark_hot(state.as_ref().context.address, None);

	if let Some(transfer) = transfer {
		handler.transfer(transfer)?;
	}

	resolver.resolve_call(code_address, input, state, handler)
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn make_enter_create_machine<H, R>(
	config: &Config,
	resolver: &R,
	caller: H160,
	init_code: Vec<u8>,
	transfer: Transfer,
	state: R::State,
	handler: &mut H,
) -> Result<InvokerControl<R::Interpreter, (ExitResult, (R::State, Vec<u8>))>, ExitError>
where
	R::State: AsRef<RuntimeState>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
{
	if let Some(limit) = config.max_initcode_size {
		if init_code.len() > limit {
			return Err(ExitException::CreateContractLimit.into());
		}
	}

	handler.mark_hot(caller, None);
	handler.mark_hot(state.as_ref().context.address, None);

	handler.transfer(transfer)?;

	if handler.code_size(state.as_ref().context.address) != U256::zero()
		|| handler.nonce(state.as_ref().context.address) > U256::zero()
	{
		return Err(ExitException::CreateCollision.into());
	}
	handler.inc_nonce(caller)?;
	if config.create_increase_nonce {
		handler.inc_nonce(state.as_ref().context.address)?;
	}

	handler.reset_storage(state.as_ref().context.address);

	resolver.resolve_create(init_code, state, handler)
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn enter_call_substack<H, R>(
	config: &Config,
	resolver: &R,
	trap_data: CallTrapData,
	code_address: H160,
	state: R::State,
	handler: &mut H,
) -> Result<
	(
		SubstackInvoke,
		InvokerControl<R::Interpreter, (ExitResult, (R::State, Vec<u8>))>,
	),
	ExitError,
>
where
	R::State: AsRef<RuntimeState>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
{
	handler.push_substate();

	let work = || -> Result<(SubstackInvoke, _), ExitError> {
		let machine = make_enter_call_machine(
			config,
			resolver,
			code_address,
			trap_data.input.clone(),
			trap_data.transfer.clone(),
			state,
			handler,
		)?;

		Ok((SubstackInvoke::Call { trap: trap_data }, machine))
	};

	match work() {
		Ok(machine) => Ok(machine),
		Err(err) => {
			handler.pop_substate(MergeStrategy::Discard);
			Err(err)
		}
	}
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn enter_create_substack<H, R>(
	config: &Config,
	resolver: &R,
	code: Vec<u8>,
	trap_data: CreateTrapData,
	state: R::State,
	handler: &mut H,
) -> Result<
	(
		SubstackInvoke,
		InvokerControl<R::Interpreter, (ExitResult, (R::State, Vec<u8>))>,
	),
	ExitError,
>
where
	R::State: AsRef<RuntimeState>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
{
	handler.push_substate();

	let work = || -> Result<(SubstackInvoke, InvokerControl<R::Interpreter, (ExitResult, (R::State, Vec<u8>))>), ExitError> {
		let CreateTrapData {
			scheme,
			value,
			code: _,
		} = trap_data.clone();

		let caller = scheme.caller();
		let address = scheme.address(handler);

		let transfer = Transfer {
			source: caller,
			target: address,
			value,
		};

		let machine = make_enter_create_machine(
			config, resolver, caller, code, transfer, state, handler,
		)?;

		Ok((
			SubstackInvoke::Create {
				address,
				trap: trap_data,
			},
			machine,
		))
	};

	match work() {
		Ok(machine) => Ok(machine),
		Err(err) => {
			handler.pop_substate(MergeStrategy::Discard);
			Err(err)
		}
	}
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

	handler.set_code(address, retbuf)?;

	Ok(())
}
