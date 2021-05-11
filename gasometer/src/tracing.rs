//! Allow to listen to gasometer events. 
//! Enable `tracing` feature for events to be emitted.

#[cfg(feature = "tracing")]
environmental::environmental!(hook: dyn EventListener + 'static);

pub trait EventListener {
    fn accept(&mut self, event: Event);
}

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

#[cfg(feature = "tracing")]
pub(crate) fn emit<F>(f: F)
where
    F: FnOnce() -> Event
{
    hook::with(|hook| {
        hook.accept(f());
    });
}

#[cfg(not(feature = "tracing"))]
pub(crate) fn emit<F>(_f: F)
where
    F: FnOnce() -> Event
{
    // No-op.
}

/// Run closure with provided listener.
#[cfg(feature = "tracing")]
pub fn using<R, F: FnOnce() -> R>(
    listener: &mut (dyn EventListener + 'static),
    f: F
) -> R {
    hook::using(listener, f)
}