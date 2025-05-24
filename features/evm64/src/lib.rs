//! # The EVM64 feature
//!
//! See [EIP-7937](https://eips.ethereum.org/EIPS/eip-7937).

pub mod eval;
mod gasometer;

pub use crate::gasometer::eval as eval_gasometer;

use evm::{
	interpreter::{
		etable::{Etable, MultiEfn, MultiEtable, Single},
		opcode::Opcode,
	},
	standard::GasometerState,
};

pub const OPCODE_EVM64_MODE: Opcode = Opcode(0xc0);

/// Append a normal `(gasometer, runtime)` etable with EVM64 gasometer and
/// opcodes.
pub fn etable<'config, S, H, Tr>(
	orig: (Single<S, H, Tr>, Etable<S, H, Tr>),
) -> (Etable<S, H, Tr>, MultiEtable<S, H, Tr>)
where
	S: AsRef<GasometerState<'config>> + AsMut<GasometerState<'config>>,
{
	let mut gasometer_etable = Etable::from(orig.0);
	let mut eval_etable = MultiEtable::from(orig.1);

	let mut mode_etable = Etable::none();
	mode_etable[Opcode::ADD.as_usize()] = eval::eval_add;

	gasometer_etable[OPCODE_EVM64_MODE.as_usize()] = eval_gasometer;
	eval_etable[OPCODE_EVM64_MODE.as_usize()] = MultiEfn::Node(Box::new(mode_etable.into()));

	(gasometer_etable, eval_etable)
}
