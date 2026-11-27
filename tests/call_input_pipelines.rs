//! Test which pipelines share the input of a call, and which keep it as they
//! were.

#[allow(dead_code)]
mod mock;
mod support;

use std::{cell::RefCell, cmp::min, convert::Infallible};

use evm::{
	InvokerControl, InvokerExit,
	backend::{OverlayedBackend, TransactionalBackend},
	interpreter::{
		Capture, Control, ExitError, FeedbackInterpreter, Interpreter, Machine, Opcode,
		etable::{Chained, DispatchEtable, Single},
		runtime::{
			Context, GasState, RuntimeBackend, RuntimeEnvironment, RuntimeState, TouchKind,
			Transfer,
		},
		trap::{CallCreateTrap, CallFeedback, CallScheme, CallTrap, CreateFeedback},
	},
	standard::{
		Config, EtableResolver, Invoker as StandardInvoker, InvokerState, Resolver, State,
		SubstackInvoke, TransactInvoke, TransactValue, eval_gasometer, routines,
	},
	uint::{H160, U256, U256Ext},
};
use support::*;

/// Address of the contract that an invoker of a test handles itself.
const HANDLED: u64 = 0x0f00;

/// The contract that calls `handled` and then the echo contract, and returns
/// both results.
///
/// ```text
/// 7f ..     PUSH32 word     value 0x11 repeated
/// 60 00     PUSH1 0         offset
/// 52        MSTORE          memory[0..32) = word
/// <STATICCALL to `handled`, input 0..32, output 0x100..0x120>
/// 50        POP             the call result
/// 7f ..     PUSH32 word     value 0x22 repeated
/// 60 20     PUSH1 32        offset
/// 52        MSTORE          memory[32..64) = word
/// <STATICCALL to ECHO, input 32..64, output 0x120..0x140>
/// 50        POP             the call result
/// 60 40     PUSH1 64        size
/// 61 01 00  PUSH2 0x100     offset
/// f3        RETURN          return memory[0x100..0x140)
/// ```
fn two_calls(handled: u64) -> Vec<u8> {
	code(&[
		&store_word(0x11, 0),
		&CallOp::static_call(handled, (0, 32), (0x100, 32)).hex_and_pop(),
		&store_word(0x22, 32),
		&CallOp::static_call(ECHO, (32, 32), (0x120, 32)).hex_and_pop(),
		&return_range(0x100, 64),
	])
}

/// The bytes that [`two_calls`] must return.
fn two_calls_expected() -> Vec<u8> {
	[vec![0x11u8; 32], vec![0x22u8; 32]].concat()
}

/// The accounts of a test whose contract calls [`HANDLED`] and then [`ECHO`].
fn two_calls_accounts() -> Vec<(u64, Vec<u8>)> {
	vec![
		(TOP, two_calls(HANDLED)),
		(HANDLED, echo_contract()),
		(ECHO, echo_contract()),
	]
}

/// The contract that makes a shared call, then a call whose trap the test's
/// opcode function builds itself, and returns its result.
///
/// ```text
/// 7f ..     PUSH32 word     value 0x11 repeated
/// 60 00     PUSH1 0         offset
/// 52        MSTORE          memory[0..32) = word
/// <STATICCALL to ECHO, input 0..32, output 0x100..0x120>
/// 50        POP             the call result
/// <CALL to the callee, through the opcode function of the test>
/// 50        POP             the call result
/// 60 20     PUSH1 32        size
/// 61 01 20  PUSH2 0x120     offset
/// f3        RETURN          return memory[0x120..0x140)
/// ```
fn shared_then_own_call(callee: u64) -> Vec<u8> {
	code(&[
		&store_word(0x11, 0),
		&CallOp::static_call(ECHO, (0, 32), (0x100, 32)).hex_and_pop(),
		&CallOp::call(callee, (0, 32), (0x120, 32)).hex_and_pop(),
		&return_range(0x120, 32),
	])
}

/// The trap of a call that an opcode function of the test builds with a struct
/// literal.
///
/// The caller pushes the seven arguments of `CALL`, so this pops them. The
/// target, the input and the output range are the ones that the test builds
/// this function with.
#[allow(deprecated)]
fn own_call_trap<S>(
	machine: &mut Machine<S>,
	target: H160,
	input: Vec<u8>,
	output: (usize, usize),
) -> Result<CallCreateTrap, ExitError>
where
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
{
	let gas = machine.stack.pop()?;
	let _target = machine.stack.pop()?;
	let _value = machine.stack.pop()?;
	let _in_offset = machine.stack.pop()?;
	let _in_len = machine.stack.pop()?;
	let _out_offset = machine.stack.pop()?;
	let _out_len = machine.stack.pop()?;

	machine.memory.resize_end(U256::from(output.0 + output.1))?;

	let caller = AsRef::<RuntimeState>::as_ref(&machine.state)
		.context
		.address;

	Ok(CallCreateTrap::Call(CallTrap {
		target,
		transfer: Some(Transfer {
			source: caller,
			target,
			value: U256::ZERO,
		}),
		input,
		gas,
		is_static: false,
		out_offset: U256::from(output.0),
		out_len: U256::from(output.1),
		context: Context {
			address: target,
			caller,
			apparent_value: U256::ZERO,
		},
		scheme: CallScheme::Call,
	}))
}

