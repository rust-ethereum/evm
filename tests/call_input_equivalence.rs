//! Test that a callee reads the same input whether the caller shares it or
//! copies it.

#[allow(dead_code)]
mod mock;
mod support;

use support::*;

/// Address of the contract that calls at the call stack limit.
const DEEP: u64 = 0x0e00;
/// The bytes that the test precompile returns.
const PRECOMPILE_RESPONSE: [u8; 4] = [0x12, 0x34, 0x56, 0x78];

/// A run of the equivalence test.
struct Case {
	/// Name of the case.
	name: &'static str,
	/// The accounts of the backend.
	accounts: Vec<(u64, Vec<u8>)>,
	/// The address that the transaction calls.
	entry: u64,
	/// The bytes that the transaction must return.
	expected: Vec<u8>,
	/// The inputs that the precompile must receive.
	precompile_inputs: Vec<Vec<u8>>,
	/// Whether the run needs a large stack.
	deep: bool,
}

/// The cases of the test.
fn cases() -> Vec<Case> {
	let mut cases = Vec::new();

	// A range that the caller wrote completely.
	let caller = code(&[
		&store_word(0x11, 0),
		&store_word(0x22, 32),
		&CallOp::static_call(ECHO, (0, 64), (0x100, 64)).hex_and_pop(),
		&return_range(0x100, 64),
	]);
	cases.push(Case {
		name: "fully written range",
		accounts: vec![(TOP, caller), (ECHO, echo_contract())],
		entry: TOP,
		expected: words(&[0x11, 0x22]),
		precompile_inputs: Vec::new(),
		deep: false,
	});

	// A range that the caller wrote partly, with a zero tail.
	let caller = code(&[
		&store_word(0x33, 0),
		&CallOp::static_call(ECHO, (0, 64), (0x100, 64)).hex_and_pop(),
		&return_range(0x100, 64),
	]);
	cases.push(Case {
		name: "partly written range with a zero tail",
		accounts: vec![(TOP, caller), (ECHO, echo_contract())],
		entry: TOP,
		expected: words(&[0x33, 0]),
		precompile_inputs: Vec::new(),
		deep: false,
	});

	// A range that lies entirely beyond the memory that the caller wrote.
	let caller = code(&[
		&store_word(0x44, 0),
		&CallOp::static_call(ECHO, (0x1000, 32), (0x100, 32)).hex_and_pop(),
		&return_range(0x100, 32),
	]);
	cases.push(Case {
		name: "range beyond the physical memory",
		accounts: vec![(TOP, caller), (ECHO, echo_contract())],
		entry: TOP,
		expected: words(&[0]),
		precompile_inputs: Vec::new(),
		deep: false,
	});

	// A zero-length input at an offset far beyond the memory.
	let caller = code(&[
		&store_word(0x55, 0),
		&CallOp::static_call(ECHO, (1 << 40, 0), (0x100, 32)).hex_and_pop(),
		&return_range(0x100, 32),
	]);
	cases.push(Case {
		name: "zero-length input at a huge offset",
		accounts: vec![(TOP, caller), (ECHO, echo_contract())],
		entry: TOP,
		expected: words(&[0]),
		precompile_inputs: Vec::new(),
		deep: false,
	});

	// An output range that is the input range, and a second call that must see
	// the bytes of the first one.
	let caller = code(&[
		&store_word(0x55, 0),
		&CallOp::static_call(RETURN_WORD, (0, 32), (0, 32)).hex_and_pop(),
		&CallOp::static_call(ECHO, (0, 32), (0x100, 32)).hex_and_pop(),
		&return_range(0x100, 32),
	]);
	cases.push(Case {
		name: "output range equal to the input range",
		accounts: vec![
			(TOP, caller),
			(ECHO, echo_contract()),
			(RETURN_WORD, return_word_contract()),
		],
		entry: TOP,
		expected: words(&[WORD]),
		precompile_inputs: Vec::new(),
		deep: false,
	});

	// A callee that forwards its own input to another callee.
	let caller = code(&[
		&store_word(0x11, 0),
		&store_word(0x22, 32),
		&CallOp::static_call(FORWARD, (0, 64), (0x100, 64)).hex_and_pop(),
		&return_range(0x100, 64),
	]);
	cases.push(Case {
		name: "nested call that forwards its input",
		accounts: vec![
			(TOP, caller),
			(ECHO, echo_contract()),
			(FORWARD, forwarding_contract(ECHO)),
		],
		entry: TOP,
		expected: words(&[0x11, 0x22]),
		precompile_inputs: Vec::new(),
		deep: false,
	});

	// A revert with return data into a range that overlaps the input.
	let caller = code(&[
		&store_word(0x66, 0),
		&CallOp::static_call(REVERT_WORD, (0, 32), (0, 32)).hex_and_pop(),
		&return_range(0, 32),
	]);
	cases.push(Case {
		name: "revert into a range that overlaps the input",
		accounts: vec![(TOP, caller), (REVERT_WORD, revert_word_contract())],
		entry: TOP,
		expected: words(&[WORD]),
		precompile_inputs: Vec::new(),
		deep: false,
	});

	// A call that carries more value than the caller has, after which the
	// caller's memory is intact and writable.
	let caller = code(&[
		&store_word(0x77, 0),
		&CallOp::call_with_value(VALUE_TARGET, (0, 32), (0, 32), VALUE).hex_and_pop(),
		&store_word(0x88, 32),
		&return_range(0, 64),
	]);
	cases.push(Case {
		name: "call with more value than the caller has",
		accounts: vec![(TOP, caller)],
		entry: TOP,
		expected: words(&[0x77, 0x88]),
		precompile_inputs: Vec::new(),
		deep: false,
	});

	// The same, but at the call stack limit.
	let deep_caller = code(&[
		&store_word(0x77, 0),
		&CallOp::static_call(NO_CODE, (0, 32), (0, 32)).hex_and_pop(),
		&store_word(0x88, 32),
		&return_range(0, 64),
	]);
	let (entry, mut accounts) = chain(DEEP);
	accounts.push((DEEP, deep_caller));
	cases.push(Case {
		name: "call at the call stack limit",
		accounts,
		entry,
		expected: words(&[0x77, 0x88]),
		precompile_inputs: Vec::new(),
		deep: true,
	});

	// Every call opcode.
	for (name, call) in [
		("CALL", CallOp::call(ECHO, (0, 32), (0x100, 32))),
		("CALLCODE", CallOp::call_code(ECHO, (0, 32), (0x100, 32))),
		(
			"DELEGATECALL",
			CallOp::delegate_call(ECHO, (0, 32), (0x100, 32)),
		),
		(
			"STATICCALL",
			CallOp::static_call(ECHO, (0, 32), (0x100, 32)),
		),
	] {
		let caller = code(&[
			&store_word(0x99, 0),
			&call.hex_and_pop(),
			&return_range(0x100, 32),
		]);
		cases.push(Case {
			name,
			accounts: vec![(TOP, caller), (ECHO, echo_contract())],
			entry: TOP,
			expected: words(&[0x99]),
			precompile_inputs: Vec::new(),
			deep: false,
		});
	}

	// A precompile that receives the input.
	let caller = code(&[
		&store_word(0x11, 0),
		&store_word(0x22, 32),
		&CallOp::static_call(PRECOMPILE, (0, 64), (0x100, 4)).hex_and_pop(),
		&return_range(0x100, 4),
	]);
	cases.push(Case {
		name: "precompile",
		accounts: vec![(TOP, caller)],
		entry: TOP,
		expected: PRECOMPILE_RESPONSE.to_vec(),
		precompile_inputs: vec![words(&[0x11, 0x22])],
		deep: false,
	});

	cases
}

#[test]
fn the_callee_reads_the_same_input() {
	for case in cases() {
		for path in Path::ALL {
			let precompiles = TestPrecompiles::new(PRECOMPILE_RESPONSE.to_vec());

			let accounts = case.accounts.clone();
			let entry = case.entry;
			let run = move || run_path(accounts, entry, path, precompiles);
			let (result, precompiles) = if case.deep {
				on_a_large_stack(run)
			} else {
				run()
			};

			let value = result.unwrap_or_else(|err| {
				panic!(
					"{} / {}: the run failed with {err:?}",
					case.name,
					path.name()
				)
			});

			assert_eq!(
				retval(&value),
				&case.expected[..],
				"{} / {}",
				case.name,
				path.name()
			);
			assert_eq!(
				precompiles.received(),
				case.precompile_inputs,
				"{} / {}",
				case.name,
				path.name()
			);
		}
	}
}
