//! Tests for sharing the input of a call with the callee.

use std::rc::Rc;

use evm_interpreter::{
	CallInput, Capture, Control, EtableInterpreter, ExitError, ExitException, ExitFatal,
	ExitResult, ExitSucceed, Machine, Opcode, StepInterpreter,
	etable::DispatchEtable,
	eval::eval_call_create_trap,
	runtime::{Context, RuntimeState, RuntimeStateAndConfig, TransactionContext},
	trap::{
		CallCreateOpcode, CallCreateTrap, CallFeedback, CallScheme, CallTrap, CreateFeedback,
		CreateScheme, CreateTrap,
	},
	uint::{H160, U256},
};

#[test]
fn call_input_is_send_and_sync() {
	fn assert_send_sync<T: Send + Sync>() {}
	assert_send_sync::<CallInput>();
}

type State = RuntimeStateAndConfig<'static>;
type Etable = DispatchEtable<State, (), CallCreateTrap>;

/// An exit reason for a feedback, and whether applying it fails.
type Reason = (&'static str, fn() -> ExitResult, bool);

fn state() -> State {
	RuntimeStateAndConfig::with_default_config(RuntimeState {
		context: Context {
			address: H160::default(),
			caller: H160::default(),
			apparent_value: U256::default(),
		},
		transaction_context: TransactionContext {
			gas_price: U256::default(),
			origin: H160::default(),
		}
		.into(),
		retbuf: Vec::new(),
	})
}

fn machine_with(opcode: Opcode) -> Machine<State> {
	Machine::new(
		Rc::new(vec![opcode.0, Opcode::STOP.0]),
		Rc::new(Vec::new()),
		1024,
		10000,
		state(),
	)
}

fn scheme_of(opcode: Opcode) -> CallScheme {
	match opcode {
		Opcode::CALL => CallScheme::Call,
		Opcode::CALLCODE => CallScheme::CallCode,
		Opcode::DELEGATECALL => CallScheme::DelegateCall,
		Opcode::STATICCALL => CallScheme::StaticCall,
		_ => unreachable!(),
	}
}

/// Push the arguments of a call opcode, from the bottom of the stack to the top.
fn push_args(machine: &mut Machine<State>, scheme: CallScheme, in_offset: u64, in_len: u64) {
	let mut args = vec![0, 0, in_len, in_offset];
	if matches!(scheme, CallScheme::Call | CallScheme::CallCode) {
		args.push(0);
	}
	args.push(0);
	args.push(0);
	for arg in args {
		machine.stack.push(U256::from(arg)).unwrap();
	}
}

/// Push the arguments of a create opcode, from the bottom of the stack to the top.
fn push_create_args(machine: &mut Machine<State>, create2: bool) {
	let mut args = Vec::new();
	if create2 {
		args.push(0); // salt
	}
	args.push(0); // code_len
	args.push(0); // code_offset
	args.push(0); // value
	for arg in args {
		machine.stack.push(U256::from(arg)).unwrap();
	}
}

/// Trap on the call opcode at position 0.
fn trap_call(machine: &mut Machine<State>) -> CallTrap {
	let mut handler = ();
	match eval_call_create_trap::<State, (), CallCreateTrap>(machine, &mut handler, 0) {
		Control::Trap(trap) => match *trap {
			CallCreateTrap::Call(trap) => trap,
			CallCreateTrap::Create(_) => panic!("expected a call trap"),
		},
		_ => panic!("expected a call trap"),
	}
}

fn setup(opcode: Opcode, share: bool) -> Machine<State> {
	let mut machine = machine_with(opcode);
	if share {
		machine.enable_call_input_sharing();
	}
	machine.memory.set(0, &[1, 2, 3, 4], None).unwrap();
	push_args(&mut machine, scheme_of(opcode), 0, 4);
	machine
}

/// Record the input range of a pending call, and drop the share.
fn pending_call_input(machine: &mut Machine<State>) {
	push_args(machine, CallScheme::Call, 0, 4);
	drop(trap_call(machine));
	assert!(machine.shared_call_input().is_some());
}

#[test]
#[allow(deprecated)]
fn shared_call_traps_leave_the_input_to_the_memory() {
	for opcode in [
		Opcode::CALL,
		Opcode::CALLCODE,
		Opcode::DELEGATECALL,
		Opcode::STATICCALL,
	] {
		let mut machine = setup(opcode, true);

		let trap = trap_call(&mut machine);

		assert!(trap.input.is_empty());
		let shared = machine.shared_call_input().unwrap();
		assert_eq!(&shared[..], &[1, 2, 3, 4][..]);
		assert_eq!(machine.memory.get(0, 4), vec![1, 2, 3, 4]);
	}
}

#[test]
#[allow(deprecated)]
fn call_trap_new_from_copies() {
	let mut machine = setup(Opcode::CALL, true);

	let trap = CallTrap::new_from(CallScheme::Call, &mut machine).unwrap();

	assert_eq!(trap.input, vec![1, 2, 3, 4]);
	assert!(machine.shared_call_input().is_none());

	// The trap constructor for all opcodes copies on a sharing machine, too.
	push_args(&mut machine, CallScheme::Call, 0, 4);
	let trap = CallCreateTrap::new_from(CallCreateOpcode::Call, &mut machine).unwrap();

	match trap {
		CallCreateTrap::Call(trap) => assert_eq!(trap.input, vec![1, 2, 3, 4]),
		CallCreateTrap::Create(_) => panic!("expected a call trap"),
	}
	assert!(machine.shared_call_input().is_none());
}

#[test]
fn feedback_takes_the_bytes_back() {
	// `reclaim` is the only take-back for the exit reasons that write no
	// return data, and for the ones that never reach `copy_large`.
	let cases: [Reason; 4] = [
		("returned", || Ok(ExitSucceed::Returned), false),
		("reverted", || Err(ExitError::Reverted), false),
		("exception", || Err(ExitException::OutOfGas.into()), false),
		("fatal", || Err(ExitFatal::NotSupported.into()), true),
	];

	for (name, reason, fatal) in cases {
		let mut machine = setup(Opcode::CALL, true);
		let ptr = machine.memory.data().as_ptr();

		let trap = trap_call(&mut machine);
		{
			let shared = machine.shared_call_input().unwrap();
			assert_eq!(&shared[..], &[1, 2, 3, 4][..], "{name}");
		}
		assert!(machine.memory.data().is_empty(), "{name}");

		let etable = Etable::core();
		let mut interpreter = EtableInterpreter::new(machine, &etable);
		let result = CallFeedback {
			trap,
			reason: reason(),
			retbuf: Vec::new(),
		}
		.to_machine(&mut interpreter);
		assert_eq!(result.is_err(), fatal, "{name}");

		assert_eq!(interpreter.memory.data().as_ptr(), ptr, "{name}");
		assert_eq!(interpreter.memory.data(), &vec![1, 2, 3, 4], "{name}");
		assert!(interpreter.as_mut().shared_call_input().is_none(), "{name}");

		interpreter.memory.set(0, &[5, 6, 7, 8], None).unwrap();
		assert_eq!(interpreter.memory.data().as_ptr(), ptr, "{name}");
		assert_eq!(interpreter.memory.data(), &vec![5, 6, 7, 8], "{name}");
	}
}

#[test]
#[allow(deprecated)]
fn no_share_copies_the_input() {
	let mut machine = setup(Opcode::CALL, false);

	let trap = trap_call(&mut machine);

	assert_eq!(trap.input, vec![1, 2, 3, 4]);
	assert!(machine.shared_call_input().is_none());
	assert_eq!(machine.memory.data(), &vec![1, 2, 3, 4]);
}

#[test]
fn step_keeps_the_range_of_its_trap() {
	let mut etable = Etable::core();
	etable[Opcode::CALL.as_usize()] = eval_call_create_trap as _;

	let mut interpreter = EtableInterpreter::new(setup(Opcode::CALL, true), &etable);
	assert!(matches!(interpreter.step(&mut ()), Err(Capture::Trap(_))));

	let shared = interpreter
		.as_mut()
		.shared_call_input()
		.expect("range kept");
	assert_eq!(&shared[..], &[1, 2, 3, 4][..]);
}

#[test]
fn step_clears_the_pending_range() {
	let mut machine = setup(Opcode::CALL, true);
	let trap = trap_call(&mut machine);
	assert!(machine.shared_call_input().is_some());
	drop(trap);

	let etable = Etable::core();
	let mut interpreter = EtableInterpreter::new(machine, &etable);
	interpreter.advance();
	assert!(matches!(interpreter.step(&mut ()), Err(Capture::Exit(_))));
	assert!(interpreter.as_mut().shared_call_input().is_none());
}

#[test]
fn trap_constructor_clears_the_pending_range() {
	let mut machine = setup(Opcode::CALL, true);
	let trap = trap_call(&mut machine);
	assert!(machine.shared_call_input().is_some());
	drop(trap);

	// The stack is empty, so this trap fails, but it clears the range first.
	assert!(CallTrap::new_from(CallScheme::Call, &mut machine).is_err());
	assert!(machine.shared_call_input().is_none());

	// The create traps clear it as well.
	pending_call_input(&mut machine);
	push_create_args(&mut machine, false);
	CreateTrap::new_create_from(&mut machine).unwrap();
	assert!(machine.shared_call_input().is_none());

	pending_call_input(&mut machine);
	push_create_args(&mut machine, true);
	CreateTrap::new_create2_from(&mut machine).unwrap();
	assert!(machine.shared_call_input().is_none());

	// So does the create feedback.
	pending_call_input(&mut machine);
	let etable = Etable::core();
	let mut interpreter = EtableInterpreter::new(machine, &etable);
	CreateFeedback {
		trap: CreateTrap {
			scheme: CreateScheme::Legacy {
				caller: H160::default(),
			},
			value: U256::default(),
			code: Vec::new(),
		},
		reason: Ok(H160::default()),
		retbuf: Vec::new(),
	}
	.to_machine(&mut interpreter)
	.unwrap();
	assert!(interpreter.as_mut().shared_call_input().is_none());
}

#[test]
#[allow(deprecated)]
fn empty_input_is_shared_as_an_empty_range() {
	let mut machine = machine_with(Opcode::CALL);
	machine.enable_call_input_sharing();
	machine.memory.set(0, &[1, 2, 3, 4], None).unwrap();
	let len = machine.memory.len();
	push_args(&mut machine, CallScheme::Call, 32, 0);

	let trap = trap_call(&mut machine);

	assert!(trap.input.is_empty());
	let shared = machine.shared_call_input().unwrap();
	assert!(shared.is_empty());
	assert_eq!(shared.len(), 0);
	assert_eq!(&shared[..], &[][..]);
	assert_eq!(machine.memory.len(), len);
	assert_eq!(machine.memory.data(), &vec![1, 2, 3, 4]);
}
