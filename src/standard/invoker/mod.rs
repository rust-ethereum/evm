mod resolver;
mod routines;

pub use resolver::{EtableResolver, PrecompileSet, Resolver};

use super::{Config, MergeableRuntimeState, TransactGasometer};
use crate::call_create::{CallCreateTrapData, CallTrapData, CreateScheme, CreateTrapData};
use crate::{
	Capture, ColoredMachine, Context, ExitError, ExitException, ExitResult, ExitSucceed,
	Gasometer as GasometerT, Invoker as InvokerT, InvokerControl, MergeStrategy, Opcode,
	RuntimeBackend, RuntimeEnvironment, RuntimeState, TransactionContext, TransactionalBackend,
	Transfer,
};
use alloc::rc::Rc;
use core::cmp::min;
use core::convert::Infallible;
use core::marker::PhantomData;
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

pub enum SubstackInvoke {
	Call { trap: CallTrapData },
	Create { trap: CreateTrapData, address: H160 },
}

pub struct TransactInvoke {
	pub create_address: Option<H160>,
	pub gas_fee: U256,
	pub gas_price: U256,
	pub caller: H160,
}

#[derive(Clone, Debug)]
pub enum TransactArgs {
	Call {
		caller: H160,
		address: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: U256,
		gas_price: U256,
		access_list: Vec<(H160, Vec<H256>)>,
	},
	Create {
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		salt: Option<H256>, // Some for CREATE2
		gas_limit: U256,
		gas_price: U256,
		access_list: Vec<(H160, Vec<H256>)>,
	},
}

impl TransactArgs {
	pub fn gas_limit(&self) -> U256 {
		match self {
			Self::Call { gas_limit, .. } => *gas_limit,
			Self::Create { gas_limit, .. } => *gas_limit,
		}
	}

	pub fn gas_price(&self) -> U256 {
		match self {
			Self::Call { gas_price, .. } => *gas_price,
			Self::Create { gas_price, .. } => *gas_price,
		}
	}

	pub fn access_list(&self) -> &Vec<(H160, Vec<H256>)> {
		match self {
			Self::Call { access_list, .. } => access_list,
			Self::Create { access_list, .. } => access_list,
		}
	}

	pub fn caller(&self) -> H160 {
		match self {
			Self::Call { caller, .. } => *caller,
			Self::Create { caller, .. } => *caller,
		}
	}

	pub fn value(&self) -> U256 {
		match self {
			Self::Call { value, .. } => *value,
			Self::Create { value, .. } => *value,
		}
	}
}

pub struct Invoker<'config, 'resolver, S, G, H, R, Tr> {
	config: &'config Config,
	resolver: &'resolver R,
	_marker: PhantomData<(S, G, H, Tr)>,
}

impl<'config, 'resolver, S, G, H, R, Tr> Invoker<'config, 'resolver, S, G, H, R, Tr> {
	pub fn new(config: &'config Config, resolver: &'resolver R) -> Self {
		Self {
			config,
			resolver,
			_marker: PhantomData,
		}
	}
}

impl<'config, 'resolver, S, G, H, R, Tr> InvokerT<H, Tr>
	for Invoker<'config, 'resolver, S, G, H, R, Tr>
