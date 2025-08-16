mod resolver;
pub mod routines;
mod state;

use alloc::{boxed::Box, rc::Rc, vec::Vec};
use core::{cmp::min, marker::PhantomData};
use evm_interpreter::{
	Capture, ExitError, ExitException, ExitFatal, ExitSucceed, FeedbackInterpreter, Interpreter,
	runtime::{
		Context, GasState, RuntimeBackend, RuntimeEnvironment, RuntimeState, SetCodeOrigin,
		TouchKind, TransactionContext, Transfer,
	},
	trap::{
		CallCreateTrap, CallFeedback, CallScheme, CallTrap, CreateFeedback, CreateScheme,
		CreateTrap, TrapConsume,
	},
};
use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

pub use self::{
	resolver::{EtableResolver, PrecompileSet, Resolver, ResolverOrigin},
	state::InvokerState,
};
use crate::{
	MergeStrategy,
	backend::TransactionalBackend,
	invoker::{Invoker as InvokerT, InvokerControl, InvokerExit},
	standard::Config,
};

/// The invoke used in a substack.
pub enum SubstackInvoke {
	/// CALL-alike opcode.
	Call {
		/// The trap of the call.
		trap: CallTrap,
	},
	/// CREATE-alike opcodes.
	Create {
		/// The trap of CREATE/CREATE2.
		trap: CreateTrap,
		/// Resolved target address.
		address: H160,
	},
}

/// Return value of a transaction. Call and create status.
#[derive(Clone, Debug)]
pub enum TransactValueCallCreate {
	/// Result of a call transaction.
	Call {
		/// The exit result. If we return a value, then it will be an
		/// `ExitSucceed`.
		succeed: ExitSucceed,
		/// The return value, if any.
		retval: Vec<u8>,
	},
	/// Result of a create transaction.
	Create {
		/// The exit result. If we return a value, then it will be an
		/// `ExitSucceed`.
		succeed: ExitSucceed,
		/// The contract address created.
		address: H160,
	},
}

/// Complete return value of a transaction.
#[derive(Clone, Debug)]
pub struct TransactValue {
	/// Call/Create status.
	pub call_create: TransactValueCallCreate,
	/// Used gas.
	pub used_gas: U256,
}

/// Transact gas price.
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum TransactGasPrice {
	/// Legacy gas price.
	Legacy(U256),
	/// EIP-1559 fee market.
	FeeMarket {
		/// `max_priority_fee_per_gas` according to EIP-1559.
		max_priority: U256,
		/// `max_fee_per_gas` according to EIP-1559.
		max: U256,
	},
}

impl TransactGasPrice {
	/// Caller fee.
	pub fn fee<H: RuntimeEnvironment>(
		&self,
		gas_limit: U256,
		config: &Config,
		handler: &H,
	) -> U256 {
		let effective_gas_price = self.effective_gas_price(config, handler);
		gas_limit.saturating_mul(effective_gas_price)
	}

	/// Refunded caller fee after call.
	pub fn refunded_fee<H: RuntimeEnvironment>(
		&self,
		refunded_gas: U256,
		config: &Config,
		handler: &H,
	) -> U256 {
		let effective_gas_price = self.effective_gas_price(config, handler);
		refunded_gas.saturating_mul(effective_gas_price)
	}

	/// Coinbase reward.
	pub fn coinbase_reward<H: RuntimeEnvironment>(
		&self,
		used_gas: U256,
		config: &Config,
		handler: &H,
	) -> U256 {
		if config.eip1559_fee_market {
			let max_priority = match self {
				Self::Legacy(gas_price) => *gas_price,
				Self::FeeMarket { max_priority, .. } => *max_priority,
			};
			let max = match self {
				Self::Legacy(gas_price) => *gas_price,
				Self::FeeMarket { max, .. } => *max,
			};
			let priority = min(
				max_priority,
				max.saturating_sub(handler.block_base_fee_per_gas()),
			);
			used_gas.saturating_mul(priority)
		} else {
			let effective_gas_price = self.effective_gas_price(config, handler);
			used_gas.saturating_mul(effective_gas_price)
		}
	}

	/// Effective gas price as returned by `GASPRICE` opcode.
	pub fn effective_gas_price<H: RuntimeEnvironment>(&self, config: &Config, handler: &H) -> U256 {
		if config.eip1559_fee_market {
			let max_priority = match self {
				Self::Legacy(gas_price) => *gas_price,
				Self::FeeMarket { max_priority, .. } => *max_priority,
			};
			let max = match self {
				Self::Legacy(gas_price) => *gas_price,
				Self::FeeMarket { max, .. } => *max,
			};

			let priority = min(
				max_priority,
				max.saturating_sub(handler.block_base_fee_per_gas()),
			);
			priority.saturating_add(handler.block_base_fee_per_gas())
		} else {
			match self {
				Self::Legacy(gas_price) => *gas_price,
				Self::FeeMarket { max_priority, .. } => *max_priority,
			}
		}
	}
}

