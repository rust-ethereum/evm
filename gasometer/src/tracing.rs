//! Allows to listen to gasometer events.

#[cfg(feature = "tracing")]
environmental::environmental!(listener: dyn EventListener + 'static);

#[cfg(feature = "tracing")]
pub trait EventListener {
    fn event(
        &mut self,
        event: Event
    );
}

#[derive(Debug, Copy, Clone)]
pub enum Event {
    RecordCost(u64),
    RecordRefund(i64),
    RecordStipend(u64),
    RecordDynamicCost {
        gas_cost: u64,
        memory_gas: u64,
        gas_refund: i64,
    },
    RecordTransaction(u64),
}

impl Event {
    #[cfg(feature = "tracing")]
    pub(crate) fn emit(self) {
        listener::with(|listener| listener.event(self));
    }

    #[cfg(not(feature = "tracing"))]
    pub(crate) fn emit(self) {
        // no op.
    }
}

/// Run closure with provided listener.
#[cfg(feature = "tracing")]
pub fn using<R, F: FnOnce() -> R>(
    new: &mut (dyn EventListener + 'static),
    f: F
) -> R {
    listener::using(new, f)
}