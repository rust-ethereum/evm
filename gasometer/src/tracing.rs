//! Allows to listen to gasometer events.

use super::Snapshot;

environmental::environmental!(listener: dyn EventListener + 'static);

#[cfg(feature = "std")]
std::thread_local! {
	static ENABLE_TRACING: AtomicBool = AtomicBool::new(false);
}

#[cfg(not(feature = "std"))]
static ENABLE_TRACING: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "std")]
pub fn enable_tracing(enable: bool) {
	ENABLE_TRACING.with(|s| s.store(enable, Ordering::Relaxed));
}

#[cfg(not(feature = "std"))]
pub fn enable_tracing(enable: bool) {
	ENABLE_TRACING.store(enable, Ordering::Relaxed);
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
			if s.load(Ordering::Relaxed) {
				listener::with(|listener| listener.event(self));
			}
		})
	}

	#[cfg(not(feature = "std"))]
	pub(crate) fn emit(self) {
		if ENABLE_TRACING.load(Ordering::Relaxed) {
			listener::with(|listener| listener.event(self));
		}
	}
}

/// Run closure with provided listener.
pub fn using<R, F: FnOnce() -> R>(new: &mut (dyn EventListener + 'static), f: F) -> R {
	listener::using(new, f)
}
