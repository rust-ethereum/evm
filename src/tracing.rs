//! Allows to listen to runtime events.

use crate::Context;
use core::cell::RefCell;
use evm_runtime::{CreateScheme, Transfer, ExitReason};
use primitive_types::{H160, H256, U256};

environmental::environmental!(listener: dyn EventListener + 'static);

#[cfg(feature = "std")]
std::thread_local! {
	static ENABLE_TRACING: RefCell<bool> = RefCell::new(false);
}

// We assume wasm is not multi-threaded.
// This is the same assumption as the environmental crate.
#[cfg(not(feature = "std"))]
struct WasmCell(RefCell<bool>);

#[cfg(not(feature = "std"))]
unsafe impl Sync for WasmCell {}

#[cfg(not(feature = "std"))]
static ENABLE_TRACING: WasmCell = WasmCell(RefCell::new(false));

#[cfg(feature = "std")]
pub fn enable_tracing(enable: bool) {
	ENABLE_TRACING.with(|s| s.replace(enable));
}

#[cfg(not(feature = "std"))]
pub fn enable_tracing(enable: bool) {
	ENABLE_TRACING.0.replace(enable);
}

#[cfg(feature = "std")]
pub fn is_tracing_enabled() -> bool {
	ENABLE_TRACING.with(|s| *s.borrow())
}

#[cfg(not(feature = "std"))]
pub fn is_tracing_enabled() -> bool {
	*ENABLE_TRACING.0.borrow()
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
	Exit {
		reason: &'a ExitReason,
		return_value: &'a [u8],
	},
	TransactCall {
		caller: H160,
		address: H160,
		value: U256,
		data: &'a [u8],
		gas_limit: u64,
	},
	TransactCreate {
		caller: H160,
		value: U256,
		init_code: &'a [u8],
		gas_limit: u64,
		address: H160,
	},
	TransactCreate2 {
		caller: H160,
		value: U256,
		init_code: &'a [u8],
		salt: H256,
		gas_limit: u64,
		address: H160,
	}
}

impl<'a> Event<'a> {
	#[cfg(feature = "std")]
	pub(crate) fn emit(self) {
		listener::with(|listener| listener.event(self));
	}

	#[cfg(not(feature = "std"))]
	pub(crate) fn emit(self) {
		listener::with(|listener| listener.event(self));
	}
}

/// Run closure with provided listener.
pub fn using<R, F: FnOnce() -> R>(new: &mut (dyn EventListener + 'static), f: F) -> R {
	listener::using(new, f)
}
