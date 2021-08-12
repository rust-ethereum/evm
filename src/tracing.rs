//! Allows to listen to runtime events.

use crate::Context;
use core::cell::RefCell;
use evm_runtime::{CreateScheme, Transfer};
use primitive_types::{H160, U256};

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
	Call {
		code_address: H160,
		transfer: &'a Option<Transfer>,
		input: &'a [u8],
		target_gas: Option<u64>,
		is_static: bool,
		context: &'a Context,
	},
	Create {
		caller: H160,
		address: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: &'a [u8],
		target_gas: Option<u64>,
	},
	Suicide {
		address: H160,
		target: H160,
		balance: U256,
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
