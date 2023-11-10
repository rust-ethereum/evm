use super::{gasometer::TransactionCost, Config, Etable, GasedMachine, Gasometer, Machine};
use crate::call_create::{CallCreateTrapData, CallTrapData, CreateTrapData};
use crate::{
	Capture, Context, ExitError, ExitException, ExitResult, Gasometer as GasometerT,
	GasometerMergeStrategy, Invoker as InvokerT, Opcode, RuntimeHandle, RuntimeState,
	TransactionContext, TransactionalBackend, TransactionalMergeStrategy, Transfer,
};
use alloc::rc::Rc;
use core::cmp::min;
use core::convert::Infallible;
use primitive_types::{H160, H256, U256};

pub enum CallCreateTrapPrepareData {
	Call {
		gas_limit: u64,
		is_static: bool,
		transaction_context: Rc<TransactionContext>,
		trap: CallTrapData,
	},
	Create {
		gas_limit: u64,
		is_static: bool,
		transaction_context: Rc<TransactionContext>,
		trap: CreateTrapData,
	},
}

pub enum CallCreateTrapEnterData {
	Call { trap: CallTrapData },
	Create { trap: CreateTrapData, address: H160 },
}

const DEFAULT_HEAP_DEPTH: Option<usize> = Some(4);

pub struct Invoker<'config> {
	config: &'config Config,
}

impl<'config> Invoker<'config> {
	pub fn new(config: &'config Config) -> Self {
		Self { config }
	}