impl From<U256> for TransactGasPrice {
	fn from(gas_price: U256) -> Self {
		Self::Legacy(gas_price)
	}
}

/// The invoke used in a top-layer transaction stack.
pub struct TransactInvoke<'config> {
	/// Create address, if it is a create transaction.
	pub create_address: Option<H160>,
	/// Gas limit.
	pub gas_limit: U256,
	/// Gas price.
	pub gas_price: TransactGasPrice,
	/// Caller.
	pub caller: H160,
	/// Config used for the transaction.
	pub config: &'config Config,
}

#[derive(Clone, Debug)]
/// Call/Create information used by [TransactArgs].
pub enum TransactArgsCallCreate {
	/// A call transaction.
	Call {
		/// Transaction target.
		address: H160,
		/// Transaction call data.
		data: Vec<u8>,
	},
	/// A create transaction.
	Create {
		/// Init code.
		init_code: Vec<u8>,
		/// Salt of `CREATE2`. `None` for a normal create transaction.
		salt: Option<H256>,
	},
}

/// Transaction arguments.
#[derive(Clone, Debug)]
pub struct TransactArgs<'config> {
	/// Call/Create information.
	pub call_create: TransactArgsCallCreate,
	/// Transaction sender.
	pub caller: H160,
	/// Transaction value.
	pub value: U256,
	/// Transaction gas limit.
	pub gas_limit: U256,
	/// Transaction gas price.
	pub gas_price: TransactGasPrice,
	/// Access list information, in the format of (address, storage keys).
	pub access_list: Vec<(H160, Vec<H256>)>,
	/// Config of this arg.
	pub config: &'config Config,
}

impl<'config> AsRef<TransactArgs<'config>> for TransactArgs<'config> {
	fn as_ref(&self) -> &TransactArgs<'config> {
		self
	}
}

/// Standard invoker.
///
/// The generic parameters are as follows:
/// * `S`: The runtime state, usually [RuntimeState] but can be customized.
/// * `H`: Backend type.
/// * `R`: Code resolver type, also handle precompiles. Usually
///   [EtableResolver] but can be customized.
/// * `Tr`: Trap type, usually [crate::interpreter::Opcode] but can be customized.
pub struct Invoker<'config, 'resolver, R> {
	resolver: &'resolver R,
	_marker: PhantomData<&'config Config>,
}

impl<'config, 'resolver, R> Invoker<'config, 'resolver, R> {
	/// Create a new standard invoker with the given config and resolver.
	pub const fn new(resolver: &'resolver R) -> Self {
		Self {
			resolver,
			_marker: PhantomData,
		}
	}
}

