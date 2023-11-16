use super::{CallCreateTrapEnterData, CallTrapData, CreateTrapData, Invoker};
use crate::standard::{Config, GasedMachine, Machine};
use crate::{
	Context, ExitError, ExitException, Gasometer as GasometerT, Opcode, RuntimeBackend,
	RuntimeEnvironment, RuntimeState, TransactionContext, TransactionalBackend,
	TransactionalMergeStrategy, Transfer,
};
use alloc::rc::Rc;
use primitive_types::{H160, U256};

macro_rules! try_or_oog {
	($e:expr) => {
		match $e {
			Ok(v) => v,
			Err(e) => return (Err(e), ::primitive_types::U256::zero()),
		}
	}
}
pub(crate) use try_or_oog;

pub fn transact_and_work<'config, H, R, F>(
	_invoker: &Invoker<'config>,
	caller: H160,
	gas_limit: U256,
	gas_price: U256,
	handler: &mut H,
	f: F,
) -> Result<R, ExitError>
where
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	F: FnOnce(&mut H) -> (Result<R, ExitError>, U256),
{
	let gas_fee = gas_limit.saturating_mul(gas_price);
	handler.withdrawal(caller, gas_fee)?;

	handler.push_substate();

	let (result, refunded_gas) = f(handler);
	let refunded_fee = refunded_gas.saturating_mul(gas_price);
	let coinbase_reward = gas_fee.saturating_sub(refunded_fee);

	handler.deposit(caller, refunded_fee);
	handler.deposit(handler.block_coinbase(), coinbase_reward);

	match result {
		Ok(exit) => {
			handler.pop_substate(TransactionalMergeStrategy::Commit);
			Ok(exit)
		}
		Err(err) => {
			handler.pop_substate(TransactionalMergeStrategy::Discard);
			Err(err)
		}
	}
}

pub fn make_enter_call_machine<'config, G, H>(
	invoker: &Invoker<'config>,
	code: Vec<u8>,
	input: Vec<u8>,
	is_static: bool,
	transfer: Option<Transfer>,
	context: Context,
	transaction_context: Rc<TransactionContext>,
	gasometer: G,
	handler: &mut H,
) -> Result<GasedMachine<G>, ExitError>
where
	G: GasometerT<RuntimeState, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	handler.mark_hot(context.address, None)?;

	if let Some(transfer) = transfer {
		handler.transfer(transfer)?;
	}

	// TODO: precompile contracts.

	let machine = Machine::new(
		Rc::new(code),
		Rc::new(input.clone()),
		invoker.config.stack_limit,
		invoker.config.memory_limit,
		RuntimeState {
			context: context.clone(),
			transaction_context,
			retbuf: Vec::new(),
			gas: U256::zero(),
		},
	);

	Ok(GasedMachine {
		machine,
		gasometer,
		is_static,
	})
}

pub fn make_enter_create_machine<'config, G, H>(
	invoker: &Invoker<'config>,
	caller: H160,
	init_code: Vec<u8>,
	is_static: bool,
	transfer: Transfer,
	context: Context,
	transaction_context: Rc<TransactionContext>,
	gasometer: G,
	handler: &mut H,
) -> Result<GasedMachine<G>, ExitError>
where
	G: GasometerT<RuntimeState, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	if let Some(limit) = invoker.config.max_initcode_size {
		if init_code.len() > limit {
			return Err(ExitException::CreateContractLimit.into());
		}
	}

	handler.mark_hot(caller, None)?;
	handler.mark_hot(context.address, None)?;

	handler.transfer(transfer)?;

	if handler.code_size(context.address) != U256::zero()
		|| handler.nonce(context.address) > U256::zero()
	{
		return Err(ExitException::CreateCollision.into());
	}
	handler.inc_nonce(caller)?;
	if invoker.config.create_increase_nonce {
		handler.inc_nonce(context.address)?;
	}

	handler.reset_storage(context.address);

	let machine = Machine::new(
		Rc::new(init_code),
		Rc::new(Vec::new()),
		invoker.config.stack_limit,
		invoker.config.memory_limit,
		RuntimeState {
			context: context.clone(),
			transaction_context,
			retbuf: Vec::new(),
			gas: U256::zero(),
		},
	);

	Ok(GasedMachine {
		machine,
		gasometer,
		is_static,
	})
}

pub fn enter_call_trap_stack<'config, G, H>(
	invoker: &Invoker<'config>,
	code: Vec<u8>,
	trap_data: CallTrapData,
	is_static: bool,
	transaction_context: Rc<TransactionContext>,
	gasometer: G,
	handler: &mut H,
) -> Result<(CallCreateTrapEnterData, GasedMachine<G>), ExitError>
where
	G: GasometerT<RuntimeState, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	handler.push_substate();

	let work = || -> Result<(CallCreateTrapEnterData, GasedMachine<G>), ExitError> {
		let machine = make_enter_call_machine(
			invoker,
			code,
			trap_data.input.clone(),
			is_static,
			trap_data.transfer.clone(),
			trap_data.context.clone(),
			transaction_context,
			gasometer,
			handler,
		)?;

		Ok((CallCreateTrapEnterData::Call { trap: trap_data }, machine))
	};

	match work() {
		Ok(machine) => Ok(machine),
		Err(err) => {
			handler.pop_substate(TransactionalMergeStrategy::Discard);
			Err(err)
		}
	}
}

pub fn enter_create_trap_stack<'config, G, H>(
	invoker: &Invoker<'config>,
	code: Vec<u8>,
	trap_data: CreateTrapData,
	is_static: bool,
	transaction_context: Rc<TransactionContext>,
	gasometer: G,
	handler: &mut H,
) -> Result<(CallCreateTrapEnterData, GasedMachine<G>), ExitError>
where
	G: GasometerT<RuntimeState, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	handler.push_substate();

	let work = || -> Result<(CallCreateTrapEnterData, GasedMachine<G>), ExitError> {
		let CreateTrapData {
			scheme,
			value,
			code: _,
		} = trap_data.clone();

		let caller = scheme.caller();
		let address = scheme.address(handler);

		let context = Context {
			address,
			caller,
			apparent_value: value,
		};

		let transfer = Transfer {
			source: caller,
			target: address,
			value,
		};

		let machine = make_enter_create_machine(
			invoker,
			caller,
			code,
			is_static,
			transfer,
			context,
			transaction_context,
			gasometer,
			handler,
		)?;

		Ok((
			CallCreateTrapEnterData::Create {
				address,
				trap: trap_data,
			},
			machine,
		))
	};

	match work() {
		Ok(machine) => Ok(machine),
		Err(err) => {
			handler.pop_substate(TransactionalMergeStrategy::Discard);
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

pub fn deploy_create_code<'config, G, H>(
	invoker: &Invoker<'config>,
	result: Result<H160, ExitError>,
	retbuf: &Vec<u8>,
	gasometer: &mut G,
	handler: &mut H,
) -> Result<H160, ExitError>
where
	G: GasometerT<RuntimeState, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	let address = result?;
	check_first_byte(invoker.config, &retbuf[..])?;

	if let Some(limit) = invoker.config.create_contract_limit {
		if retbuf.len() > limit {
			return Err(ExitException::CreateContractLimit.into());
		}
	}

	GasometerT::<RuntimeState, H>::record_codedeposit(gasometer, retbuf.len())?;

	handler.set_code(address, retbuf.clone());

	Ok(address)
}
