//! Allows to listen to gasometer events.

use super::Snapshot;

environmental::environmental!(listener: dyn EventListener + 'static);

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

// Expose `listener::with` to the crate only.
pub(crate) fn with<F: FnOnce(&mut (dyn EventListener + 'static))>(
	f: F
) {
	listener::with(f);
}

/// Run closure with provided listener.
pub fn using<R, F: FnOnce() -> R>(new: &mut (dyn EventListener + 'static), f: F) -> R {
	listener::using(new, f)
}
