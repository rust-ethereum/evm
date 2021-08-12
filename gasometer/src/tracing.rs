//! Allows to listen to gasometer events.

use super::Snapshot;
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

impl Snapshot {
	pub fn gas(&self) -> u64 {
		self.gas_limit - self.used_gas - self.memory_gas
	}
}

#[derive(Debug, Copy, Clone)]
pub enum Event {
	RecordCost {
		cost: u64,
		snapshot: Snapshot,
	},
	RecordRefund {
		refund: i64,
		snapshot: Snapshot,
	},
	RecordStipend {
		stipend: u64,
		snapshot: Snapshot,
	},
	RecordDynamicCost {
		gas_cost: u64,
		memory_gas: u64,
		gas_refund: i64,
		snapshot: Snapshot,
	},
	RecordTransaction {
		cost: u64,
		snapshot: Snapshot,
	},
}

impl Event {
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
