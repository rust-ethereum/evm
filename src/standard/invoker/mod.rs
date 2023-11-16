mod routines;

use self::routines::try_or_oog;
use super::{Config, Etable, GasedMachine, TransactGasometer};
use crate::call_create::{CallCreateTrapData, CallTrapData, CreateScheme, CreateTrapData};
use crate::{
	Capture, Context, ExitError, ExitException, ExitResult, Gasometer as GasometerT,
	GasometerMergeStrategy, Invoker as InvokerT, Opcode, RuntimeBackend, RuntimeEnvironment,
	RuntimeState, TransactionContext, TransactionalBackend, TransactionalMergeStrategy, Transfer,
};
use alloc::rc::Rc;
use core::cmp::min;
use core::convert::Infallible;
use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

pub enum CallCreateTrapPrepareData<G> {
	Call {
		gasometer: G,
		code: Vec<u8>,
		is_static: bool,
		transaction_context: Rc<TransactionContext>,
		trap: CallTrapData,
	},
	Create {
		gasometer: G,
		code: Vec<u8>,
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

	pub fn transact_call<G, H>(
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
		G: GasometerT<RuntimeState, H> + TransactGasometer<'config, RuntimeState>,
		H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	{
		routines::transact_and_work(
			self,
			caller,
			gas_limit,
			gas_price,
			handler,
			|handler: &mut H| -> (ExitResult, U256) {
				let context = Context {
					caller,
					address,
					apparent_value: value,
				};
				let transfer = Transfer {
					source: caller,
					target: address,
					value,
				};
				let transaction_context = TransactionContext {
					origin: caller,
					gas_price,
				};

				let code = handler.code(address);

				let gasometer = try_or_oog!(G::new_transact_call(
					gas_limit,
					&code,
					&data,
					&access_list,
					self.config
				));

				let machine = try_or_oog!(routines::make_enter_call_machine(
					self,
					code,
					data,
					false, // is_static
					Some(transfer),
					context,
					Rc::new(transaction_context),
					gasometer,
					handler
				));

				if self.config.increase_state_access_gas {
					if self.config.warm_coinbase_address {
						let coinbase = handler.block_coinbase();
						try_or_oog!(handler.mark_hot(coinbase, None));
					}
					try_or_oog!(handler.mark_hot(caller, None));
					try_or_oog!(handler.mark_hot(address, None));
				}

				try_or_oog!(handler.inc_nonce(caller));

				let (machine, result) =
					crate::execute(machine, handler, 0, DEFAULT_HEAP_DEPTH, self, etable);

				let refunded_gas = U256::from(machine.gasometer.gas());
				(result, refunded_gas)
			},
		)
	}

	pub fn transact_create<G, H>(
		&self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		gas_limit: U256,
		gas_price: U256,
		access_list: Vec<(H160, Vec<H256>)>,
		handler: &mut H,
		etable: &Etable<H>,
	) -> Result<H160, ExitError>
	where
		G: GasometerT<RuntimeState, H> + TransactGasometer<'config, RuntimeState>,
		H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	{
		routines::transact_and_work(
			self,
			caller,
			gas_limit,
			gas_price,
			handler,
			|handler| -> (Result<H160, ExitError>, U256) {
				let scheme = CreateScheme::Legacy { caller };
				let address = scheme.address(handler);

				let context = Context {
					caller,
					address,
					apparent_value: value,
				};
				let transaction_context = TransactionContext {
					origin: caller,
					gas_price,
				};
				let transfer = Transfer {
					source: caller,
					target: address,
					value,
				};

				let gasometer = try_or_oog!(G::new_transact_create(
					gas_limit,
					&init_code,
					&access_list,
					self.config
				));

				let machine = try_or_oog!(routines::make_enter_create_machine(
					self,
					caller,
					init_code,
					false, // is_static
					transfer,
					context,
					Rc::new(transaction_context),
					gasometer,
					handler,
				));

				let (mut machine, result) =
					crate::execute(machine, handler, 0, DEFAULT_HEAP_DEPTH, self, etable);
				let retbuf = machine.machine.into_retbuf();
				let address = try_or_oog!(routines::deploy_create_code(
					self,
					result.map(|_| address),
					&retbuf,
					&mut machine.gasometer,
					handler
				));

				let refunded_gas = U256::from(machine.gasometer.gas());
				(Ok(address), refunded_gas)
			},
		)
	}

	pub fn transact_create2<G, H>(
		&self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		salt: H256,
		gas_limit: U256,
		gas_price: U256,
		access_list: Vec<(H160, Vec<H256>)>,
		handler: &mut H,
		etable: &Etable<H>,
	) -> Result<H160, ExitError>
	where
		G: GasometerT<RuntimeState, H> + TransactGasometer<'config, RuntimeState>,
		H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	{
		routines::transact_and_work(
			self,
			caller,
			gas_limit,
			gas_price,
			handler,
			|handler| -> (Result<H160, ExitError>, U256) {
				let scheme = CreateScheme::Create2 {
					caller,
					code_hash: H256::from_slice(Keccak256::digest(&init_code).as_slice()),
					salt,
				};
				let address = scheme.address(handler);

				let context = Context {
					caller,
					address,
					apparent_value: value,
				};
				let transaction_context = TransactionContext {
					origin: caller,
					gas_price,
				};
				let transfer = Transfer {
					source: caller,
					target: address,
					value,
				};

				let gasometer = try_or_oog!(G::new_transact_create(
					gas_limit,
					&init_code,
					&access_list,
					self.config
				));

				let machine = try_or_oog!(routines::make_enter_create_machine(
					self,
					caller,
					init_code,
					false, // is_static
					transfer,
					context,
					Rc::new(transaction_context),
					gasometer,
					handler,
				));

				let (mut machine, result) =
					crate::execute(machine, handler, 0, DEFAULT_HEAP_DEPTH, self, etable);
				let retbuf = machine.machine.into_retbuf();
				let address = try_or_oog!(routines::deploy_create_code(
					self,
					result.map(|_| address),
					&retbuf,
					&mut machine.gasometer,
					handler
				));

				let refunded_gas = U256::from(machine.gasometer.gas());
				(Ok(address), refunded_gas)
			},
		)
	}
}

impl<'config, G, H> InvokerT<RuntimeState, G, H, Opcode> for Invoker<'config>
where
	G: GasometerT<RuntimeState, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
{
	type Interrupt = Infallible;
	type CallCreateTrapPrepareData = CallCreateTrapPrepareData<G>;
	type CallCreateTrapEnterData = CallCreateTrapEnterData;

	fn prepare_trap(
		&self,
		opcode: Opcode,
		machine: &mut GasedMachine<G>,
		handler: &mut H,
		depth: usize,
	) -> Capture<Result<Self::CallCreateTrapPrepareData, ExitError>, Infallible> {
		fn l64(gas: U256) -> U256 {
			gas - gas / U256::from(64)
		}

		if depth >= self.config.call_stack_limit {
			return Capture::Exit(Err(ExitException::CallTooDeep.into()));
		}

		let trap_data = match CallCreateTrapData::new_from(opcode, &mut machine.machine) {
			Ok(trap_data) => trap_data,
			Err(err) => return Capture::Exit(Err(err)),
		};

		let after_gas = if self.config.call_l64_after_gas {
			l64(machine.gasometer.gas())
		} else {
			machine.gasometer.gas()
		};
		let target_gas = trap_data.target_gas().unwrap_or(after_gas);
		let mut gas_limit = min(after_gas, target_gas);

		match &trap_data {
			CallCreateTrapData::Call(call) if call.has_value() => {
				gas_limit = gas_limit.saturating_add(U256::from(self.config.call_stipend));
			}
			_ => (),
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

		let code = trap_data.code(handler);
		let submeter = match machine.gasometer.submeter(gas_limit, &code) {
			Ok(submeter) => submeter,
			Err(err) => return Capture::Exit(Err(err)),
		};

		Capture::Exit(Ok(match trap_data {
			CallCreateTrapData::Call(call_trap_data) => CallCreateTrapPrepareData::Call {
				gasometer: submeter,
				code,
				is_static,
				transaction_context,
				trap: call_trap_data,
			},
			CallCreateTrapData::Create(create_trap_data) => CallCreateTrapPrepareData::Create {
				gasometer: submeter,
				code,
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
	) -> Result<(Self::CallCreateTrapEnterData, GasedMachine<G>), ExitError> {
		match trap_data {
			CallCreateTrapPrepareData::Create {
				gasometer,
				code,
				is_static,
				transaction_context,
				trap,
			} => routines::enter_create_trap_stack(
				self,
				code,
				trap,
				is_static,
				transaction_context,
				gasometer,
				handler,
			),
			CallCreateTrapPrepareData::Call {
				gasometer,
				code,
				is_static,
				transaction_context,
				trap,
			} => routines::enter_call_trap_stack(
				self,
				code,
				trap,
				is_static,
				transaction_context,
				gasometer,
				handler,
			),
		}
	}

	fn exit_trap_stack(
		&self,
		result: ExitResult,
		mut child: GasedMachine<G>,
		trap_data: Self::CallCreateTrapEnterData,
		parent: &mut GasedMachine<G>,
		handler: &mut H,
	) -> Result<(), ExitError> {
		match trap_data {
			CallCreateTrapEnterData::Create { address, trap } => {
				let retbuf = child.machine.into_retbuf();
				let result = routines::deploy_create_code(
					self,
					result.map(|_| address),
					&retbuf,
					&mut child.gasometer,
					handler,
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
						GasometerT::<RuntimeState, H>::merge(
							&mut parent.gasometer,
							child.gasometer,
							GasometerMergeStrategy::Discard,
						);
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
						GasometerT::<RuntimeState, H>::merge(
							&mut parent.gasometer,
							child.gasometer,
							GasometerMergeStrategy::Discard,
						);
					}
				};

				trap.feedback(result, retbuf, &mut parent.machine)?;

				Ok(())
			}
		}
	}
}