where
	S: MergeableRuntimeState<ColoredMachine<S, G, R::Color>>,
	G: GasometerT<S, H> + TransactGasometer<'config, S>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<S, G, H, Tr>,
	Tr: IntoCallCreateTrap,
{
	type Machine = ColoredMachine<S, G, R::Color>;
	type Interrupt = Tr::Interrupt;
	type TransactArgs = TransactArgs;
	type TransactInvoke = TransactInvoke;
	type TransactValue = (ExitSucceed, Option<H160>);
	type SubstackInvoke = SubstackInvoke;

	fn new_transact(
		&self,
		args: TransactArgs,
		handler: &mut H,
	) -> Result<
		(
			TransactInvoke,
			InvokerControl<ColoredMachine<S, G, R::Color>, (ExitResult, (S, G, Vec<u8>))>,
		),
		ExitError,
	> {
		let caller = args.caller();
		let gas_price = args.gas_price();

		let gas_fee = args.gas_limit().saturating_mul(gas_price);
		handler.withdrawal(caller, gas_fee)?;

		handler.inc_nonce(caller)?;

		let address = match &args {
			TransactArgs::Call { address, .. } => *address,
			TransactArgs::Create {
				caller,
				salt,
				init_code,
				..
			} => match salt {
				Some(salt) => {
					let scheme = CreateScheme::Create2 {
						caller: *caller,
						code_hash: H256::from_slice(Keccak256::digest(&init_code).as_slice()),
						salt: *salt,
					};
					scheme.address(handler)
				}
				None => {
					let scheme = CreateScheme::Legacy { caller: *caller };
					scheme.address(handler)
				}
			},
		};
		let value = args.value();

		let invoke = TransactInvoke {
			gas_fee,
			gas_price: args.gas_price(),
			caller: args.caller(),
			create_address: match &args {
				TransactArgs::Call { .. } => None,
				TransactArgs::Create { .. } => Some(address),
			},
		};

		handler.push_substate();

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

		let work = || -> Result<(TransactInvoke, _), ExitError> {
			match args {
				TransactArgs::Call {
					caller,
					address,
					data,
					gas_limit,
					access_list,
					..
				} => {
					for (address, keys) in &access_list {
						handler.mark_hot(*address, None);
						for key in keys {
							handler.mark_hot(*address, Some(*key));
						}
					}

					let gasometer =
						G::new_transact_call(gas_limit, &data, &access_list, self.config)?;

					let machine = routines::make_enter_call_machine(
						self.config,
						self.resolver,
						address,
						data,
						false, // is_static
						Some(transfer),
						S::new_transact_call(RuntimeState {
							context,
							transaction_context: Rc::new(transaction_context),
							retbuf: Vec::new(),
							gas: U256::from(gas_limit),
						}),
						gasometer,
						handler,
					)?;

					if self.config.increase_state_access_gas {
						if self.config.warm_coinbase_address {
							let coinbase = handler.block_coinbase();
							handler.mark_hot(coinbase, None);
						}
						handler.mark_hot(caller, None);
						handler.mark_hot(address, None);
					}

					Ok((invoke, machine))
				}
				TransactArgs::Create {
					caller,
					init_code,
					gas_limit,
					access_list,
					..
				} => {
					let gasometer =
						G::new_transact_create(gas_limit, &init_code, &access_list, self.config)?;

					let machine = routines::make_enter_create_machine(
						self.config,
						self.resolver,
						caller,
						init_code,
						false, // is_static
						transfer,
						S::new_transact_create(RuntimeState {
							context,
							transaction_context: Rc::new(transaction_context),
							retbuf: Vec::new(),
							gas: U256::from(gas_limit),
						}),
						gasometer,
						handler,
					)?;

					Ok((invoke, machine))
				}
			}
		};

		match work() {
			Ok(ret) => Ok(ret),
			Err(err) => {
				handler.pop_substate(MergeStrategy::Discard);
				Err(err)
			}
		}
	}

	fn finalize_transact(
		&self,
		invoke: &TransactInvoke,
		result: ExitResult,
		(_substate, mut submeter, retval): (S, G, Vec<u8>),
		handler: &mut H,
	) -> Result<Self::TransactValue, ExitError> {
		let left_gas = submeter.effective_gas();

		let work = || -> Result<Self::TransactValue, ExitError> {
			if result.is_ok() {
				if let Some(address) = invoke.create_address {
					let retbuf = retval;

					routines::deploy_create_code(
						self.config,
						address,
						&retbuf,
						&mut submeter,
						handler,
					)?;
				}
			}

			result.map(|s| (s, invoke.create_address))
		};

		let result = work();

		let refunded_gas = match result {
			Ok(_) | Err(ExitError::Reverted) => left_gas,
			Err(_) => U256::zero(),
		};
		let refunded_fee = refunded_gas.saturating_mul(invoke.gas_price);
		let coinbase_reward = invoke.gas_fee.saturating_sub(refunded_fee);

		match &result {
			Ok(_) => {
				handler.pop_substate(MergeStrategy::Commit);
			}
			Err(_) => {
				handler.pop_substate(MergeStrategy::Discard);
			}
		}

		handler.deposit(invoke.caller, refunded_fee);
		handler.deposit(handler.block_coinbase(), coinbase_reward);

		result
	}

	fn enter_substack(
		&self,
		trap: Tr,
		machine: &mut ColoredMachine<S, G, R::Color>,
		handler: &mut H,
		depth: usize,
	) -> Capture<
		Result<
			(
				SubstackInvoke,
				InvokerControl<ColoredMachine<S, G, R::Color>, (ExitResult, (S, G, Vec<u8>))>,
			),
			ExitError,
		>,
		Tr::Interrupt,
	> {
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
		let gas_limit = min(after_gas, target_gas);

		let call_has_value = match &trap_data {
			CallCreateTrapData::Call(call) if call.has_value() => true,
			_ => false,
		};

		let is_static = if machine.is_static {
			true
		} else {
			match &trap_data {
				CallCreateTrapData::Call(CallTrapData { is_static, .. }) => *is_static,
				_ => false,
			}
		};

		let transaction_context = machine.machine.state.as_ref().transaction_context.clone();

		match trap_data {
			CallCreateTrapData::Call(call_trap_data) => {
				let submeter = match machine.gasometer.submeter(gas_limit, call_has_value) {
					Ok(submeter) => submeter,
					Err(err) => return Capture::Exit(Err(err)),
				};

				let substate = machine.machine.state.substate(
					RuntimeState {
						context: call_trap_data.context.clone(),
						transaction_context,
						retbuf: Vec::new(),
						gas: U256::from(gas_limit),
					},
					&machine,
				);

				let target = call_trap_data.target;

				Capture::Exit(routines::enter_call_substack(
					self.config,
					self.resolver,
					call_trap_data,
					target,
					is_static,
					substate,
					submeter,
					handler,
				))
			}
			CallCreateTrapData::Create(create_trap_data) => {
				let code = create_trap_data.code.clone();
				let submeter = match machine.gasometer.submeter(gas_limit, call_has_value) {
					Ok(submeter) => submeter,
					Err(err) => return Capture::Exit(Err(err)),
				};

				let caller = create_trap_data.scheme.caller();
				let address = create_trap_data.scheme.address(handler);
				let substate = machine.machine.state.substate(
					RuntimeState {
						context: Context {
							address,
							caller,
							apparent_value: create_trap_data.value,
						},
						transaction_context,
						retbuf: Vec::new(),
						gas: U256::from(gas_limit),
					},
					&machine,
				);

				Capture::Exit(routines::enter_create_substack(
					self.config,
					self.resolver,
					code,
					create_trap_data,
					is_static,
					substate,
					submeter,
					handler,
				))
			}
		}
	}

	fn exit_substack(
		&self,
		result: ExitResult,
		(substate, submeter, retval): (S, G, Vec<u8>),
		trap_data: SubstackInvoke,
		parent: &mut ColoredMachine<S, G, R::Color>,
		handler: &mut H,
	) -> Result<(), ExitError> {
		let strategy = match &result {
			Ok(_) => MergeStrategy::Commit,
			Err(ExitError::Reverted) => MergeStrategy::Revert,
			Err(_) => MergeStrategy::Discard,
		};

		match trap_data {
			SubstackInvoke::Create { address, trap } => {
				let retbuf = retval;
				parent.machine.state.merge(substate, strategy);

				let mut child_gasometer = submeter;
				let result = result.and_then(|_| {
					routines::deploy_create_code(
						self.config,
						address,
						&retbuf,
						&mut child_gasometer,
						handler,
					)?;

					Ok(address)
				});

				handler.pop_substate(strategy);
				TransactGasometer::<S>::merge(&mut parent.gasometer, child_gasometer, strategy);

				trap.feedback(result, retbuf, &mut parent.machine)?;

				Ok(())
			}
			SubstackInvoke::Call { trap } => {
				parent.machine.state.merge(substate, strategy);

				let retbuf = retval;

				handler.pop_substate(strategy);
				TransactGasometer::<S>::merge(&mut parent.gasometer, submeter, strategy);

				trap.feedback(result, retbuf, &mut parent.machine)?;

				Ok(())
			}
		}
	}
}
