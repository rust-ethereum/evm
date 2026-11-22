//! Test that a call reads its input from the caller's memory instead of
//! copying it, by measuring what it allocates.
//!
//! The binary holds one test, and a counting global allocator, so that the
//! counts belong to that test alone.

#[allow(dead_code)]
mod mock;
mod support;

use std::{
	alloc::{GlobalAlloc, Layout, System},
	sync::atomic::{AtomicU64, Ordering},
};

use evm::{
	backend::OverlayedBackend,
	interpreter::ExitError,
	standard::{Config, Invoker, TransactValue},
};
use support::*;

/// An allocator that counts the bytes that it hands out.
struct Counting;

/// Bytes handed out since the test started.
static ALLOCATED: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for Counting {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		ALLOCATED.fetch_add(layout.size() as u64, Ordering::Relaxed);
		unsafe { System.alloc(layout) }
	}

	unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
		ALLOCATED.fetch_add(layout.size() as u64, Ordering::Relaxed);
		unsafe { System.alloc_zeroed(layout) }
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		unsafe { System.dealloc(ptr, layout) }
	}

	unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
		ALLOCATED.fetch_add(new_size as u64, Ordering::Relaxed);
		unsafe { System.realloc(ptr, layout, new_size) }
	}
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// One callee of the allocation test, and what the loop must observe after the
/// last of its calls.
#[derive(Clone)]
struct Scenario {
	/// Name of the scenario.
	name: &'static str,
	/// The callee's address.
	callee: u64,
	/// The callee's code, if it has any.
	callee_code: Option<Vec<u8>>,
	/// Whether the loop calls with a value.
	with_value: bool,
	/// Whether the loop runs at the call stack limit.
	deep: bool,
	/// Whether the calls succeed.
	succeed: bool,
	/// The word that the callee leaves in the loop's memory, if it returns one.
	word: Option<Vec<u8>>,
}

impl Scenario {
	/// The address that the transaction calls.
	const fn entry(&self) -> u64 {
		if self.deep { CHAIN_BASE } else { TOP }
	}

	/// The accounts of the backend, besides the loop and the top-level
	/// contract.
	fn accounts(&self, in_len: usize) -> Vec<(u64, Vec<u8>)> {
		let mut accounts = Vec::new();

		if self.deep {
			// A chain of contracts, each of which calls the next one, so that
			// the loop runs at the call stack limit.
			for index in 0..CHAIN_LEN {
				let next = if index + 1 < CHAIN_LEN {
					CHAIN_BASE + index as u64 + 1
				} else {
					LOOP
				};
				accounts.push((CHAIN_BASE + index as u64, chain_contract(next)));
			}
		} else {
			accounts.push((TOP, top_level(LOOP)));
		}

		accounts.push((
			LOOP,
			if self.with_value {
				looping_contract_with_value(in_len, self.callee)
			} else {
				looping_contract(in_len, self.callee)
			},
		));

		if let Some(code) = &self.callee_code {
			accounts.push((self.callee, code.clone()));
		}

		accounts
	}
}

/// The word that the test precompile returns.
const PRECOMPILE_WORD: u8 = 0xcd;

/// The scenarios of the test.
fn scenarios() -> Vec<Scenario> {
	vec![
		Scenario {
			name: "no code",
			callee: NO_CODE,
			callee_code: None,
			with_value: false,
			deep: false,
			succeed: true,
			word: None,
		},
		Scenario {
			name: "returns 32 bytes into the input range",
			callee: RETURN_WORD,
			callee_code: Some(return_word_contract()),
			with_value: false,
			deep: false,
			succeed: true,
			word: Some(words(&[WORD])),
		},
		Scenario {
			name: "reverts with 32 bytes into the input range",
			callee: REVERT_WORD,
			callee_code: Some(revert_word_contract()),
			with_value: false,
			deep: false,
			succeed: false,
			word: Some(words(&[WORD])),
		},
		Scenario {
			name: "value above the caller's balance",
			callee: VALUE_TARGET,
			callee_code: None,
			with_value: true,
			deep: false,
			succeed: false,
			word: None,
		},
		Scenario {
			name: "precompile",
			callee: PRECOMPILE,
			callee_code: None,
			with_value: false,
			deep: false,
			succeed: true,
			word: Some(words(&[PRECOMPILE_WORD])),
		},
		Scenario {
			name: "at the call stack limit",
			callee: NO_CODE,
			callee_code: None,
			with_value: false,
			deep: true,
			succeed: false,
			word: None,
		},
	]
}