impl<'config, 'resolver, H, R> InvokerT<H> for Invoker<'config, 'resolver, R>
where
	R::State: InvokerState + AsRef<RuntimeState> + AsMut<RuntimeState> + AsRef<Config>,
	<R::State as InvokerState>::TransactArgs: AsRef<TransactArgs<'config>>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
	<R::Interpreter as Interpreter<H>>::Trap: TrapConsume<CallCreateTrap>,
	R::Interpreter: FeedbackInterpreter<H, CallFeedback> + FeedbackInterpreter<H, CreateFeedback>,
{
	type State = R::State;
	type Interpreter = R::Interpreter;
	type Interrupt =
		<<R::Interpreter as Interpreter<H>>::Trap as TrapConsume<CallCreateTrap>>::Rest;
	type TransactArgs = <<R::Interpreter as Interpreter<H>>::State as InvokerState>::TransactArgs;
	type TransactValue = TransactValue;
	type TransactInvoke = TransactInvoke<'config>;
	type SubstackInvoke = SubstackInvoke;

	fn new_transact(
		&self,
		args: Self::TransactArgs,
		handler: &mut H,
	) -> Result<
		(
			Self::TransactInvoke,
			InvokerControl<Self::Interpreter, Self::State>,
		),
		ExitError,
	> {
		let caller = AsRef::<TransactArgs>::as_ref(&args).caller;
		let gas_price = AsRef::<TransactArgs>::as_ref(&args).gas_price;
		let gas_fee = gas_price.fee(
			AsRef::<TransactArgs>::as_ref(&args).gas_limit,
			AsRef::<TransactArgs>::as_ref(&args).config,
			handler,
		);
		let coinbase = handler.block_coinbase();

		let address = match &AsRef::<TransactArgs>::as_ref(&args).call_create {
			TransactArgsCallCreate::Call { address, .. } => *address,
			TransactArgsCallCreate::Create {
				salt, init_code, ..
			} => {
				if let Some(limit) = AsRef::<TransactArgs<'config>>::as_ref(&args)
					.config
					.max_initcode_size()
					&& init_code.len() > limit
				{
					return Err(ExitException::CreateContractLimit.into());
				}

				match salt {
					Some(salt) => {
						let scheme = CreateScheme::Create2 {
							caller,
							code_hash: H256::from_slice(Keccak256::digest(init_code).as_slice()),
							salt: *salt,
						};
						scheme.address(handler)
					}
					None => {
						let scheme = CreateScheme::Legacy { caller };
						scheme.address(handler)
					}
				}
			}
		};
		let value = AsRef::<TransactArgs>::as_ref(&args).value;
		let gas_limit = AsRef::<TransactArgs>::as_ref(&args).gas_limit;

		let invoke = TransactInvoke {
			gas_limit,
			gas_price: AsRef::<TransactArgs>::as_ref(&args).gas_price,
			caller: AsRef::<TransactArgs>::as_ref(&args).caller,
			create_address: match AsRef::<TransactArgs>::as_ref(&args).call_create {
				TransactArgsCallCreate::Call { .. } => None,
				TransactArgsCallCreate::Create { .. } => Some(address),
			},
			config: AsRef::<TransactArgs>::as_ref(&args).config,
		};

		handler.mark_hot(coinbase, TouchKind::Coinbase);

		if handler.code_size(caller) != U256::zero() {
			handler.push_substate();
			return Ok((
				invoke,
				InvokerControl::DirectExit(InvokerExit {
					result: Err(ExitException::NotEOA.into()),
					substate: None,
					retval: Vec::new(),
				}),
			));
		}

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
			gas_price: gas_price
				.effective_gas_price(AsRef::<TransactArgs>::as_ref(&args).config, handler),
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

		let access_list = AsRef::<TransactArgs>::as_ref(&args).access_list.clone();
		for (address, keys) in &access_list {
			handler.mark_hot(*address, TouchKind::Access);
			for key in keys {
				handler.mark_storage_hot(*address, *key);
			}
		}

		handler.mark_hot(caller, TouchKind::Access);
		handler.mark_hot(caller, TouchKind::StateChange);
		handler.mark_hot(address, TouchKind::Access);

		let machine = match &AsRef::<TransactArgs>::as_ref(&args).call_create {
			TransactArgsCallCreate::Call { data, .. } => {
				let state = <<R::Interpreter as Interpreter<H>>::State>::new_transact_call(
					runtime_state,
					gas_limit,
					data,
					&access_list,
					&args,
				)?;

				routines::make_enter_call_machine(
					ResolverOrigin::Transaction,
					self.resolver,
					CallScheme::Call,
					address,
					data.clone(),
					Some(transfer),
					state,
					handler,
				)?
			}
			TransactArgsCallCreate::Create { init_code, .. } => {
				let state = <<R::Interpreter as Interpreter<H>>::State>::new_transact_create(
					runtime_state,
					gas_limit,
					init_code,
					&access_list,
					&args,
				)?;

				routines::make_enter_create_machine(
					ResolverOrigin::Transaction,
					self.resolver,
					caller,
					init_code.clone(),
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
							match routines::deploy_create_code(
								address,
								retbuf,
								substate,
								handler,
								SetCodeOrigin::Transaction,
							) {
								Ok(()) => Ok(TransactValueCallCreate::Create {
									succeed: result,
									address,
								}),
								Err(e)
									if AsRef::<Config>::as_ref(&substate)
										.eip2_no_empty_contract =>
								{
									Err(e)
								}
								Err(_) => Ok(TransactValueCallCreate::Create {
									succeed: result,
									address,
								}),
							}
						} else {
							Err(ExitFatal::Unfinished.into())
						}
					} else {
						Ok(TransactValueCallCreate::Call {
							succeed: result,
							retval: exit.retval,
						})
					}
				}
				Err(result) => Err(result),
			}
		};

		let result = work();

		let effective_gas = match result {
			Ok(_) => exit
				.substate
				.as_ref()
				.map(|s| s.effective_gas(true))
				.unwrap_or_default(),
			Err(ExitError::Reverted) => exit
				.substate
				.as_ref()
				.map(|s| s.effective_gas(false))
				.unwrap_or_default(),
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

		let used_gas = invoke.gas_limit.saturating_sub(effective_gas);
		let refunded_fee = invoke
			.gas_price
			.refunded_fee(effective_gas, invoke.config, handler);
		handler.deposit(invoke.caller, refunded_fee);
		let coinbase_reward = invoke
			.gas_price
			.coinbase_reward(used_gas, invoke.config, handler);
		handler.deposit(handler.block_coinbase(), coinbase_reward);

		result.map(|call_create| TransactValue {
			call_create,
			used_gas,
		})
	}

	fn enter_substack(
		&self,
		trap: <R::Interpreter as Interpreter<H>>::Trap,
		machine: &mut Self::Interpreter,
		handler: &mut H,
		depth: usize,
	) -> Capture<
		Result<
			(
				Self::SubstackInvoke,
				InvokerControl<Self::Interpreter, Self::State>,
			),
			ExitError,
		>,
		Self::Interrupt,
	> {
		fn l64(gas: U256) -> U256 {
			gas - gas / U256::from(64)
		}

		let trap_data = match trap.consume() {
			Ok(trap_data) => trap_data,
			Err(interrupt) => return Capture::Trap(Box::new(interrupt)),
		};

		let invoke = match trap_data {
			CallCreateTrap::Call(call_trap_data) => SubstackInvoke::Call {
				trap: call_trap_data,
			},
			CallCreateTrap::Create(create_trap_data) => {
				let address = create_trap_data.scheme.address(handler);
				SubstackInvoke::Create {
					address,
					trap: create_trap_data,
				}
			}
		};

		let after_gas = if AsRef::<Config>::as_ref(machine.state()).eip150_call_l64_after_gas {
			l64(machine.state().gas())
		} else {
			machine.state().gas()
		};
		let target_gas = match &invoke {
			SubstackInvoke::Call { trap, .. } => trap.gas,
			SubstackInvoke::Create { .. } => after_gas,
		};
		let gas_limit = min(after_gas, target_gas);

		let call_has_value = matches!(&invoke, SubstackInvoke::Call { trap: call_trap_data } if call_trap_data.has_value());

		let is_static = if machine.state().is_static() {
			true
		} else {
			match &invoke {
				SubstackInvoke::Call {
					trap: CallTrap { is_static, .. },
				} => *is_static,
				_ => false,
			}
		};

		let transaction_context = {
			AsRef::<RuntimeState>::as_ref(machine.state())
				.transaction_context
				.clone()
		};

		match invoke {
			SubstackInvoke::Call {
				trap: call_trap_data,
			} => {
				let substate = match machine.state_mut().substate(
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

				if depth > AsRef::<Config>::as_ref(machine.state()).call_stack_limit() {
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

				if let Some(transfer) = &call_trap_data.transfer
					&& transfer.value != U256::zero()
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

				// EIP-2929 and EIP-161 has two different rules. In EIP-161, the
				// touch is reverted if a call fails. In EIP-2929, the touch
				// stays warm.
				handler.mark_hot(call_trap_data.context.address, TouchKind::Access);

				let target = call_trap_data.target;

				Capture::Exit(routines::enter_call_substack(
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

				if let Some(limit) = AsRef::<Config>::as_ref(&machine.state()).max_initcode_size()
					&& code.len() > limit
				{
					return Capture::Exit(Err(ExitException::CreateContractLimit.into()));
				}

				let substate = match machine.state_mut().substate(
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

				if depth > AsRef::<Config>::as_ref(machine.state()).call_stack_limit() {
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

				// EIP-2929 and EIP-161 has two different rules. In EIP-161, the
				// touch is reverted if a call fails. In EIP-2929, the touch
				// stays warm.
				handler.mark_hot(address, TouchKind::Access);

				Capture::Exit(routines::enter_create_substack(
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
		let mut strategy = match &exit.result {
			Ok(_) => MergeStrategy::Commit,
			Err(ExitError::Exception(ExitException::OutOfFund)) => MergeStrategy::Revert,
			Err(ExitError::Exception(ExitException::CallTooDeep)) => MergeStrategy::Revert,
			Err(ExitError::Exception(ExitException::MaxNonce)) => MergeStrategy::Revert,
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
								Err(err)
									if AsRef::<Config>::as_ref(&substate)
										.eip2_no_empty_contract =>
								{
									retbuf = Vec::new();
									strategy = MergeStrategy::Discard;
									Err(err)
								}
								Err(_) => {
									retbuf = Vec::new();
									Ok(address)
								}
							}
						}
						Err(err) => Err(err),
					};

					parent.state_mut().merge(substate, strategy);
					result
				} else {
					Err(ExitFatal::Unfinished.into())
				};

				handler.pop_substate(strategy)?;

				let feedback = CreateFeedback {
					reason: result,
					retbuf,
					trap,
				};
				parent.feedback(feedback, handler)
			}
			SubstackInvoke::Call { trap } => {
				let retbuf = exit.retval;

				if let Some(substate) = exit.substate {
					parent.state_mut().merge(substate, strategy);
				}
				handler.pop_substate(strategy)?;

				let feedback = CallFeedback {
					reason: exit.result,
					retbuf,
					trap,
				};
				parent.feedback(feedback, handler)
			}
		}
	}
}
