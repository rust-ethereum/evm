#[macro_use]
mod macros;
mod arithmetic;
mod bitwise;
mod misc;

use core::ops::{BitAnd, BitOr, BitXor};
use primitive_types::{H256, U256};
use crate::{ExitReason, ExitSucceed, ExitError, Machine, Opcode};

pub enum Control {
	Continue(usize),
	Exit(ExitReason),
	Jump(usize),
}

pub fn eval(state: &mut Machine, opcode: Opcode, position: usize) -> Control {
	match opcode {
		Opcode::Stop => Control::Exit(ExitSucceed::Stopped.into()),
		Opcode::Add => op2_u256_tuple!(state, overflowing_add),
		Opcode::Mul => op2_u256_tuple!(state, overflowing_mul),
		Opcode::Sub => op2_u256_tuple!(state, overflowing_sub),
		Opcode::Div => op2_u256_fn!(state, self::arithmetic::div),
		Opcode::SDiv => op2_u256_fn!(state, self::arithmetic::sdiv),
		Opcode::Mod => op2_u256_fn!(state, self::arithmetic::rem),
		Opcode::SMod => op2_u256_fn!(state, self::arithmetic::srem),
		Opcode::AddMod => op3_u256_fn!(state, self::arithmetic::addmod),
		Opcode::MulMod => op3_u256_fn!(state, self::arithmetic::mulmod),
		Opcode::Exp => op2_u256_fn!(state, self::arithmetic::exp),
		Opcode::SignExtend => op2_u256_fn!(state, self::arithmetic::signextend),
		Opcode::Lt => op2_u256_bool_ref!(state, lt),
		Opcode::Gt => op2_u256_bool_ref!(state, gt),
		Opcode::SLt => op2_u256_fn!(state, self::bitwise::slt),
		Opcode::SGt => op2_u256_fn!(state, self::bitwise::sgt),
		Opcode::Eq => op2_u256_bool_ref!(state, eq),
		Opcode::IsZero => op1_u256_fn!(state, self::bitwise::iszero),
		Opcode::And => op2_u256!(state, bitand),
		Opcode::Or => op2_u256!(state, bitor),
		Opcode::Xor => op2_u256!(state, bitxor),
		Opcode::Not => op1_u256_fn!(state, self::bitwise::not),
		Opcode::Byte => op2_u256_fn!(state, self::bitwise::byte),
		Opcode::Shl => op2_u256_fn!(state, self::bitwise::shl),
		Opcode::Shr => op2_u256_fn!(state, self::bitwise::shr),
		Opcode::Sar => op2_u256_fn!(state, self::bitwise::sar),
		Opcode::CodeSize => self::misc::codesize(state),
		Opcode::CodeCopy => self::misc::codecopy(state),
		Opcode::CallDataLoad => self::misc::calldataload(state),
		Opcode::CallDataSize => self::misc::calldatasize(state),
		Opcode::CallDataCopy => self::misc::calldatacopy(state),
		Opcode::Pop => self::misc::pop(state),
		Opcode::MLoad => self::misc::mload(state),
		Opcode::MStore => self::misc::mstore(state),
		Opcode::MStore8 => self::misc::mstore8(state),
		Opcode::Jump => self::misc::jump(state),
		Opcode::JumpI => self::misc::jumpi(state),
		Opcode::PC => self::misc::pc(state, position),
		Opcode::MSize => self::misc::msize(state),
		Opcode::JumpDest => Control::Continue(1),
		Opcode::Push(n) => self::misc::push(state, n as usize, position),
		Opcode::Dup(n) => self::misc::dup(state, n as usize),
		Opcode::Swap(n) => self::misc::swap(state, n as usize),
		Opcode::Return => self::misc::ret(state),
		Opcode::Revert => self::misc::revert(state),
		Opcode::Invalid => Control::Exit(ExitError::DesignatedInvalid.into()),
	}
}
