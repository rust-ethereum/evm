//! A call passes the input range of the caller's memory to the callee without
//! copying it.
//!
//! This is the only test in this binary, so that the counting allocator sees
//! only the allocations of this test.

mod common;

use common::*;
use evm::executor::stack::{PrecompileFailure, PrecompileFn, PrecompileOutput};
use evm::{Config, Context, ExitReason, ExitSucceed};
use primitive_types::H160;
use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The memory the looping contract physically writes, and the length of the
/// input range it passes to each callee.
const L: usize = 65536;
/// The number of calls the looping contract makes.
const N: usize = 16;

/// The call stack limit of `Config::cancun()`.
const CALL_STACK_LIMIT: usize = Config::cancun().call_stack_limit;

/// The address of the looping contract.
const LOOP_ADDR: H160 = address(0x07);
/// An address without code.
const EMPTY_ADDR: H160 = address(0x08);
/// An address holding the callee that returns 32 bytes.
const RETURNS_32_ADDR: H160 = address(0x09);
/// An address holding the callee that reverts with 32 bytes.
const REVERTS_32_ADDR: H160 = address(0x0a);
/// An address of the test precompile.
const PRECOMPILE_ADDR: H160 = address(0x0b);

/// CALL, which takes a value.
const CALL: u8 = 0xf1;
/// STATICCALL, which takes no value.
const STATICCALL: u8 = 0xfa;

/// The callee that returns 32 bytes into the output range of the call.
///
/// PUSH1 0x2a; PUSH1 0; MSTORE; PUSH1 32; PUSH1 0; RETURN
const RETURNS_32: &str = "602a60005260206000f3";

/// The callee that reverts with 32 bytes into the output range of the call,
/// which overlaps the input range.
///
/// PUSH1 0x2a; PUSH1 0; MSTORE; PUSH1 32; PUSH1 0; REVERT
const REVERTS_32: &str = "602a60005260206000fd";

/// The bytes the counting allocator has seen allocated.
static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
/// The length of the input the test precompile last received.
static PRECOMPILE_INPUT_LEN: AtomicUsize = AtomicUsize::new(0);

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
		System.alloc(layout)
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		System.dealloc(ptr, layout)
	}

	unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
		ALLOCATED.fetch_add(new_size, Ordering::Relaxed);
		System.realloc(ptr, layout, new_size)
	}

	unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
		ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
		System.alloc_zeroed(layout)
	}
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// A precompile that records the length of the input it receives and returns
/// the start of that input: its first 32 bytes, zero-padded, which is all the
/// output range of the call holds. It does not copy the whole input, so that a
/// run measures the executor's share and not this precompile's copy.
fn returns_the_start_of_its_input(
	input: &[u8],
	_gas_limit: Option<u64>,
	_context: &Context,
	_is_static: bool,
) -> Result<(PrecompileOutput, u64), PrecompileFailure> {
	PRECOMPILE_INPUT_LEN.store(input.len(), Ordering::Relaxed);

	let mut output = vec![0u8; 32];
	let len = input.len().min(32);
	output[..len].copy_from_slice(&input[..len]);

	Ok((
		PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output,
		},
		0,
	))
}

/// Bytecode of the top-level contract. It calls the looping contract with an
/// empty input and a 32-byte output range, which the call writes with the
/// looping contract's return value, then stores its own call's success flag at
/// offset 32 and returns both words.
///
/// PUSH1 32 (out_len); PUSH1 0 (out_offset); PUSH1 0 (in_len);
/// PUSH1 0 (in_offset); PUSH1 0 (value); PUSH2 LOOP; GAS; CALL;
/// PUSH1 32; MSTORE;
/// PUSH1 64; PUSH1 0; RETURN
fn top_level() -> Vec<u8> {
	decode_hex(&format!(
		"6020600060006000600061{:04x}5af160205260406000f3",
		address_word(LOOP_ADDR)
	))
}

/// Bytecode of the looping contract. It writes one word at `L - 32`, so its
/// memory physically covers `[0, L)`, then repeats `N` times: a call of
/// `callee` with the input range `[0, in_len)` (empty for the control runs)
/// and the output range `[0, 32)`, which overlaps the input range; it adds the
/// call's success flag to a running sum, then writes a word at 0 into the
/// input range. It returns the sum as a 32-byte word.
///
/// PUSH1 0 (sum)
/// PUSH1 1; PUSH2 (L - 32); MSTORE
/// [PUSH1 32 (out_len); PUSH1 0 (out_offset); PUSH3 L (in_len) or PUSH1 0;
///  PUSH1 0 (in_offset); [PUSH1 value]; PUSH2 callee; GAS; <opcode>; ADD;
///  PUSH1 0x2a; PUSH1 0; MSTORE] * N
/// PUSH1 0; MSTORE
/// PUSH1 32; PUSH1 0; RETURN
fn looping(callee: H160, in_len: usize, opcode: u8, value: u64) -> Vec<u8> {
	let mut code = String::new();
	// PUSH1 0 (the sum of the success flags)
	code += "6000";
	// PUSH1 1; PUSH2 (L - 32); MSTORE
	code += &format!("600161{:04x}52", L - 32);
	for _ in 0..N {
		// PUSH1 32 (out_len); PUSH1 0 (out_offset)
		code += "60206000";
		// PUSH3 L (in_len), or PUSH1 0 for the control run
		if in_len == 0 {
			code += "6000";
		} else {
			code += &format!("62{in_len:06x}");
		}
		// PUSH1 0 (in_offset)
		code += "6000";
		// CALL and CALLCODE take a value
		if opcode == CALL || opcode == 0xf2 {
			code += &format!("60{value:02x}");
		}
		// PUSH2 callee; GAS; <opcode>; ADD
		code += &format!("61{:04x}5a{opcode:02x}01", address_word(callee));
		// PUSH1 0x2a; PUSH1 0; MSTORE
		code += "602a600052";
	}
	// PUSH1 0; MSTORE
	code += "600052";
	// PUSH1 32; PUSH1 0; RETURN
	code += "60206000f3";

	decode_hex(&code)
}

