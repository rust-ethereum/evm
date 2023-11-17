use super::{CallTrapData, CreateTrapData, Invoker, SubstackInvoke};
use crate::standard::{Config, MergeableRuntimeState};
use crate::{
	ExitError, ExitException, GasedMachine, Gasometer as GasometerT, Machine, MergeStrategy,
	Opcode, RuntimeBackend, RuntimeEnvironment, TransactionalBackend, Transfer,
};
use alloc::rc::Rc;
use primitive_types::{H160, U256};

pub fn make_enter_call_machine<'config, S, G, H>(
	invoker: &Invoker<'config>,
	code: Vec<u8>,
	input: Vec<u8>,
	is_static: bool,
	transfer: Option<Transfer>,
	state: S,
	gasometer: G,
	handler: &mut H,
) -> Result<GasedMachine<S, G>, ExitError>
where
	S: MergeableRuntimeState,
	G: GasometerT<S, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	handler.mark_hot(state.as_ref().context.address, None)?;

	if let Some(transfer) = transfer {
		handler.transfer(transfer)?;
	}

	// TODO: precompile contracts.

	let machine = Machine::<S>::new(
		Rc::new(code),
		Rc::new(input.clone()),
		invoker.config.stack_limit,
		invoker.config.memory_limit,
		state,
	);

	Ok(GasedMachine {
		machine,
		gasometer,
		is_static,
	})
}

pub fn make_enter_create_machine<'config, S, G, H>(
	invoker: &Invoker<'config>,
	caller: H160,
	init_code: Vec<u8>,
	is_static: bool,
	transfer: Transfer,
	state: S,
	gasometer: G,
	handler: &mut H,
) -> Result<GasedMachine<S, G>, ExitError>
where
	S: MergeableRuntimeState,
	G: GasometerT<S, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	if let Some(limit) = invoker.config.max_initcode_size {
		if init_code.len() > limit {
			return Err(ExitException::CreateContractLimit.into());
		}
	}

	handler.mark_hot(caller, None)?;
	handler.mark_hot(state.as_ref().context.address, None)?;

	handler.transfer(transfer)?;

	if handler.code_size(state.as_ref().context.address) != U256::zero()
		|| handler.nonce(state.as_ref().context.address) > U256::zero()
	{
		return Err(ExitException::CreateCollision.into());
	}
	handler.inc_nonce(caller)?;
	if invoker.config.create_increase_nonce {
		handler.inc_nonce(state.as_ref().context.address)?;
	}

	handler.reset_storage(state.as_ref().context.address);

	let machine = Machine::new(
		Rc::new(init_code),
		Rc::new(Vec::new()),
		invoker.config.stack_limit,
		invoker.config.memory_limit,
		state,
	);

	Ok(GasedMachine {
		machine,
		gasometer,
		is_static,
	})
}

pub fn enter_call_substack<'config, S, G, H>(
	invoker: &Invoker<'config>,
	code: Vec<u8>,
	trap_data: CallTrapData,
	is_static: bool,
	state: S,
	gasometer: G,
	handler: &mut H,
) -> Result<(SubstackInvoke, GasedMachine<S, G>), ExitError>
where
	S: MergeableRuntimeState,
	G: GasometerT<S, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	handler.push_substate();

	let work = || -> Result<(SubstackInvoke, GasedMachine<S, G>), ExitError> {
		let machine = make_enter_call_machine(
			invoker,
			code,
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

pub fn enter_create_substack<'config, S, G, H>(
	invoker: &Invoker<'config>,
	code: Vec<u8>,
	trap_data: CreateTrapData,
	is_static: bool,
	state: S,
	gasometer: G,
	handler: &mut H,
) -> Result<(SubstackInvoke, GasedMachine<S, G>), ExitError>
where
	S: MergeableRuntimeState,
	G: GasometerT<S, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	handler.push_substate();

	let work = || -> Result<(SubstackInvoke, GasedMachine<S, G>), ExitError> {
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
			invoker, caller, code, is_static, transfer, state, gasometer, handler,
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
	invoker: &Invoker<'config>,
	address: H160,
	retbuf: &Vec<u8>,
	gasometer: &mut G,
	handler: &mut H,
) -> Result<(), ExitError>
where
	S: MergeableRuntimeState,
	G: GasometerT<S, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	check_first_byte(invoker.config, &retbuf[..])?;

	if let Some(limit) = invoker.config.create_contract_limit {
		if retbuf.len() > limit {
			return Err(ExitException::CreateContractLimit.into());
		}
	}

	GasometerT::<S, H>::record_codedeposit(gasometer, retbuf.len())?;

	handler.set_code(address, retbuf.clone());

	Ok(())
}