/// The trap of a call that an opcode function of the test builds with
/// [CallTrap::new_from], which copies the input.
fn copied_call_trap<S>(machine: &mut Machine<S>) -> Result<CallCreateTrap, ExitError>
where
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
{
	Ok(CallCreateTrap::Call(CallTrap::new_from(
		CallScheme::Call,
		machine,
	)?))
}

/// An invoker that delegates to the standard invoker, and handles the calls to
/// one address itself, with [routines::enter_call_substack].
struct DelegatingInvoker<'a, 'config, 'resolver, R> {
	/// The standard invoker that handles every other trap.
	standard: &'a StandardInvoker<'config, 'resolver, R>,
	/// The resolver that the handled calls are resolved with.
	resolver: &'resolver R,
	/// The address that this invoker handles.
	handled: H160,
	/// The input of every call trap that this invoker handled.
	seen: RefCell<Vec<Vec<u8>>>,
}

impl<'a, 'config, 'resolver, H, R> evm::Invoker<H> for DelegatingInvoker<'a, 'config, 'resolver, R>
where
	R::State: InvokerState + AsRef<RuntimeState> + AsMut<RuntimeState> + AsRef<Config>,
	<R::State as InvokerState>::TransactArgs: AsRef<evm::standard::TransactArgs<'config>>,
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
	R::Interpreter: Interpreter<H, Trap = CallCreateTrap>
		+ FeedbackInterpreter<H, CallFeedback>
		+ FeedbackInterpreter<H, CreateFeedback>,
{
	type State = R::State;
	type Interpreter = R::Interpreter;
	type Interrupt = Infallible;
	type TransactArgs = <R::State as InvokerState>::TransactArgs;
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
		self.standard.new_transact(args, handler)
	}

	fn finalize_transact(
		&self,
		invoke: &Self::TransactInvoke,
		exit: InvokerExit<Self::State>,
		handler: &mut H,
	) -> Result<Self::TransactValue, ExitError> {
		self.standard.finalize_transact(invoke, exit, handler)
	}

	#[allow(deprecated)]
	fn enter_substack(
		&self,
		trap: CallCreateTrap,
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
		match trap {
			CallCreateTrap::Call(call) if call.target == self.handled => {
				self.seen.borrow_mut().push(call.input.clone());

				Capture::Exit(enter_handled_call(self.resolver, call, machine, handler))
			}
			trap => self.standard.enter_substack(trap, machine, handler, depth),
		}
	}

	fn exit_substack(
		&self,
		trap_data: Self::SubstackInvoke,
		exit: InvokerExit<Self::State>,
		parent: &mut Self::Interpreter,
		handler: &mut H,
	) -> Result<(), ExitError> {
		self.standard
			.exit_substack(trap_data, exit, parent, handler)
	}
}

/// Enter a call that a test's invoker handles itself. It does what the standard
/// invoker does before a call, and then calls
/// [routines::enter_call_substack].
#[allow(clippy::type_complexity)]
fn enter_handled_call<H, R>(
	resolver: &R,
	trap: CallTrap,
	machine: &mut R::Interpreter,
	handler: &mut H,
) -> Result<(SubstackInvoke, InvokerControl<R::Interpreter, R::State>), ExitError>
where
	H: RuntimeEnvironment + RuntimeBackend + TransactionalBackend,
	R: Resolver<H>,
	R::State: InvokerState + AsRef<RuntimeState> + AsMut<RuntimeState> + AsRef<Config>,
	R::Interpreter: Interpreter<H, Trap = CallCreateTrap>,
{
	let after_gas = if AsRef::<Config>::as_ref(machine.state()).eip150_call_l64_after_gas {
		let gas = machine.state().gas();

		gas - gas / U256::VALUE_64
	} else {
		machine.state().gas()
	};
	let gas_limit = min(after_gas, trap.gas);
	let is_static = machine.state().is_static() || trap.is_static;
	let call_has_value = trap.has_value();
	let transaction_context = AsRef::<RuntimeState>::as_ref(machine.state())
		.transaction_context
		.clone();

	let substate = machine.state_mut().substate(
		RuntimeState {
			context: trap.context.clone(),
			transaction_context,
			retbuf: Vec::new(),
		},
		gas_limit,
		is_static,
		call_has_value,
	)?;

	handler.mark_hot(trap.context.address, TouchKind::Access);

	let target = trap.target;

	routines::enter_call_substack(resolver, trap, target, substate, handler)
}

