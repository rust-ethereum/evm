use super::{CallTrapData, CreateTrapData, Resolver, SubstackInvoke, TransactGasometer};
use crate::standard::Config;
use crate::{
	ColoredMachine, ExitError, ExitException, ExitResult, Gasometer as GasometerT, InvokerControl,
	MergeStrategy, Opcode, RuntimeBackend, RuntimeEnvironment, RuntimeState, StaticGasometer,
	TransactionalBackend, Transfer,
};
use primitive_types::{H160, U256};

pub fn maybe_analyse_code<'config, S: AsRef<RuntimeState>, G: TransactGasometer<'config, S>, C>(
	result: &mut InvokerControl<ColoredMachine<S, G, C>, (ExitResult, (S, G, Vec<u8>))>,
) {
	if let InvokerControl::Enter(machine) = result {
		machine.gasometer.analyse_code(&machine.machine.code())
	}
}

pub fn make_enter_call_machine<'config, 'resolver, S, G, H, R, Tr>(
	_config: &'config Config,
	resolver: &'resolver R,
	code_address: H160,
	input: Vec<u8>,
	is_static: bool,
	transfer: Option<Transfer>,
	state: S,
	gasometer: G,
	handler: &mut H,
) -> Result<InvokerControl<ColoredMachine<S, G, R::Color>, (ExitResult, (S, G, Vec<u8>))>, ExitError>
where
	S: AsRef<RuntimeState>,
	G: GasometerT<S, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<S, G, H, Tr>,
{
	handler.mark_hot(state.as_ref().context.address, None);

	if let Some(transfer) = transfer {
		handler.transfer(transfer)?;
	}

	resolver.resolve_call(code_address, input, is_static, state, gasometer, handler)
}

pub fn make_enter_create_machine<'config, 'resolver, S, G, H, R, Tr>(
	config: &'config Config,
	resolver: &'resolver R,
	caller: H160,
	init_code: Vec<u8>,
	is_static: bool,
	transfer: Transfer,
	state: S,
	gasometer: G,
	handler: &mut H,
) -> Result<InvokerControl<ColoredMachine<S, G, R::Color>, (ExitResult, (S, G, Vec<u8>))>, ExitError>
where
	S: AsRef<RuntimeState>,
	G: GasometerT<S, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<S, G, H, Tr>,
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

	resolver.resolve_create(init_code, is_static, state, gasometer, handler)
}

pub fn enter_call_substack<'config, 'resolver, S, G, H, R, Tr>(
	config: &'config Config,
	resolver: &'resolver R,
	trap_data: CallTrapData,
	code_address: H160,
	is_static: bool,
	state: S,
	gasometer: G,
	handler: &mut H,
) -> Result<
	(
		SubstackInvoke,
		InvokerControl<ColoredMachine<S, G, R::Color>, (ExitResult, (S, G, Vec<u8>))>,
	),
	ExitError,
>
where
	S: AsRef<RuntimeState>,
	G: GasometerT<S, H> + TransactGasometer<'config, S>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<S, G, H, Tr>,
{
	handler.push_substate();

	let work = || -> Result<(SubstackInvoke, _), ExitError> {
		let machine = make_enter_call_machine(
			config,
			resolver,
			code_address,
			trap_data.input.clone(),
			is_static,
			trap_data.transfer.clone(),
			state,
			gasometer,
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

pub fn enter_create_substack<'config, 'resolver, S, G, H, R, Tr>(
	config: &'config Config,
	resolver: &'resolver R,
	code: Vec<u8>,
	trap_data: CreateTrapData,
	is_static: bool,
	state: S,
	gasometer: G,
	handler: &mut H,
) -> Result<
	(
		SubstackInvoke,
		InvokerControl<ColoredMachine<S, G, R::Color>, (ExitResult, (S, G, Vec<u8>))>,
	),
	ExitError,
>
where
	S: AsRef<RuntimeState>,
	G: GasometerT<S, H> + TransactGasometer<'config, S>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<S, G, H, Tr>,
{
	handler.push_substate();

	let work = || -> Result<(SubstackInvoke, InvokerControl<ColoredMachine<S, G, R::Color>, (ExitResult, (S, G, Vec<u8>))>), ExitError> {
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
			config, resolver, caller, code, is_static, transfer, state, gasometer, handler,
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

pub fn deploy_create_code<'config, S, G, H>(
	config: &'config Config,
	address: H160,
	retbuf: &Vec<u8>,
	gasometer: &mut G,
	handler: &mut H,
) -> Result<(), ExitError>
where
	G: GasometerT<S, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	check_first_byte(config, &retbuf[..])?;

	if let Some(limit) = config.create_contract_limit {
		if retbuf.len() > limit {
			return Err(ExitException::CreateContractLimit.into());
		}
	}

	StaticGasometer::record_codedeposit(gasometer, retbuf.len())?;

	handler.set_code(address, retbuf.clone())?;

	Ok(())
}