/// A callee the looping contract calls, and what a run of it expects.
struct Case {
	/// What the case is, for the messages.
	name: &'static str,
	/// The address the loop calls.
	callee: H160,
	/// The code at that address; unused for a precompile.
	code: &'static str,
	/// The opcode the loop uses for each call.
	opcode: u8,
	/// The value the loop sends with each call.
	value: u64,
	/// The call stack limit of the configuration.
	call_stack_limit: usize,
	/// Whether the callee is a precompile.
	precompile: bool,
	/// The sum of the success flags the loop returns.
	flags: u64,
}

impl Case {
	/// A case that calls `callee` with `STATICCALL` and no value.
	const fn static_call(name: &'static str, callee: H160, code: &'static str, flags: u64) -> Self {
		Self {
			name,
			callee,
			code,
			opcode: STATICCALL,
			value: 0,
			call_stack_limit: CALL_STACK_LIMIT,
			precompile: false,
			flags,
		}
	}
}

/// Run the loop once with the input range of `in_len` bytes: check the exit
/// reason, the output, the input the precompile received and the flags, and
/// return the bytes the run allocated.
fn measure(case: &Case, in_len: usize) -> usize {
	let mut config = Config::cancun();
	config.call_stack_limit = case.call_stack_limit;

	let mut accounts = state(&[
		(CONTRACT, top_level()),
		(
			LOOP_ADDR,
			looping(case.callee, in_len, case.opcode, case.value),
		),
	]);
	let mut precompiles: BTreeMap<H160, PrecompileFn> = BTreeMap::new();
	if case.precompile {
		precompiles.insert(case.callee, returns_the_start_of_its_input as PrecompileFn);
	} else {
		accounts.insert(case.callee, account(decode_hex(case.code)));
	}

	ALLOCATED.store(0, Ordering::Relaxed);
	let (reason, output) = run_with_precompiles(accounts, Vec::new(), &config, &precompiles);
	let allocated = ALLOCATED.load(Ordering::Relaxed);

	assert_eq!(
		reason,
		ExitReason::Succeed(ExitSucceed::Returned),
		"{}: the exit reason",
		case.name
	);

	// The loop's sum comes first, and the top-level contract's own flag says
	// that the loop ran to its end.
	let mut expected = word(case.flags);
	expected.extend_from_slice(&word(1));
	assert_eq!(
		output, expected,
		"{}: the flags the loop summed and the top-level call's flag",
		case.name
	);

	if case.precompile {
		assert_eq!(
			PRECOMPILE_INPUT_LEN.load(Ordering::Relaxed),
			in_len,
			"{}: the length of the input the precompile received",
			case.name
		);
	}

	allocated
}

#[test]
fn a_call_does_not_copy_its_input() {
	let cases = [
		Case::static_call("an address without code", EMPTY_ADDR, "", N as u64),
		Case::static_call(
			"a contract that returns 32 bytes",
			RETURNS_32_ADDR,
			RETURNS_32,
			N as u64,
		),
		Case::static_call(
			"a contract that reverts with 32 bytes",
			REVERTS_32_ADDR,
			REVERTS_32,
			0,
		),
		Case {
			// The loop runs at depth 1, so a limit of 1 rejects its calls.
			call_stack_limit: 1,
			..Case::static_call("a call past the call stack limit", EMPTY_ADDR, "", 0)
		},
		Case {
			// The looping contract holds no balance, so it cannot transfer.
			opcode: CALL,
			value: 1,
			..Case::static_call("a call that cannot pay", EMPTY_ADDR, "", 0)
		},
		Case {
			precompile: true,
			..Case::static_call(
				"a precompile that returns the start of its input",
				PRECOMPILE_ADDR,
				"",
				N as u64,
			)
		},
	];

	for case in &cases {
		// The run passes `L` bytes as the input of each of the `N` calls; the
		// control passes an empty input, and is otherwise identical.
		let run_bytes = measure(case, L);
		let control_bytes = measure(case, 0);
		let difference = run_bytes as i64 - control_bytes as i64;

		println!(
			"{}: the run allocated {run_bytes} bytes and its control {control_bytes} bytes, \
			 a difference of {difference} bytes for {N} calls of {L} input bytes, \
			 summing its flags to {}",
			case.name, case.flags
		);

		assert!(
			run_bytes >= L,
			"{}: the run allocated {run_bytes} bytes, less than the {L} bytes of its memory",
			case.name
		);
		assert!(
			difference < 2 * L as i64,
			"{}: the run allocated {run_bytes} bytes and its control {control_bytes} bytes, \
			 so the {N} calls of {L} input bytes added {difference} bytes, which is not below {}",
			case.name,
			2 * L
		);
	}
}
