//! # The EVM64 feature
//!
//! See [EIP-7937](https://eips.ethereum.org/EIPS/eip-7937).

pub mod eval;
mod gasometer;

use evm::{
	interpreter::{
		etable::{DispatchEtable, MultiEfn, MultiEtable, Single},
		Opcode,
	},
	standard::GasometerState,
};

pub use crate::gasometer::eval as eval_gasometer;

pub const OPCODE_EVM64_MODE: Opcode = Opcode(0xc0);

/// Append a normal `(gasometer, runtime)` etable with EVM64 gasometer and
/// opcodes.
pub fn etable<S, H, Tr>(
	orig: (Single<S, H, Tr>, DispatchEtable<S, H, Tr>),
) -> (DispatchEtable<S, H, Tr>, MultiEtable<S, H, Tr>)
where
	S: AsRef<GasometerState> + AsMut<GasometerState>,
{
	let mut gasometer_etable = DispatchEtable::from(orig.0);
	let mut eval_etable = MultiEtable::from(orig.1);

	let mut mode_etable = DispatchEtable::none();
	mode_etable[Opcode::ADD.as_usize()] = eval::eval_add;
	mode_etable[Opcode::MUL.as_usize()] = eval::eval_mul;
	mode_etable[Opcode::SUB.as_usize()] = eval::eval_sub;
	mode_etable[Opcode::DIV.as_usize()] = eval::eval_div;
	mode_etable[Opcode::SDIV.as_usize()] = eval::eval_sdiv;
	mode_etable[Opcode::MOD.as_usize()] = eval::eval_mod;
	mode_etable[Opcode::SMOD.as_usize()] = eval::eval_smod;
	mode_etable[Opcode::ADDMOD.as_usize()] = eval::eval_addmod;
	mode_etable[Opcode::MULMOD.as_usize()] = eval::eval_mulmod;
	mode_etable[Opcode::EXP.as_usize()] = eval::eval_exp;
	mode_etable[Opcode::LT.as_usize()] = eval::eval_lt;
	mode_etable[Opcode::GT.as_usize()] = eval::eval_gt;
	mode_etable[Opcode::SLT.as_usize()] = eval::eval_slt;
	mode_etable[Opcode::SGT.as_usize()] = eval::eval_sgt;
	mode_etable[Opcode::EQ.as_usize()] = eval::eval_eq;
	mode_etable[Opcode::ISZERO.as_usize()] = eval::eval_iszero;
	mode_etable[Opcode::AND.as_usize()] = eval::eval_and;
	mode_etable[Opcode::OR.as_usize()] = eval::eval_or;
	mode_etable[Opcode::XOR.as_usize()] = eval::eval_xor;
	mode_etable[Opcode::NOT.as_usize()] = eval::eval_not;
	mode_etable[Opcode::SHL.as_usize()] = eval::eval_shl;
	mode_etable[Opcode::SHR.as_usize()] = eval::eval_shr;
	mode_etable[Opcode::SAR.as_usize()] = eval::eval_sar;
	mode_etable[Opcode::JUMP.as_usize()] = eval::eval_jump;
	mode_etable[Opcode::JUMPI.as_usize()] = eval::eval_jumpi;

	gasometer_etable[OPCODE_EVM64_MODE.as_usize()] = eval_gasometer;
	eval_etable[OPCODE_EVM64_MODE.as_usize()] = MultiEfn::Node(Box::new(mode_etable.into()));

	(gasometer_etable, eval_etable)
}