/// Run the scenario with the given input length, and return the number of
/// bytes that the run allocated.
fn measure(
	scenario: &Scenario,
	path: Path,
	in_len: usize,
) -> (Result<TransactValue, ExitError>, u64) {
	let accounts = scenario.accounts(in_len);
	let entry = scenario.entry();
	let deep = scenario.deep;

	let before = ALLOCATED.load(Ordering::Relaxed);
	let run = move || {
		run_path(
			accounts,
			entry,
			path,
			TestPrecompiles::lengths_only(vec![PRECOMPILE_WORD; 32]),
		)
	};
	let (result, precompiles) = if deep {
		// A recursion needs one stack frame of this process for each call.
		on_a_large_stack(run)
	} else {
		run()
	};
	let after = ALLOCATED.load(Ordering::Relaxed);

	// The precompile receives the whole input range, or none of it in a
	// control run.
	if scenario.callee == PRECOMPILE {
		let expected: Vec<usize> = std::iter::repeat_n(in_len, CALL_COUNT).collect();
		assert_eq!(precompiles.lengths(), expected, "{}", scenario.name);
	}

	(result, after - before)
}

/// Run the scenario through an interpreter that wraps the standard one and
/// forwards its machine, and return the number of bytes that the run allocated.
fn measure_wrapped(
	scenario: &Scenario,
	path: Path,
	in_len: usize,
) -> (Result<TransactValue, ExitError>, u64) {
	let accounts = scenario.accounts(in_len);
	let entry = scenario.entry();

	let before = ALLOCATED.load(Ordering::Relaxed);
	let result = {
		let config = Config::cancun();
		let etable = support::etable();
		let resolver = support::resolver(&etable, &());
		let wrapped = support::wrap::WrappedResolver::<(), _, true>::new(&resolver);
		let invoker = Invoker::new(&wrapped);
		let mut backend = OverlayedBackend::new(support::backend(&accounts), &config.runtime);
		let args = support::call_args(&config, entry, Vec::new());

		match path {
			Path::Recursive => support::transact_with_invoker(&invoker, None, &mut backend, args),
			Path::Hybrid => support::transact_with_invoker(&invoker, Some(1), &mut backend, args),
			Path::Heap => support::heap_transact_with_invoker(&invoker, &mut backend, args),
		}
	};
	let after = ALLOCATED.load(Ordering::Relaxed);

	(result, after - before)
}

/// The status and the returned word of a run.
fn result_words(value: &TransactValue) -> (&[u8], &[u8]) {
	let retval = retval(value);

	assert_eq!(retval.len(), 64, "the loop returns two words");

	(&retval[0..32], &retval[32..64])
}

/// Assert the results of a run with the input and of its control, and that the
/// run with the input did not allocate a copy of it.
fn check(
	scenario: &Scenario,
	path: Path,
	share: (Result<TransactValue, ExitError>, u64),
	control: (Result<TransactValue, ExitError>, u64),
) {
	let (share, share_bytes) = share;
	let (control, control_bytes) = control;

	let share = share.unwrap_or_else(|err| {
		panic!(
			"{} / {}: the run failed with {err:?}",
			scenario.name,
			path.name()
		)
	});
	let control = control.unwrap_or_else(|err| {
		panic!(
			"{} / {}: the control run failed with {err:?}",
			scenario.name,
			path.name()
		)
	});

	let (share_status, share_word) = result_words(&share);
	let (control_status, control_word) = result_words(&control);

	let expected_status = word(u64::from(scenario.succeed));
	assert_eq!(
		share_status,
		&expected_status[..],
		"{} / {}",
		scenario.name,
		path.name()
	);
	assert_eq!(
		control_status,
		&expected_status[..],
		"{} / {}",
		scenario.name,
		path.name()
	);
	if let Some(word) = &scenario.word {
		assert_eq!(share_word, &word[..], "{} / {}", scenario.name, path.name());
	}
	assert_eq!(
		share_word,
		control_word,
		"{} / {}: the shared run returned other bytes than its control",
		scenario.name,
		path.name()
	);

	let difference = share_bytes as i128 - control_bytes as i128;

	println!(
		"{} / {}: {} bytes with the input, {} bytes without it, difference {} bytes",
		scenario.name,
		path.name(),
		share_bytes,
		control_bytes,
		difference
	);

	assert!(
		difference < 2 * MEMORY_LEN as i128,
		"{} / {}: the run with the input allocated {difference} bytes more than its control",
		scenario.name,
		path.name()
	);
}

#[test]
fn call_input_is_not_copied() {
	let scenarios = scenarios();

	for scenario in &scenarios {
		for path in Path::ALL {
			check(
				scenario,
				path,
				measure(scenario, path, MEMORY_LEN),
				measure(scenario, path, 0),
			);
		}
	}

	// The same bound holds through an interpreter that wraps the standard one
	// and forwards its machine, for a callee without code.
	let wrapped = &Scenario {
		name: "no code, through a wrapped interpreter",
		..scenarios[0].clone()
	};

	for path in Path::ALL {
		check(
			wrapped,
			path,
			measure_wrapped(wrapped, path, MEMORY_LEN),
			measure_wrapped(wrapped, path, 0),
		);
	}
}
