mod resolver;
pub mod routines;
mod state;

use alloc::{rc::Rc, vec::Vec};
use core::{cmp::min, convert::Infallible};

use evm_interpreter::{
	error::{
		CallCreateTrap, CallCreateTrapData, CallScheme, CallTrapData, Capture, CreateScheme,
		CreateTrapData, ExitError, ExitException, ExitFatal, ExitSucceed, TrapConsume,
	},
	opcode::Opcode,
	runtime::{
		Context, GasState, RuntimeBackend, RuntimeEnvironment, RuntimeState, SetCodeOrigin,
		TouchKind, TransactionContext, Transfer,
	},
	Interpreter,
};
use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

pub use self::{
	resolver::{EtableResolver, PrecompileSet, Resolver},
	state::InvokerState,
};
use crate::{
	backend::TransactionalBackend,
	invoker::{Invoker as InvokerT, InvokerControl, InvokerExit},
	standard::Config,
	MergeStrategy,
};

/// A trap that can be turned into either a call/create trap (where we push new
/// call stack), or an interrupt (an external signal).
pub trait IntoCallCreateTrap {
	/// An external signal.
	type Interrupt;

	/// Turn the current trap into either a call/create trap or an interrupt.
	fn into_call_create_trap(self) -> Result<Opcode, Self::Interrupt>;
}

impl IntoCallCreateTrap for Opcode {
	type Interrupt = Infallible;

	fn into_call_create_trap(self) -> Result<Opcode, Infallible> {
		Ok(self)
	}
}

/// The invoke used in a substack.
pub enum SubstackInvoke {
	Call { trap: CallTrapData },
	Create { trap: CreateTrapData, address: H160 },
}

/// Return value of a transaction.
#[derive(Clone, Debug)]
pub enum TransactValue {
	Call {
		/// The exit result. If we return a value, then it will be an
		/// `ExitSucceed`.
		succeed: ExitSucceed,
		/// The return value, if any.
		retval: Vec<u8>,
	},
	Create {
		/// The exit result. If we return a value, then it will be an
		/// `ExitSucceed`.
		succeed: ExitSucceed,
		/// The contract address created.
		address: H160,
	},
}

/// The invoke used in a top-layer transaction stack.
pub struct TransactInvoke {
	pub create_address: Option<H160>,
	pub gas_limit: U256,
	pub gas_price: U256,
	pub caller: H160,
}

/// Transaction arguments.
#[derive(Clone, Debug)]
pub enum TransactArgs {
	/// A call transaction.
	Call {
		/// Transaction sender.
		caller: H160,
		/// Transaction target.
		address: H160,
		/// Transaction value.
		value: U256,
		/// Transaction call data.
		data: Vec<u8>,
		/// Transaction gas limit.
		gas_limit: U256,
		/// Transaction gas price.
		gas_price: U256,
		/// Access list information, in the format of (address, storage keys).
		access_list: Vec<(H160, Vec<H256>)>,
	},
	/// A create transaction.
	Create {
		/// Transaction sender.
		caller: H160,
		/// Transaction value.
		value: U256,
		/// Init code.
		init_code: Vec<u8>,
		/// Salt of `CREATE2`. `None` for a normal create transaction.
		salt: Option<H256>,
		/// Transaction gas limit.
		gas_limit: U256,
		/// Transaction gas price.
		gas_price: U256,
		/// Access list information, in the format of (address, storage keys).
		access_list: Vec<(H160, Vec<H256>)>,
	},
}

impl TransactArgs {
	/// Transaction gas limit.
	pub fn gas_limit(&self) -> U256 {
		match self {
			Self::Call { gas_limit, .. } => *gas_limit,
			Self::Create { gas_limit, .. } => *gas_limit,
		}
	}

	/// Transaction gas price.
	pub fn gas_price(&self) -> U256 {
		match self {
			Self::Call { gas_price, .. } => *gas_price,
			Self::Create { gas_price, .. } => *gas_price,
		}
	}

	/// Access list information.
	pub fn access_list(&self) -> &Vec<(H160, Vec<H256>)> {
		match self {
			Self::Call { access_list, .. } => access_list,
			Self::Create { access_list, .. } => access_list,
		}
	}

	/// Transaction sender.
	pub fn caller(&self) -> H160 {
		match self {
			Self::Call { caller, .. } => *caller,
			Self::Create { caller, .. } => *caller,
		}
	}

	/// Transaction value.
	pub fn value(&self) -> U256 {
		match self {
			Self::Call { value, .. } => *value,
			Self::Create { value, .. } => *value,
		}
	}
}