/// Run the transaction through every call stack, and assert its return data.
macro_rules! run_every_path {
	($config:expr, $invoker:expr, $accounts:expr, $expected:expr) => {{
		for path in Path::ALL {
			let mut backend = OverlayedBackend::new(support::backend(&$accounts), &$config.runtime);
			let args = support::call_args(&$config, TOP, Vec::new());

			let value = match path {
				Path::Recursive => {
					support::transact_with_invoker(&$invoker, None, &mut backend, args)
				}
				Path::Hybrid => {
					support::transact_with_invoker(&$invoker, Some(1), &mut backend, args)
				}
				Path::Heap => support::heap_transact_with_invoker(&$invoker, &mut backend, args),
			}
			.unwrap_or_else(|err| panic!("{}: the transaction failed with {err:?}", path.name()));

			assert_eq!(support::retval(&value), &($expected)[..], "{}", path.name());
		}
	}};
}

#[test]
fn a_delegating_invoker_copies_the_input() {
	let config = Config::cancun();
	let etable = support::etable();
	let resolver = support::resolver(&etable, &());
	let standard = StandardInvoker::new(&resolver);
	let invoker = DelegatingInvoker {
		standard: &standard,
		resolver: &resolver,
		handled: H160::from_low_u64_be(HANDLED),
		seen: RefCell::new(Vec::new()),
	};

	run_every_path!(config, invoker, two_calls_accounts(), two_calls_expected());

	// The invoker does not declare sharing, so every machine that the call stack
	// runs copies the input, and the trap that it handled carries the bytes.
	let seen = invoker.seen.borrow();

	assert_eq!(seen.len(), Path::ALL.len());
	for input in seen.iter() {
		assert_eq!(input, &vec![0x11u8; 32]);
	}
}

#[test]
fn a_wrapper_invoker_sees_the_input_of_a_trap() {
	for forward in [false, true] {
		let config = Config::cancun();
		let etable = support::etable();
		let resolver = support::resolver(&etable, &());
		let standard = StandardInvoker::new(&resolver);
		let invoker = RecordingInvoker::new(&standard, forward);

		run_every_path!(config, invoker, two_calls_accounts(), two_calls_expected());

		// Without opting in, the wrapper sees the bytes that the trap carries.
		// When it forwards the declaration, the standard opcode function leaves
		// the input to the machine, and the wrapper sees an empty vector.
		let seen = invoker.seen();

		assert_eq!(seen.len(), 2 * Path::ALL.len());
		for input in seen {
			if forward {
				assert!(input.is_empty(), "the wrapper sees an empty input");
			} else {
				assert_eq!(input.len(), 32);
			}
		}
	}
}

#[test]
fn an_interpreter_without_a_machine_does_not_share() {
	let config = Config::cancun();
	let etable = support::etable();
	let resolver = support::resolver(&etable, &());
	let wrapped = support::wrap::WrappedResolver::<(), _, false>::new(&resolver);
	let standard = StandardInvoker::new(&wrapped);
	let invoker = RecordingInvoker::new(&standard, true);

	run_every_path!(config, invoker, two_calls_accounts(), two_calls_expected());

	// The invoker declares sharing, but the interpreter does not expose its
	// machine, so the traps carry the copied input.
	let seen = invoker.seen();

	assert_eq!(seen.len(), 2 * Path::ALL.len());
	for input in seen {
		assert_eq!(input.len(), 32);
	}
}

#[test]
fn an_interpreter_that_forwards_its_machine_shares() {
	let config = Config::cancun();
	let etable = support::etable();
	let resolver = support::resolver(&etable, &());
	let wrapped = support::wrap::WrappedResolver::<(), _, true>::new(&resolver);
	let standard = StandardInvoker::new(&wrapped);
	let invoker = RecordingInvoker::new(&standard, true);

	run_every_path!(config, invoker, two_calls_accounts(), two_calls_expected());

	let seen = invoker.seen();

	assert_eq!(seen.len(), 2 * Path::ALL.len());
	for input in seen {
		assert!(input.is_empty(), "the traps leave the input to the machine");
	}
}

#[test]
fn a_resolver_that_copies_the_input_gives_the_same_results() {
	let config = Config::cancun();
	let etable = support::etable();
	let resolver = support::resolver(&etable, &());
	let wrapped = support::wrap::CopyOnlyResolver::<(), _, true>::new(&resolver);
	let invoker = StandardInvoker::new(&wrapped);

	run_every_path!(config, invoker, two_calls_accounts(), two_calls_expected());
}

