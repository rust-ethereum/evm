//! Allows to listen to runtime events.

use crate::{Capture, Context, ExitReason, Memory, Opcode, Stack, Trap};
use primitive_types::{H160, H256, U256};

environmental::environmental!(listener: dyn EventListener + 'static);

pub trait EventListener {
	fn event(&mut self, event: Event);
}

#[derive(Debug, Copy, Clone)]
pub enum Event<'a> {
	Step {
		context: &'a Context,
		opcode: Opcode,
		position: &'a Result<usize, ExitReason>,
		stack: &'a Stack,
		memory: &'a Memory,
	},
	StepResult {
		result: &'a Result<(), Capture<ExitReason, Trap>>,
		return_value: &'a [u8],
	},
	SLoad {
		address: H160,
		index: H256,
		value: H256,
	},
	SStore {
		address: H160,
		index: H256,
		value: H256,
	},
	TransactSuicide {
		address: H160,
		target: H160,
		balance: U256,
	},
	TransactTransfer {
		call_type: &'a String,
		address: H160,
		target: H160,
		balance: U256,
		input: &'a String,
	},
	TransactCreate {
		call_type: &'a String,
		address: H160,
		target: H160,
		balance: U256,
		is_create2: bool,
		input: &'a String,
	},
}

// Expose `listener::with` to the crate only.
pub(crate) fn with<F: FnOnce(&mut (dyn EventListener + 'static))>(f: F) {
	listener::with(f);
}

/// Run closure with provided listener.
pub fn using<R, F: FnOnce() -> R>(new: &mut (dyn EventListener + 'static), f: F) -> R {
	listener::using(new, f)
}
