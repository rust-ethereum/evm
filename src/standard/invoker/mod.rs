mod routines;

use self::routines::try_or_oog;
use super::{Config, MergeableRuntimeState, TransactGasometer};
use crate::call_create::{CallCreateTrapData, CallTrapData, CreateScheme, CreateTrapData};
use crate::{
	Capture, Context, Control, Etable, ExitError, ExitException, ExitResult, GasedMachine,
	Gasometer as GasometerT, Invoker as InvokerT, Machine, MergeStrategy, Opcode, RuntimeBackend,
	RuntimeEnvironment, RuntimeState, TransactionContext, TransactionalBackend, Transfer,
};
use alloc::rc::Rc;
use core::cmp::min;
use core::convert::Infallible;
use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

pub trait IntoCallCreateTrap {
	type Interrupt;

	fn into_call_create_trap(self) -> Result<Opcode, Self::Interrupt>;
}

impl IntoCallCreateTrap for Opcode {
	type Interrupt = Infallible;

	fn into_call_create_trap(self) -> Result<Opcode, Infallible> {
		Ok(self)
	}
}

pub enum CallCreateTrapPrepareData<S, G> {
	Call {
		gasometer: G,
		code: Vec<u8>,
		is_static: bool,
		trap: CallTrapData,
		state: S,
	},
	Create {
		gasometer: G,
		code: Vec<u8>,
		is_static: bool,
		trap: CreateTrapData,
		state: S,
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

	pub fn transact_call<S, G, H, F>(
		&self,
		caller: H160,
		address: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: U256,
		gas_price: U256,
		access_list: Vec<(H160, Vec<H256>)>,
		handler: &mut H,
		etable: &Etable<S, H, Opcode, F>,
	) -> ExitResult
	where
		S: MergeableRuntimeState,
		G: GasometerT<S, H> + TransactGasometer<'config, S>,
		H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Opcode>,
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
					S::new_transact_call(RuntimeState {
						context,
						transaction_context: Rc::new(transaction_context),
						retbuf: Vec::new(),
						gas: U256::zero(),
					}),
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

	pub fn transact_create<S, G, H, F>(
		&self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		gas_limit: U256,
		gas_price: U256,
		access_list: Vec<(H160, Vec<H256>)>,
		handler: &mut H,
		etable: &Etable<S, H, Opcode, F>,
	) -> Result<H160, ExitError>
	where
		S: MergeableRuntimeState,
		G: GasometerT<S, H> + TransactGasometer<'config, S>,
		H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Opcode>,
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
					S::new_transact_create(RuntimeState {
						context,
						transaction_context: Rc::new(transaction_context),
						retbuf: Vec::new(),
						gas: U256::zero(),
					}),
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

	pub fn transact_create2<S, G, H, F>(
		&self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		salt: H256,
		gas_limit: U256,
		gas_price: U256,
		access_list: Vec<(H160, Vec<H256>)>,
		handler: &mut H,
		etable: &Etable<S, H, Opcode, F>,
	) -> Result<H160, ExitError>
	where
		S: MergeableRuntimeState,
		G: GasometerT<S, H> + TransactGasometer<'config, S>,
		H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Opcode>,
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
					S::new_transact_create(RuntimeState {
						context,
						transaction_context: Rc::new(transaction_context),
						retbuf: Vec::new(),
						gas: U256::zero(),
					}),
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

impl<'config, S, G, H, Tr> InvokerT<S, G, H, Tr> for Invoker<'config>
where
	S: MergeableRuntimeState,
	G: GasometerT<S, H>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	Tr: IntoCallCreateTrap,
{
	type Interrupt = Tr::Interrupt;
	type CallCreateTrapPrepareData = CallCreateTrapPrepareData<S, G>;
	type CallCreateTrapEnterData = CallCreateTrapEnterData;

	fn prepare_trap(
		&self,
		trap: Tr,
		machine: &mut GasedMachine<S, G>,
		handler: &mut H,
		depth: usize,
	) -> Capture<Result<Self::CallCreateTrapPrepareData, ExitError>, Tr::Interrupt> {
		fn l64(gas: U256) -> U256 {
			gas - gas / U256::from(64)
		}

		let opcode = match trap.into_call_create_trap() {
			Ok(opcode) => opcode,
			Err(interrupt) => return Capture::Trap(interrupt),
		};

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
			CallCreateTrapData::Call(call_trap_data) => {
				let substate = machine.machine.state.substate(RuntimeState {
					context: call_trap_data.context.clone(),
					transaction_context,
					retbuf: Vec::new(),
					gas: U256::zero(),
				});
				CallCreateTrapPrepareData::Call {
					gasometer: submeter,
					code,
					is_static,
					trap: call_trap_data,
					state: substate,
				}
			}
			CallCreateTrapData::Create(create_trap_data) => {
				let caller = create_trap_data.scheme.caller();
				let address = create_trap_data.scheme.address(handler);
				let substate = machine.machine.state.substate(RuntimeState {
					context: Context {
						address,
						caller,
						apparent_value: create_trap_data.value,
					},
					transaction_context,
					retbuf: Vec::new(),
					gas: U256::zero(),
				});
				CallCreateTrapPrepareData::Create {
					gasometer: submeter,
					code,
					is_static,
					trap: create_trap_data,
					state: substate,
				}
			}
		}))
	}

	fn enter_trap_stack(
		&self,
		trap_data: Self::CallCreateTrapPrepareData,
		handler: &mut H,
	) -> Result<(Self::CallCreateTrapEnterData, GasedMachine<S, G>), ExitError> {
		match trap_data {
			CallCreateTrapPrepareData::Create {
				gasometer,
				code,
				is_static,
				trap,
				state,
			} => routines::enter_create_trap_stack(
				self, code, trap, is_static, state, gasometer, handler,
			),
			CallCreateTrapPrepareData::Call {
				gasometer,
				code,
				is_static,
				trap,
				state,
			} => routines::enter_call_trap_stack(
				self, code, trap, is_static, state, gasometer, handler,
			),
		}
	}

	fn exit_trap_stack(
		&self,
		result: ExitResult,
		mut child: GasedMachine<S, G>,
		trap_data: Self::CallCreateTrapEnterData,
		parent: &mut GasedMachine<S, G>,
		handler: &mut H,
	) -> Result<(), ExitError> {
		let strategy = match &result {
			Ok(_) => MergeStrategy::Commit,
			Err(ExitError::Reverted) => MergeStrategy::Revert,
			Err(_) => MergeStrategy::Discard,
		};

		match trap_data {
			CallCreateTrapEnterData::Create { address, trap } => {
				parent.machine.state.merge(child.machine.state, strategy);

				let retbuf = child.machine.memory.into_data();
				let result = routines::deploy_create_code(
					self,
					result.map(|_| address),
					&retbuf,
					&mut child.gasometer,
					handler,
				);

				handler.pop_substate(strategy);
				GasometerT::<S, H>::merge(&mut parent.gasometer, child.gasometer, strategy);

				trap.feedback(result, retbuf, &mut parent.machine)?;

				Ok(())
			}
			CallCreateTrapEnterData::Call { trap } => {
				parent.machine.state.merge(child.machine.state, strategy);

				let retbuf = child.machine.memory.into_data();

				handler.pop_substate(strategy);
				GasometerT::<S, H>::merge(&mut parent.gasometer, child.gasometer, strategy);

				trap.feedback(result, retbuf, &mut parent.machine)?;

				Ok(())
			}
		}
	}
}