#[test]
fn an_opcode_function_that_copies_the_input() {
	let config = Config::cancun();
	let exec = DispatchEtable::runtime().wrap(|inner, opcode| {
		move |machine: &mut Machine<State<'_>>,
		      handler: &mut TestBackend<'_>,
		      position: usize|
		      -> Control<CallCreateTrap> {
			if opcode == Opcode::CALL {
				// The trap copies the input, also on a machine that shares it.
				match copied_call_trap(machine) {
					Ok(trap) => Control::Trap(Box::new(trap)),
					Err(err) => Control::Exit(Err(err)),
				}
			} else {
				inner(machine, handler, position)
			}
		}
	});
	let etable = Chained(Single::new(eval_gasometer), exec);
	let resolver = EtableResolver::new(&(), &etable);
	let standard = StandardInvoker::new(&resolver);
	let invoker = RecordingInvoker::new(&standard, true);

	let accounts = vec![(TOP, shared_then_own_call(ECHO)), (ECHO, echo_contract())];

	run_every_path!(config, invoker, accounts, words(&[0x11]));

	// The first call of the contract shares its input, and the second one gets a
	// trap that the test's opcode function builds with [CallTrap::new_from],
	// which copies it.
	let seen = invoker.seen();

	assert_eq!(seen.len(), 2 * Path::ALL.len());
	for input in seen.iter().filter(|input| !input.is_empty()) {
		assert_eq!(
			input,
			&vec![0x11u8; 32],
			"the trap carries the copied input"
		);
	}
	assert_eq!(
		seen.iter().filter(|input| !input.is_empty()).count(),
		Path::ALL.len()
	);
}

#[test]
fn a_trap_that_an_opcode_function_builds_itself() {
	for carried in [Vec::new(), vec![0xde, 0xad, 0xbe, 0xef]] {
		let config = Config::cancun();
		let target = H160::from_low_u64_be(CALLDATA_LEN);
		let carried_by_traps = carried.clone();
		let exec = DispatchEtable::runtime().wrap(move |inner, opcode| {
			let carried = carried_by_traps.clone();
			move |machine: &mut Machine<State<'_>>,
			      handler: &mut TestBackend<'_>,
			      position: usize|
			      -> Control<CallCreateTrap> {
				if opcode == Opcode::CALL {
					match own_call_trap(machine, target, carried.clone(), (0x120, 32)) {
						Ok(trap) => Control::Trap(Box::new(trap)),
						Err(err) => Control::Exit(Err(err)),
					}
				} else {
					inner(machine, handler, position)
				}
			}
		});
		let etable = Chained(Single::new(eval_gasometer), exec);
		let resolver = EtableResolver::new(&(), &etable);
		let invoker = StandardInvoker::new(&resolver);

		let accounts = vec![
			(TOP, shared_then_own_call(CALLDATA_LEN)),
			(ECHO, echo_contract()),
			(CALLDATA_LEN, calldata_len_contract()),
		];

		// The callee reports the length of its calldata, which is the length of
		// the input that the trap carries, and is never the range of the shared
		// call before it.
		run_every_path!(config, invoker, accounts, word(carried.len() as u64));
	}
}

#[test]
#[allow(deprecated)]
fn a_standard_trap_that_a_wrapper_gives_bytes() {
	let config = Config::cancun();
	let exec = DispatchEtable::runtime().wrap(|inner, opcode| {
		move |machine: &mut Machine<State<'_>>,
		      handler: &mut TestBackend<'_>,
		      position: usize|
		      -> Control<CallCreateTrap> {
			// Let the standard opcode function build the trap, which records
			// the input range of the call, and then give it bytes of its own.
			let control = inner(machine, handler, position);
			if opcode == Opcode::CALL
				&& let Control::Trap(mut trap) = control
			{
				if let CallCreateTrap::Call(call) = &mut *trap {
					call.input = vec![0xde, 0xad, 0xbe, 0xef];
				}

				return Control::Trap(trap);
			}
			control
		}
	});
	let etable = Chained(Single::new(eval_gasometer), exec);
	let resolver = EtableResolver::new(&(), &etable);
	let invoker = StandardInvoker::new(&resolver);
	let accounts = vec![
		(TOP, shared_then_own_call(CALLDATA_LEN)),
		(ECHO, echo_contract()),
		(CALLDATA_LEN, calldata_len_contract()),
	];

	// The trap carries four bytes, and the callee sees them rather than the
	// range that the opcode function recorded for the call.
	run_every_path!(config, invoker, accounts, word(4));
}
