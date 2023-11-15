use super::{CallCreateTrapEnterData, CallTrapData, CreateTrapData, Invoker};
use crate::standard::{Config, GasedMachine, Gasometer, Machine};
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

pub fn make_enter_call_machine<'config, H>(
	invoker: &Invoker<'config>,
	is_transaction: bool,
	target: H160,
	input: Vec<u8>,
	mut gas_limit: u64,
	is_static: bool,
	transfer: Option<Transfer>,
	context: Context,
	transaction_context: Rc<TransactionContext>,
	handler: &mut H,
) -> Result<GasedMachine<'config>, ExitError>
where
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	handler.mark_hot(context.address, None)?;
	let code = handler.code(target);

	if let Some(transfer) = transfer.clone() {
		if !is_transaction && transfer.value != U256::zero() {
			gas_limit = gas_limit.saturating_add(invoker.config.call_stipend);
		}

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

	let gasometer = Gasometer::new(gas_limit, &machine, invoker.config);

	Ok(GasedMachine {
		machine,
		gasometer,
		is_static,
	})
}

pub fn make_enter_create_machine<'config, H>(
	invoker: &Invoker<'config>,
	caller: H160,
	target: H160,
	init_code: Vec<u8>,
	gas_limit: u64,
	is_static: bool,
	transfer: Transfer,
	context: Context,
	transaction_context: Rc<TransactionContext>,
	handler: &mut H,
) -> Result<GasedMachine<'config>, ExitError>
where
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	if let Some(limit) = invoker.config.max_initcode_size {
		if init_code.len() > limit {
			return Err(ExitException::CreateContractLimit.into());
		}
	}

	handler.mark_hot(caller, None)?;
	handler.mark_hot(target, None)?;

	handler.transfer(transfer)?;

	if handler.code_size(target) != U256::zero() || handler.nonce(target) > U256::zero() {
		return Err(ExitException::CreateCollision.into());
	}
	handler.inc_nonce(caller)?;
	if invoker.config.create_increase_nonce {
		handler.inc_nonce(target)?;
	}

	handler.reset_storage(target);

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

	let gasometer = Gasometer::new(gas_limit, &machine, invoker.config);

	Ok(GasedMachine {
		machine,
		gasometer,
		is_static,
	})
}

pub fn enter_call_trap_stack<'config, H>(
	invoker: &Invoker<'config>,
	trap_data: CallTrapData,
	gas_limit: u64,
	is_static: bool,
	transaction_context: Rc<TransactionContext>,
	handler: &mut H,
) -> Result<(CallCreateTrapEnterData, GasedMachine<'config>), ExitError>
where
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	handler.push_substate();

	let work = || -> Result<(CallCreateTrapEnterData, GasedMachine<'config>), ExitError> {
		let machine = make_enter_call_machine(
			invoker,
			false, // is_transaction
			trap_data.target,
			trap_data.input.clone(),
			gas_limit,
			is_static,
			trap_data.transfer.clone(),
			trap_data.context.clone(),
			transaction_context,
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

pub fn enter_create_trap_stack<'config, H>(
	invoker: &Invoker<'config>,
	trap_data: CreateTrapData,
	gas_limit: u64,
	is_static: bool,
	transaction_context: Rc<TransactionContext>,
	handler: &mut H,
) -> Result<(CallCreateTrapEnterData, GasedMachine<'config>), ExitError>
where
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	handler.push_substate();

	let work = || -> Result<(CallCreateTrapEnterData, GasedMachine<'config>), ExitError> {
		let CreateTrapData {
			scheme,
			value,
			code,
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
			address,
			code,
			gas_limit,
			is_static,
			transfer,
			context,
			transaction_context,
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

pub fn deploy_create_code<'config, H>(
	invoker: &Invoker<'config>,
	result: Result<H160, ExitError>,
	retbuf: &Vec<u8>,
	gasometer: &mut Gasometer<'config>,
	handler: &mut H,
) -> Result<H160, ExitError>
where
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