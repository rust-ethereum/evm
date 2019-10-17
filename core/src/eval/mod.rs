#[macro_use]
mod macros;
mod arithmetic;
mod bitwise;

use core::ops::{BitAnd, BitOr, BitXor};
use primitive_types::{H256, U256};
use crate::{ExitReason, Core, Opcode};

pub enum Control {
    Continue(usize),
    Exit(ExitReason),
    Jump(usize),
}

pub fn eval(opcode: Opcode, position: usize, state: &mut Core) -> Control {
    match opcode {
        Opcode::Stop => Control::Exit(ExitReason::Stopped),
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
        _ => unimplemented!(),
    }
}