	pub fn transact_call<H>(
		&self,
		caller: H160,
		address: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: U256,
		gas_price: U256,
		access_list: Vec<(H160, Vec<H256>)>,
		handler: &mut H,
		etable: &Etable<H>,
	) -> ExitResult
	where
		H: RuntimeHandle + TransactionalBackend,
	{
		handler.push_substate();

		let work = || -> ExitResult {
			let gas_fee = gas_limit.saturating_mul(gas_price);
			handler.transfer(Transfer {
				source: caller,
				target: handler.block_coinbase(),
				value: gas_fee,
			})?;

			let gas_limit = if gas_limit > U256::from(u64::MAX) {
				return Err(ExitException::OutOfGas.into());
			} else {
				gas_limit.as_u64()
			};

			let context = Context {
				caller,
				address,
				apparent_value: value,
			};
			let code = handler.code(address);

			let transaction_cost = TransactionCost::call(&data, &access_list).cost(self.config);

			let machine = Machine::new(
				Rc::new(code),
				Rc::new(data),
				self.config.stack_limit,
				self.config.memory_limit,
				RuntimeState {
					context,
					transaction_context: TransactionContext {
						origin: caller,
						gas_price,
					}
					.into(),
					retbuf: Vec::new(),
					gas: U256::zero(),
				},
			);
			let mut gasometer = Gasometer::new(gas_limit, &machine, self.config);

			gasometer.record_cost(transaction_cost)?;

			if self.config.increase_state_access_gas {
				if self.config.warm_coinbase_address {
					let coinbase = handler.block_coinbase();
					handler.mark_hot(coinbase, None)?;
				}
				handler.mark_hot(caller, None)?;
				handler.mark_hot(address, None)?;
			}

			handler.inc_nonce(caller)?;

			let transfer = Transfer {
				source: caller,
				target: address,
				value,
			};
			handler.transfer(transfer)?;

			let machine = GasedMachine {
				machine,
				gasometer,
				is_static: false,
			};

			let (machine, result) =
				crate::execute(machine, handler, 0, DEFAULT_HEAP_DEPTH, self, etable);

			let refunded_gas = U256::from(machine.gasometer.gas());
			let refunded_fee = refunded_gas * gas_price;
			handler.transfer(Transfer {
				source: handler.block_coinbase(),
				target: caller,
				value: refunded_fee,
			})?;

			result
		};

		match work() {
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
}

impl<'config, H> InvokerT<RuntimeState, Gasometer<'config>, H, Opcode> for Invoker<'config>
where
	H: RuntimeHandle + TransactionalBackend,
{
	type Interrupt = Infallible;
	type CallCreateTrapPrepareData = CallCreateTrapPrepareData;
	type CallCreateTrapEnterData = CallCreateTrapEnterData;

	fn prepare_trap(
		&self,
		opcode: Opcode,
		machine: &mut GasedMachine<'config>,
		_handler: &mut H,
		depth: usize,
	) -> Capture<Result<Self::CallCreateTrapPrepareData, ExitError>, Infallible> {
		fn l64(gas: u64) -> u64 {
			gas - gas / 64
		}

		if depth >= self.config.call_stack_limit {
			return Capture::Exit(Err(ExitException::CallTooDeep.into()));
		}

		let trap_data = match CallCreateTrapData::new_from(opcode, &mut machine.machine) {
			Ok(trap_data) => trap_data,
			Err(err) => return Capture::Exit(Err(err)),
		};

		let after_gas = U256::from(if self.config.call_l64_after_gas {
			l64(machine.gasometer.gas())
		} else {
			machine.gasometer.gas()
		});
		let target_gas = trap_data.target_gas().unwrap_or(after_gas);
		let gas_limit = min(after_gas, target_gas);

		let gas_limit = if gas_limit > U256::from(u64::MAX) {
			return Capture::Exit(Err(ExitException::OutOfGas.into()));
		} else {
			gas_limit.as_u64()
		};

		match machine.gasometer.record_cost(gas_limit) {
			Ok(()) => (),
			Err(err) => return Capture::Exit(Err(err)),
		}

		let is_static = if machine.is_static {
			true
		} else {
			match &trap_data {
				CallCreateTrapData::Call(CallTrapData { is_static, .. }) => *is_static,
				_ => false,
			}
		};

		let transaction_context = machine.machine.state.as_ref().transaction_context.clone();

		Capture::Exit(Ok(match trap_data {
			CallCreateTrapData::Call(call_trap_data) => CallCreateTrapPrepareData::Call {
				gas_limit,
				is_static,
				transaction_context,
				trap: call_trap_data,
			},
			CallCreateTrapData::Create(create_trap_data) => CallCreateTrapPrepareData::Create {
				gas_limit,
				is_static,
				transaction_context,
				trap: create_trap_data,
			},
		}))
	}

	fn enter_trap_stack(
		&self,
		trap_data: Self::CallCreateTrapPrepareData,
		handler: &mut H,
	) -> Result<(Self::CallCreateTrapEnterData, GasedMachine<'config>), ExitError> {
		match trap_data {
			CallCreateTrapPrepareData::Create {
				gas_limit,
				is_static,
				transaction_context,
				trap,
			} => enter_create_trap_stack(
				gas_limit,
				is_static,
				transaction_context,
				trap,
				handler,
				self.config,
			),
			CallCreateTrapPrepareData::Call {
				gas_limit,
				is_static,
				transaction_context,
				trap,
			} => enter_call_trap_stack(
				gas_limit,
				is_static,
				transaction_context,
				trap,
				handler,
				self.config,
			),
		}
	}

	fn exit_trap_stack(
		&self,
		result: ExitResult,
		mut child: GasedMachine<'config>,
		trap_data: Self::CallCreateTrapEnterData,
		parent: &mut GasedMachine<'config>,
		handler: &mut H,
	) -> Result<(), ExitError> {
		match trap_data {
			CallCreateTrapEnterData::Create { address, trap } => {
				let retbuf = child.machine.into_retbuf();
				let result = exit_create_trap_stack_no_exit_substate(
					result.map(|_| address),
					&retbuf,
					&mut child.gasometer,
					handler,
					self.config,
				);

				match &result {
					Ok(_) => {
						handler.pop_substate(TransactionalMergeStrategy::Commit);
						GasometerT::<RuntimeState, H>::merge(
							&mut parent.gasometer,
							child.gasometer,
							GasometerMergeStrategy::Commit,
						);
					}
					Err(ExitError::Reverted) => {
						handler.pop_substate(TransactionalMergeStrategy::Discard);
						GasometerT::<RuntimeState, H>::merge(
							&mut parent.gasometer,
							child.gasometer,
							GasometerMergeStrategy::Revert,
						);
					}
					Err(_) => {
						handler.pop_substate(TransactionalMergeStrategy::Discard);
					}
				};

				trap.feedback(result, retbuf, &mut parent.machine)?;

				Ok(())
			}
			CallCreateTrapEnterData::Call { trap } => {
				let retbuf = child.machine.into_retbuf();

				match &result {
					Ok(_) => {
						handler.pop_substate(TransactionalMergeStrategy::Commit);
						GasometerT::<RuntimeState, H>::merge(
							&mut parent.gasometer,
							child.gasometer,
							GasometerMergeStrategy::Commit,
						);
					}
					Err(ExitError::Reverted) => {
						handler.pop_substate(TransactionalMergeStrategy::Discard);
						GasometerT::<RuntimeState, H>::merge(
							&mut parent.gasometer,
							child.gasometer,
							GasometerMergeStrategy::Revert,
						);
					}
					Err(_) => {
						handler.pop_substate(TransactionalMergeStrategy::Discard);
					}
				};

				trap.feedback(result, retbuf, &mut parent.machine)?;

				Ok(())
			}
		}
	}
}

fn enter_create_trap_stack<'config, H>(
	gas_limit: u64,
	is_static: bool,
	transaction_context: Rc<TransactionContext>,
	trap_data: CreateTrapData,
	handler: &mut H,
	config: &'config Config,
) -> Result<(CallCreateTrapEnterData, GasedMachine<'config>), ExitError>
where
	H: RuntimeHandle + TransactionalBackend,
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

		handler.mark_hot(caller, None)?;
		handler.mark_hot(address, None)?;

		if handler.balance(caller) < value {
			return Err(ExitException::OutOfFund.into());
		}

		handler.inc_nonce(caller)?;

		if handler.code_size(address) != U256::zero() || handler.nonce(address) > U256::zero() {
			return Err(ExitException::CreateCollision.into());
		}

		handler.reset_storage(address);

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

		handler.transfer(transfer)?;

		if config.create_increase_nonce {
			handler.inc_nonce(address)?;
		}

		let machine = Machine::new(
			Rc::new(code),
			Rc::new(Vec::new()),
			config.stack_limit,
			config.memory_limit,
			RuntimeState {
				context,
				transaction_context,
				retbuf: Vec::new(),
				gas: U256::zero(),
			},
		);

		let gasometer = Gasometer::new(gas_limit, &machine, config);

		Ok((
			CallCreateTrapEnterData::Create {
				address,
				trap: trap_data,
			},
			GasedMachine {
				machine,
				gasometer,
				is_static,
			},
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

fn exit_create_trap_stack_no_exit_substate<'config, H>(
	result: Result<H160, ExitError>,
	retbuf: &Vec<u8>,
	gasometer: &mut Gasometer<'config>,
	handler: &mut H,
	config: &'config Config,
) -> Result<H160, ExitError>
where
	H: RuntimeHandle + TransactionalBackend,
{
	fn check_first_byte(config: &Config, code: &[u8]) -> Result<(), ExitError> {
		if config.disallow_executable_format && Some(&Opcode::EOFMAGIC.as_u8()) == code.first() {
			return Err(ExitException::InvalidOpcode(Opcode::EOFMAGIC).into());
		}
		Ok(())
	}

	let address = result?;
	check_first_byte(config, &retbuf[..])?;

	if let Some(limit) = config.create_contract_limit {
		if retbuf.len() > limit {
			return Err(ExitException::CreateContractLimit.into());
		}
	}

	GasometerT::<RuntimeState, H>::record_codedeposit(gasometer, retbuf.len())?;

	handler.set_code(address, retbuf.clone());

	Ok(address)
}

fn enter_call_trap_stack<'config, H>(
	mut gas_limit: u64,
	is_static: bool,
	transaction_context: Rc<TransactionContext>,
	trap_data: CallTrapData,
	handler: &mut H,
	config: &'config Config,
) -> Result<(CallCreateTrapEnterData, GasedMachine<'config>), ExitError>
where
	H: RuntimeHandle + TransactionalBackend,
{
	handler.push_substate();

	let work = || -> Result<(CallCreateTrapEnterData, GasedMachine<'config>), ExitError> {
		handler.mark_hot(trap_data.context.address, None)?;
		let code = handler.code(trap_data.target);

		if let Some(transfer) = trap_data.transfer.clone() {
			if transfer.value != U256::zero() {
				gas_limit = gas_limit.saturating_add(config.call_stipend);
			}

			handler.transfer(transfer)?;
		}

		// TODO: precompile contracts.

		let machine = Machine::new(
			Rc::new(code),
			Rc::new(trap_data.input.clone()),
			config.stack_limit,
			config.memory_limit,
			RuntimeState {
				context: trap_data.context.clone(),
				transaction_context,
				retbuf: Vec::new(),
				gas: U256::zero(),
			},
		);

		let gasometer = Gasometer::new(gas_limit, &machine, config);

		Ok((
			CallCreateTrapEnterData::Call { trap: trap_data },
			GasedMachine {
				machine,
				gasometer,
				is_static,
			},
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
