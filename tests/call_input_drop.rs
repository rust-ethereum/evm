//! Test that a heap call stack dropped inside a call whose input the callee
//! shares stops cleanly: the caller's trap carries no input of its own, the
//! drop does not copy the caller's memory, and it unwinds the call stack and
//! finalizes the transaction.
//!
//! The binary holds one test, and a counting global allocator, so that the
//! count belongs to that test alone.

#[allow(dead_code)]
mod mock;
mod support;

use std::{
	alloc::{GlobalAlloc, Layout, System},
	sync::atomic::{AtomicU64, Ordering},
};

use evm::{
	HeapTransact,
	backend::OverlayedBackend,
	standard::{Config, Invoker as StandardInvoker},
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

/// The length of the input of the call.
const INPUT_LEN: usize = 4096;

#[test]
fn dropping_a_heap_stack_in_a_call() {
	let config = Config::cancun();
	let etable = support::etable();
	let resolver = support::resolver(&etable, &());
	let standard = StandardInvoker::new(&resolver);
	let invoker = RecordingInvoker::new(&standard, true);

	// The caller writes the last word of its memory, then calls the contract
	// that returns its calldata, with that memory as the input.
	let caller = support::code(&[
		&store_word(0x11, INPUT_LEN - 32),
		&CallOp::call(ECHO, (0, INPUT_LEN), (0, 32)).hex_and_pop(),
		STOP,
	]);
	let echo = echo_contract();

	let accounts = vec![(TOP, caller), (ECHO, echo.clone())];
	let mut backend = OverlayedBackend::new(support::backend(&accounts), &config.runtime);
	let args = support::call_args(&config, TOP, Vec::new());

	let mut transact =
		HeapTransact::new(args, &invoker, &mut backend).expect("the heap stack is created");

	let mut entered = false;
	for _ in 0..16 {
		match transact.step_run() {
			Ok(()) => (),
			Err(_) => break,
		}

		if let Some(interpreter) = transact.last_interpreter()
			&& interpreter.code() == &echo[..]
		{
			entered = true;
			break;
		}
	}

	assert!(entered, "the heap stack enters the callee");
	assert!(
		transact.last_interpreter().is_some(),
		"the callee is suspended"
	);

	// The caller's trap carries no input of its own, so the callee reads the
	// caller's memory.
	assert_eq!(
		invoker.seen(),
		vec![Vec::new()],
		"the caller's trap leaves the input to the caller's memory"
	);

	// Dropping the heap stack lets the caller take its memory back, which
	// copies nothing while the callee holds no share.
	let before = ALLOCATED.load(Ordering::Relaxed);
	drop(transact);
	let after = ALLOCATED.load(Ordering::Relaxed);
	let allocated = after - before;

	println!("dropping the heap stack allocated {allocated} bytes");

	assert!(
		allocated < INPUT_LEN as u64,
		"dropping the heap stack allocated {allocated} bytes"
	);

	// The drop unwinds the call stack through the invoker, and finalizes the
	// transaction as unfinished.
	assert_eq!(invoker.exits(), 1, "the drop exits the caller's sub-layer");
	assert!(invoker.finalized(), "the drop finalizes the transaction");

	// The backend is still usable after the drop.
	let data = vec![1u8, 2, 3, 4];
	let args = support::call_args(&config, ECHO, data.clone());
	let value = support::transact_recursive(&etable, &mut backend, args)
		.expect("the next transaction runs");

	assert_eq!(support::retval(&value), &data[..]);
}
