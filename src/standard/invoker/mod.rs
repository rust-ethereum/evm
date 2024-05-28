mod resolver;
pub mod routines;
mod state;

use alloc::{rc::Rc, vec::Vec};
use core::{cmp::min, convert::Infallible};

use evm_interpreter::{
	error::{
		CallCreateTrap, CallCreateTrapData, CallTrapData, Capture, CreateScheme, CreateTrapData,
		ExitError, ExitException, ExitResult, ExitSucceed, TrapConsume,
	},
	opcode::Opcode,
	runtime::{
		Context, GasState, RuntimeBackend, RuntimeEnvironment, RuntimeState, TransactionContext,
		Transfer,
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
	invoker::{Invoker as InvokerT, InvokerControl},
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
	pub gas_fee: U256,
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
	) -> Result<
		(
			Self::TransactInvoke,
			InvokerControl<Self::Interpreter, (ExitResult, (R::State, Vec<u8>))>,
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
		let runtime_state = RuntimeState {
			context,
			transaction_context: Rc::new(transaction_context),
			retbuf: Vec::new(),
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

					let state = <R::State>::new_transact_call(
						runtime_state,
						gas_limit,
						&data,
						&access_list,
						self.config,
					)?;

					let machine = routines::make_enter_call_machine(
						self.config,
						self.resolver,
						address,
						data,
						Some(transfer),
						state,
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
					let state = <R::State>::new_transact_create(
						runtime_state,
						gas_limit,
						&init_code,
						&access_list,
						self.config,
					)?;

					let machine = routines::make_enter_create_machine(
						self.config,
						self.resolver,
						caller,
						init_code,
						transfer,
						state,
						handler,
					)?;

					Ok((invoke, machine))
				}
			}
		};

		work().map_err(|err| {
			handler.pop_substate(MergeStrategy::Discard);
			err
		})
	}

	fn finalize_transact(
		&self,
		invoke: &Self::TransactInvoke,
		result: ExitResult,
		(mut substate, retval): (R::State, Vec<u8>),
		handler: &mut H,
	) -> Result<Self::TransactValue, ExitError> {
		let left_gas = substate.effective_gas();

		let work = || -> Result<Self::TransactValue, ExitError> {
			match result {
				Ok(result) => {
					if let Some(address) = invoke.create_address {
						let retbuf = retval;

						routines::deploy_create_code(
							self.config,
							address,
							retbuf,
							&mut substate,
							handler,
						)?;

						Ok(TransactValue::Create {
							succeed: result,
							address,
						})
					} else {
						Ok(TransactValue::Call {
							succeed: result,
							retval,
						})
					}
				}
				Err(result) => Err(result),
			}
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
		machine: &mut Self::Interpreter,
		handler: &mut H,
		depth: usize,
	) -> Capture<
		Result<
			(
				Self::SubstackInvoke,
				InvokerControl<Self::Interpreter, (ExitResult, (R::State, Vec<u8>))>,
			),
			ExitError,
		>,
		Self::Interrupt,
	> {
		fn l64(gas: U256) -> U256 {
			gas - gas / U256::from(64)
		}

		let opcode = match trap.consume() {
			Ok(opcode) => opcode,
			Err(interrupt) => return Capture::Trap(interrupt),
		};

		if depth >= self.config.call_stack_limit {
			return Capture::Exit(Err(ExitException::CallTooDeep.into()));
		}

		let trap_data = match CallCreateTrapData::new_from(opcode, machine.machine_mut()) {
			Ok(trap_data) => trap_data,
			Err(err) => return Capture::Exit(Err(err)),
		};

		let after_gas = if self.config.call_l64_after_gas {
			l64(machine.machine().state.gas())
		} else {
			machine.machine().state.gas()
		};
		let target_gas = trap_data.target_gas().unwrap_or(after_gas);
		let gas_limit = min(after_gas, target_gas);

		let call_has_value =
			matches!(&trap_data, CallCreateTrapData::Call(call) if call.has_value());

		let is_static = if machine.machine().state.is_static() {
			true
		} else {
			match &trap_data {
				CallCreateTrapData::Call(CallTrapData { is_static, .. }) => *is_static,
				_ => false,
			}
		};

		let transaction_context = machine.machine().state.as_ref().transaction_context.clone();

		match trap_data {
			CallCreateTrapData::Call(call_trap_data) => {
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

				let target = call_trap_data.target;

				Capture::Exit(routines::enter_call_substack(
					self.config,
					self.resolver,
					call_trap_data,
					target,
					substate,
					handler,
				))
			}
			CallCreateTrapData::Create(create_trap_data) => {
				let caller = create_trap_data.scheme.caller();
				let address = create_trap_data.scheme.address(handler);
				let code = create_trap_data.code.clone();

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
					Ok(submeter) => submeter,
					Err(err) => return Capture::Exit(Err(err)),
				};

				Capture::Exit(routines::enter_create_substack(
					self.config,
					self.resolver,
					code,
					create_trap_data,
					substate,
					handler,
				))
			}
		}
	}

	fn exit_substack(
		&self,
		result: ExitResult,
		(mut substate, retval): (R::State, Vec<u8>),
		trap_data: Self::SubstackInvoke,
		parent: &mut Self::Interpreter,
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

				let result = result.and_then(|_| {
					routines::deploy_create_code(
						self.config,
						address,
						retbuf.clone(),
						&mut substate,
						handler,
					)?;

					Ok(address)
				});

				parent.machine_mut().state.merge(substate, strategy);
				handler.pop_substate(strategy);

				trap.feedback(result, retbuf, parent)?;

				Ok(())
			}
			SubstackInvoke::Call { trap } => {
				let retbuf = retval;

				parent.machine_mut().state.merge(substate, strategy);
				handler.pop_substate(strategy);

				trap.feedback(result, retbuf, parent)?;

				Ok(())
			}
		}
	}
}
