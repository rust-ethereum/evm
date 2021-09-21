//! Allows to listen to runtime events.

use crate::Context;
use evm_runtime::{CreateScheme, ExitReason, Transfer};
use primitive_types::{H160, H256, U256};

environmental::environmental!(listener: dyn EventListener + 'static);

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
