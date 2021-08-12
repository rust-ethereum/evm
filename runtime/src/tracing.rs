//! Allows to listen to runtime events.

use crate::{Capture, Context, ExitReason, Memory, Opcode, Stack, Trap};
use primitive_types::{H160, H256};
use core::cell::RefCell;

environmental::environmental!(listener: dyn EventListener + 'static);

#[cfg(feature = "std")]
std::thread_local! {
	static ENABLE_TRACING: RefCell<bool> = RefCell::new(false);
}

// We assume wasm is not multi-threaded.
// This is the same assumption as the environmental crate.
#[cfg(not(feature = "std"))]
static ENABLE_TRACING: RefCell<bool> = RefCell::new(false);

#[cfg(feature = "std")]
pub fn enable_tracing(enable: bool) {
	ENABLE_TRACING.with(|s| s.replace(enable));
}

#[cfg(not(feature = "std"))]
pub fn enable_tracing(enable: bool) {
	ENABLE_TRACING.replace(enable);
}

pub trait EventListener {
	fn event(&mut self, event: Event);
}

#[derive(Debug, Copy, Clone)]
pub enum Event<'a> {
	Step {
		context: &'a Context,
		opcode: Opcode,
		position: &'a Result<usize, ExitReason>,
		stack: &'a Stack,
		memory: &'a Memory,
	},
	StepResult {
		result: &'a Result<(), Capture<ExitReason, Trap>>,
		return_value: &'a [u8],
	},
	SLoad {
		address: H160,
		index: H256,
		value: H256,
	},
	SStore {
		address: H160,
		index: H256,
		value: H256,
	},
}

impl<'a> Event<'a> {
	#[cfg(feature = "std")]
	pub(crate) fn emit(self) {
		ENABLE_TRACING.with(|s| {
			if *s.borrow() {
				listener::with(|listener| listener.event(self));
			}
		})
	}

	#[cfg(not(feature = "std"))]
	pub(crate) fn emit(self) {
		if *ENABLE_TRACING.borrow() {
			listener::with(|listener| listener.event(self));
		}
	}
}

/// Run closure with provided listener.
pub fn using<R, F: FnOnce() -> R>(new: &mut (dyn EventListener + 'static), f: F) -> R {
	listener::using(new, f)
}
