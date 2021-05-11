//! Allows to listen to gasometer events.

#[cfg(feature = "tracing")]
environmental::environmental!(hook: dyn EventListener + 'static);

#[cfg(feature = "tracing")]
pub trait EventListener {
    fn record_cost(cost: u64) { }
    fn record_refund(refund: i64) { }
    fn record_stipend(stipend: u64) { }
    fn record_dynamic_cost(
        gas_cost: u64,
        memory_gas: u64,
        gas_refund: i64
    ) { }
    fn record_transaction(transaction_cost: u64) { }
}

/// Run closure with provided listener.
#[cfg(feature = "tracing")]
pub fn using<R, F: FnOnce() -> R>(
    listener: &mut (dyn EventListener + 'static),
    f: F
) -> R {
    hook::using(listener, f)
}

#[cfg(not(feature = "tracing"))]
macro_rules! event {
    ($func_name: ident ( $($arg:expr),* )) => { }
}

#[cfg(feature = "tracing")]
macro_rules! event {
    ($method: ident ( $($arg:expr),* )) => {
        hook::with(|hook| hook.$method( $($arg)* ));
    }
}