//! Allows to listen to runtime events.

use crate::{Context};
use evm_runtime::{CreateScheme, Transfer};
use primitive_types::{H160, U256};

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
		scheme: CreateScheme,
		value: U256,
		init_code: &'a [u8],
		target_gas: Option<u64>,
    }
}

impl<'a> Event<'a> {
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