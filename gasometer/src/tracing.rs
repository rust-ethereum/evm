//! Allows to listen to gasometer events.

#[cfg(feature = "tracing")]
pub(crate) mod inner {
    environmental::environmental!(hook: dyn EventListener + 'static);

    pub trait EventListener {
        fn record_cost(
            &mut self,
            _cost: u64
        ) { }

        fn record_refund(
            &mut self,
            _refund: i64
        ) { }

        fn record_stipend(
            &mut self,
            _stipend: u64
        ) { }

        fn record_dynamic_cost(
            &mut self, 
            _gas_cost: u64,
            _memory_gas: u64,
            _gas_refund: i64
        ) { }

        fn record_transaction(
            &mut self,
            _transaction_cost: u64
        ) { }
    }

    /// Run closure with provided listener.
    pub fn using<R, F: FnOnce() -> R>(
        listener: &mut (dyn EventListener + 'static),
        f: F
    ) -> R {
        hook::using(listener, f)
    }

    pub(crate) fn with<F: FnOnce(&mut (dyn EventListener + 'static))>(
        f: F
    ) {
        hook::with(f);
    }
}

#[cfg(feature = "tracing")]
pub use inner::using;

#[cfg(not(feature = "tracing"))]
macro_rules! event {
    ($func_name: ident ( $($arg:expr),* )) => { }
}

#[cfg(feature = "tracing")]
macro_rules! event {
    ($method: ident ( $($arg:expr),* )) => {
        $crate::tracing::inner::with(|hook| hook.$method( $($arg),* ));
    }
}