/// Standard invoker.
///
/// The generic parameters are as follows:
/// * `S`: The runtime state, usually [RuntimeState] but can be customized.
/// * `H`: Backend type.
/// * `R`: Code resolver type, also handle precompiles. Usually
///   [EtableResolver] but can be customized.
/// * `Tr`: Trap type, usually [crate::Opcode] but can be customized.
pub struct Invoker<'config, 'resolver, R> {
	config: &'config Config,
	resolver: &'resolver R,
}

impl<'config, 'resolver, R> Invoker<'config, 'resolver, R> {
	/// Create a new standard invoker with the given config and resolver.
	pub fn new(config: &'config Config, resolver: &'resolver R) -> Self {
		Self { config, resolver }
	}
}

impl<'config, 'resolver, H, R, Tr> InvokerT<H, Tr> for Invoker<'config, 'resolver, R>
where
	R::State: InvokerState<'config> + AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
	Tr: TrapConsume<CallCreateTrap>,
{
	type State = R::State;
	type Interpreter = R::Interpreter;
	type Interrupt = Tr::Rest;
	type TransactArgs = TransactArgs;
	type TransactInvoke = TransactInvoke;
	type TransactValue = TransactValue;
	type SubstackInvoke = SubstackInvoke;

	fn new_transact(
		&self,
		args: Self::TransactArgs,
		handler: &mut H,
	) -> Result<(Self::TransactInvoke, InvokerControl<Self::Interpreter>), ExitError> {
		let caller = args.caller();
		let gas_price = args.gas_price();
		let gas_fee = args.gas_limit().saturating_mul(gas_price);
		let coinbase = handler.block_coinbase();

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
						code_hash: H256::from_slice(Keccak256::digest(init_code).as_slice()),
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
			gas_limit: args.gas_limit(),
			gas_price: args.gas_price(),
			caller: args.caller(),
			create_address: match &args {
				TransactArgs::Call { .. } => None,
				TransactArgs::Create { .. } => Some(address),
			},
		};

		match handler.inc_nonce(caller) {
			Ok(()) => (),
			Err(err) => {
				handler.push_substate();
				return Ok((
					invoke,
					InvokerControl::DirectExit(InvokerExit {
						result: Err(err),
						substate: None,
						retval: Vec::new(),
					}),
				));
			}
		}

		match handler.withdrawal(caller, gas_fee) {
			Ok(()) => (),
			Err(err) => {
				handler.push_substate();
				return Ok((
					invoke,
					InvokerControl::DirectExit(InvokerExit {
						result: Err(err),
						substate: None,
						retval: Vec::new(),
					}),
				));
			}
		}

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
		let runtime_state = RuntimeState {
			context,
			transaction_context: Rc::new(transaction_context),
			retbuf: Vec::new(),
		};

		let machine = match args {
			TransactArgs::Call {
				caller,
				address,
				data,
				gas_limit,
				access_list,
				..
			} => {
				for (address, keys) in &access_list {
					handler.mark_hot(*address, TouchKind::Access);
					for key in keys {
						handler.mark_storage_hot(*address, *key);
					}
				}

				let state = <R::State>::new_transact_call(
					runtime_state,
					gas_limit,
					&data,
					&access_list,
					self.config,
				)?;

				handler.mark_hot(coinbase, TouchKind::Coinbase);
				handler.mark_hot(caller, TouchKind::StateChange);
				handler.mark_hot(address, TouchKind::StateChange);

				routines::make_enter_call_machine(
					self.config,
					self.resolver,
					CallScheme::Call,
					address,
					data,
					Some(transfer),
					state,
					handler,
				)?
			}
			TransactArgs::Create {
				caller,
				init_code,
				gas_limit,
				access_list,
				..
			} => {
				let state = <R::State>::new_transact_create(
					runtime_state,
					gas_limit,
					&init_code,
					&access_list,
					self.config,
				)?;

				handler.mark_hot(coinbase, TouchKind::Coinbase);
				handler.mark_hot(caller, TouchKind::StateChange);
				handler.mark_hot(address, TouchKind::StateChange);

				routines::make_enter_create_machine(
					self.config,
					self.resolver,
					caller,
					init_code,
					transfer,
					state,
					handler,
				)?
			}
		};

		Ok((invoke, machine))
	}

	fn finalize_transact(
		&self,
		invoke: &Self::TransactInvoke,
		mut exit: InvokerExit<Self::State>,
		handler: &mut H,
	) -> Result<Self::TransactValue, ExitError> {
		let substate = exit.substate.as_mut();

		let work = || -> Result<_, ExitError> {
			match exit.result {
				Ok(result) => {
					if let Some(address) = invoke.create_address {
						let retbuf = exit.retval;

						if let Some(substate) = substate {
							routines::deploy_create_code(
								self.config,
								address,
								retbuf,
								substate,
								handler,
								SetCodeOrigin::Transaction,
							)?;

							Ok(TransactValue::Create {
								succeed: result,
								address,
							})
						} else {
							Err(ExitFatal::Unfinished.into())
						}
					} else {
						Ok(TransactValue::Call {
							succeed: result,
							retval: exit.retval,
						})
					}
				}
				Err(result) => Err(result),
			}
		};

		let result = work();

		let left_gas = exit
			.substate
			.as_ref()
			.map(|s| s.effective_gas())
			.unwrap_or_default();

		let refunded_gas = match result {
			Ok(_) | Err(ExitError::Reverted) => left_gas,
			Err(_) => U256::zero(),
		};

		match &result {
			Ok(_) => {
				handler.pop_substate(MergeStrategy::Commit)?;
			}
			Err(_) => {
				handler.pop_substate(MergeStrategy::Discard)?;
			}
		}

		let refunded_fee = refunded_gas.saturating_mul(invoke.gas_price);
		handler.deposit(invoke.caller, refunded_fee);
		// Reward coinbase address
		// EIP-1559 updated the fee system so that miners only get to keep the priority fee.
		// The base fee is always burned.
		let coinbase_gas_price = if self.config.eip_1559_enabled {
			invoke
				.gas_price
				.saturating_sub(handler.block_base_fee_per_gas())
		} else {
			invoke.gas_price
		};
		let coinbase_reward = invoke
			.gas_limit
			.saturating_mul(coinbase_gas_price)
			.saturating_sub(refunded_fee);
		handler.deposit(handler.block_coinbase(), coinbase_reward);

		result
	}

	fn enter_substack(
		&self,
		trap: Tr,
		machine: &mut Self::Interpreter,
		handler: &mut H,
		depth: usize,
	) -> Capture<
		Result<(Self::SubstackInvoke, InvokerControl<Self::Interpreter>), ExitError>,
		Self::Interrupt,
	> {
		fn l64(gas: U256) -> U256 {
			gas - gas / U256::from(64)
		}

		let opcode = match trap.consume() {
			Ok(opcode) => opcode,
			Err(interrupt) => return Capture::Trap(interrupt),
		};

		let trap_data = match CallCreateTrapData::new_from(opcode, machine.machine_mut()) {
			Ok(trap_data) => trap_data,
			Err(err) => return Capture::Exit(Err(err)),
		};

		let invoke = match trap_data {
			CallCreateTrapData::Call(call_trap_data) => SubstackInvoke::Call {
				trap: call_trap_data,
			},
			CallCreateTrapData::Create(create_trap_data) => {
				let address = create_trap_data.scheme.address(handler);
				SubstackInvoke::Create {
					address,
					trap: create_trap_data,
				}
			}
		};

		let after_gas = if self.config.call_l64_after_gas {
			l64(machine.machine().state.gas())
		} else {
			machine.machine().state.gas()
		};
		let target_gas = match &invoke {
			SubstackInvoke::Call { trap, .. } => trap.gas,
			SubstackInvoke::Create { .. } => after_gas,
		};
		let gas_limit = min(after_gas, target_gas);

		let call_has_value = matches!(&invoke, SubstackInvoke::Call { trap: call_trap_data } if call_trap_data.has_value());

		let is_static = if machine.machine().state.is_static() {
			true
		} else {
			match &invoke {
				SubstackInvoke::Call {
					trap: CallTrapData { is_static, .. },
				} => *is_static,
				_ => false,
			}
		};

		let transaction_context = machine.machine().state.as_ref().transaction_context.clone();

		match invoke {
			SubstackInvoke::Call {
				trap: call_trap_data,
			} => {
				let substate = match machine.machine_mut().state.substate(
					RuntimeState {
						context: call_trap_data.context.clone(),
						transaction_context,
						retbuf: Vec::new(),
					},
					gas_limit,
					is_static,
					call_has_value,
				) {
					Ok(submeter) => submeter,
					Err(err) => return Capture::Exit(Err(err)),
				};

				if depth > self.config.call_stack_limit {
					handler.push_substate();
					return Capture::Exit(Ok((
						SubstackInvoke::Call {
							trap: call_trap_data,
						},
						InvokerControl::DirectExit(InvokerExit {
							result: Err(ExitException::CallTooDeep.into()),
							substate: Some(substate),
							retval: Vec::new(),
						}),
					)));
				}

				if let Some(transfer) = &call_trap_data.transfer {
					handler.mark_hot(transfer.target, TouchKind::StateChange);

					if transfer.value != U256::zero()
						&& handler.balance(transfer.source) < transfer.value
					{
						handler.push_substate();
						return Capture::Exit(Ok((
							SubstackInvoke::Call {
								trap: call_trap_data,
							},
							InvokerControl::DirectExit(InvokerExit {
								result: Err(ExitException::OutOfFund.into()),
								substate: Some(substate),
								retval: Vec::new(),
							}),
						)));
					}
				}

				let target = call_trap_data.target;

				handler.mark_hot(call_trap_data.context.address, TouchKind::StateChange);

				Capture::Exit(routines::enter_call_substack(
					self.config,
					self.resolver,
					call_trap_data,
					target,
					substate,
					handler,
				))
			}
			SubstackInvoke::Create {
				trap: create_trap_data,
				address,
			} => {
				let caller = create_trap_data.scheme.caller();
				let code = create_trap_data.code.clone();
				let value = create_trap_data.value;

				let substate = match machine.machine_mut().state.substate(
					RuntimeState {
						context: Context {
							address,
							caller,
							apparent_value: create_trap_data.value,
						},
						transaction_context,
						retbuf: Vec::new(),
					},
					gas_limit,
					is_static,
					call_has_value,
				) {
					Ok(substate) => substate,
					Err(err) => return Capture::Exit(Err(err)),
				};

				if depth > self.config.call_stack_limit {
					handler.push_substate();
					return Capture::Exit(Ok((
						SubstackInvoke::Create {
							trap: create_trap_data,
							address,
						},
						InvokerControl::DirectExit(InvokerExit {
							result: Err(ExitException::CallTooDeep.into()),
							substate: Some(substate),
							retval: Vec::new(),
						}),
					)));
				}

				if value != U256::zero() && handler.balance(caller) < value {
					handler.push_substate();
					return Capture::Exit(Ok((
						SubstackInvoke::Create {
							trap: create_trap_data,
							address,
						},
						InvokerControl::DirectExit(InvokerExit {
							result: Err(ExitException::OutOfFund.into()),
							substate: Some(substate),
							retval: Vec::new(),
						}),
					)));
				}

				handler.mark_hot(address, TouchKind::StateChange);

				match handler.inc_nonce(caller) {
					Ok(()) => (),
					Err(err) => {
						handler.push_substate();
						return Capture::Exit(Ok((
							SubstackInvoke::Create {
								trap: create_trap_data,
								address,
							},
							InvokerControl::DirectExit(InvokerExit {
								result: Err(err),
								substate: Some(substate),
								retval: Vec::new(),
							}),
						)));
					}
				}

				Capture::Exit(routines::enter_create_substack(
					self.config,
					self.resolver,
					code,
					create_trap_data,
					address,
					substate,
					handler,
				))
			}
		}
	}

	fn exit_substack(
		&self,
		trap_data: Self::SubstackInvoke,
		exit: InvokerExit<Self::State>,
		parent: &mut Self::Interpreter,
		handler: &mut H,
	) -> Result<(), ExitError> {
		let strategy = match &exit.result {
			Ok(_) => MergeStrategy::Commit,
			Err(ExitError::Exception(ExitException::OutOfFund)) => MergeStrategy::Revert,
			Err(ExitError::Exception(ExitException::CallTooDeep)) => MergeStrategy::Revert,
			Err(ExitError::Reverted) => MergeStrategy::Revert,
			Err(_) => MergeStrategy::Discard,
		};

		match trap_data {
			SubstackInvoke::Create { address, trap } => {
				let mut retbuf = exit.retval;
				let caller = trap.scheme.caller();

				let result = if let Some(mut substate) = exit.substate {
					let result = match exit.result {
						Ok(_) => {
							match routines::deploy_create_code(
								self.config,
								address,
								retbuf,
								&mut substate,
								handler,
								SetCodeOrigin::Subcall(caller),
							) {
								Ok(()) => {
									retbuf = Vec::new();
									Ok(address)
								}
								Err(err) => {
									retbuf = Vec::new();
									Err(err)
								}
							}
						}
						Err(err) => Err(err),
					};

					parent.machine_mut().state.merge(substate, strategy);
					result
				} else {
					Err(ExitFatal::Unfinished.into())
				};

				handler.pop_substate(strategy)?;

				trap.feedback(result, retbuf, parent)
			}
			SubstackInvoke::Call { trap } => {
				let retbuf = exit.retval;

				if let Some(substate) = exit.substate {
					parent.machine_mut().state.merge(substate, strategy);
				}
				handler.pop_substate(strategy)?;

				trap.feedback(exit.result, retbuf, parent)
			}
		}
	}
}
