#[macro_use]
mod macros;
mod arithmetic;
mod bitwise;
mod misc;
mod system;

use crate::{
	CallCreateTrap, Control, ExitException, ExitSucceed, GasState, Machine, Opcode, RuntimeBackend,
	RuntimeEnvironment, RuntimeState,
};
use core::ops::{BitAnd, BitOr, BitXor};
use primitive_types::{H256, U256};

pub fn eval_pass<S, H, Tr>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	Control::Continue
}

pub fn eval_stop<S, H, Tr>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	Control::Exit(ExitSucceed::Stopped.into())
}

pub fn eval_add<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_tuple!(machine, overflowing_add)
}

pub fn eval_mul<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_tuple!(machine, overflowing_mul)
}

pub fn eval_sub<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_tuple!(machine, overflowing_sub)
}

pub fn eval_div<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::arithmetic::div)
}

pub fn eval_sdiv<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::arithmetic::sdiv)
}

pub fn eval_mod<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::arithmetic::rem)
}

pub fn eval_smod<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::arithmetic::srem)
}

pub fn eval_addmod<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op3_u256_fn!(machine, self::arithmetic::addmod)
}

pub fn eval_mulmod<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op3_u256_fn!(machine, self::arithmetic::mulmod)
}

pub fn eval_exp<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::arithmetic::exp)
}

pub fn eval_signextend<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::arithmetic::signextend)
}

pub fn eval_lt<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_bool_ref!(machine, lt)
}

pub fn eval_gt<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_bool_ref!(machine, gt)
}

pub fn eval_slt<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::bitwise::slt)
}

pub fn eval_sgt<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::bitwise::sgt)
}

pub fn eval_eq<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_bool_ref!(machine, eq)
}

pub fn eval_iszero<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op1_u256_fn!(machine, self::bitwise::iszero)
}

pub fn eval_and<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256!(machine, bitand)
}

pub fn eval_or<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256!(machine, bitor)
}

pub fn eval_xor<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256!(machine, bitxor)
}

pub fn eval_not<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op1_u256_fn!(machine, self::bitwise::not)
}

pub fn eval_byte<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::bitwise::byte)
}

pub fn eval_shl<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::bitwise::shl)
}

pub fn eval_shr<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::bitwise::shr)
}

pub fn eval_sar<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::bitwise::sar)
}

pub fn eval_codesize<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::codesize(machine)
}

pub fn eval_codecopy<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::codecopy(machine)
}

pub fn eval_calldataload<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::calldataload(machine)
}

pub fn eval_calldatasize<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::calldatasize(machine)
}

pub fn eval_calldatacopy<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::calldatacopy(machine)
}

pub fn eval_pop<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::pop(machine)
}

pub fn eval_mload<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::mload(machine)
}

pub fn eval_mstore<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::mstore(machine)
}

pub fn eval_mstore8<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::mstore8(machine)
}

pub fn eval_jump<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::jump(machine)
}

pub fn eval_jumpi<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::jumpi(machine)
}

pub fn eval_pc<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::pc(machine, position)
}

pub fn eval_msize<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::msize(machine)
}

pub fn eval_jumpdest<S, H, Tr>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	Control::Continue
}

pub fn eval_push0<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 0, position)
}

pub fn eval_push1<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 1, position)
}

pub fn eval_push2<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 2, position)
}

pub fn eval_push3<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 3, position)
}

pub fn eval_push4<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 4, position)
}

pub fn eval_push5<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 5, position)
}

pub fn eval_push6<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 6, position)
}

pub fn eval_push7<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 7, position)
}

pub fn eval_push8<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 8, position)
}

pub fn eval_push9<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 9, position)
}

pub fn eval_push10<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 10, position)
}

pub fn eval_push11<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 11, position)
}

pub fn eval_push12<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 12, position)
}

pub fn eval_push13<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 13, position)
}

pub fn eval_push14<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 14, position)
}

pub fn eval_push15<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 15, position)
}

pub fn eval_push16<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 16, position)
}

pub fn eval_push17<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 17, position)
}

pub fn eval_push18<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 18, position)
}

pub fn eval_push19<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 19, position)
}

pub fn eval_push20<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 20, position)
}

pub fn eval_push21<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 21, position)
}

pub fn eval_push22<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 22, position)
}

pub fn eval_push23<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 23, position)
}

pub fn eval_push24<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 24, position)
}

pub fn eval_push25<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 25, position)
}

pub fn eval_push26<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 26, position)
}

pub fn eval_push27<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 27, position)
}

pub fn eval_push28<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 28, position)
}

pub fn eval_push29<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 29, position)
}

pub fn eval_push30<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 30, position)
}

pub fn eval_push31<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 31, position)
}

pub fn eval_push32<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Tr> {
	self::misc::push(machine, 32, position)
}

pub fn eval_dup1<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 1)
}

pub fn eval_dup2<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 2)
}

pub fn eval_dup3<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 3)
}

pub fn eval_dup4<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 4)
}

pub fn eval_dup5<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 5)
}

pub fn eval_dup6<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 6)
}

pub fn eval_dup7<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 7)
}

pub fn eval_dup8<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 8)
}

pub fn eval_dup9<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 9)
}

pub fn eval_dup10<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 10)
}

pub fn eval_dup11<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 11)
}

pub fn eval_dup12<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 12)
}

pub fn eval_dup13<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 13)
}

pub fn eval_dup14<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 14)
}

pub fn eval_dup15<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 15)
}

pub fn eval_dup16<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::dup(machine, 16)
}

pub fn eval_swap1<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 1)
}

pub fn eval_swap2<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 2)
}

pub fn eval_swap3<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 3)
}

pub fn eval_swap4<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 4)
}

pub fn eval_swap5<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 5)
}

pub fn eval_swap6<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 6)
}

pub fn eval_swap7<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 7)
}

pub fn eval_swap8<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 8)
}

pub fn eval_swap9<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 9)
}

pub fn eval_swap10<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 10)
}

pub fn eval_swap11<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 11)
}

pub fn eval_swap12<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 12)
}

pub fn eval_swap13<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 13)
}

pub fn eval_swap14<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 14)
}

pub fn eval_swap15<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 15)
}

pub fn eval_swap16<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::swap(machine, 16)
}

pub fn eval_return<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::ret(machine)
}

pub fn eval_revert<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::misc::revert(machine)
}

pub fn eval_invalid<S, H, Tr>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	Control::Exit(ExitException::DesignatedInvalid.into())
}

pub fn eval_unknown<S, H, Tr>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	Control::Exit(ExitException::InvalidOpcode(opcode).into())
}

pub fn eval_sha3<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::sha3(machine)
}

pub fn eval_address<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::address(machine)
}

pub fn eval_balance<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::balance(machine, handle)
}

pub fn eval_selfbalance<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::selfbalance(machine, handle)
}

pub fn eval_origin<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::origin(machine, handle)
}

pub fn eval_caller<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::caller(machine)
}

pub fn eval_callvalue<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::callvalue(machine)
}

pub fn eval_gasprice<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::gasprice(machine, handle)
}

pub fn eval_extcodesize<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::extcodesize(machine, handle)
}

pub fn eval_extcodehash<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::extcodehash(machine, handle)
}

pub fn eval_extcodecopy<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::extcodecopy(machine, handle)
}

pub fn eval_returndatasize<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::returndatasize(machine)
}

pub fn eval_returndatacopy<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::returndatacopy(machine)
}

pub fn eval_blockhash<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::blockhash(machine, handle)
}

pub fn eval_coinbase<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::coinbase(machine, handle)
}

pub fn eval_timestamp<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::timestamp(machine, handle)
}

pub fn eval_number<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::number(machine, handle)
}

pub fn eval_difficulty<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::prevrandao(machine, handle)
}

pub fn eval_gaslimit<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::gaslimit(machine, handle)
}

pub fn eval_sload<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::sload(machine, handle)
}

pub fn eval_sstore<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::sstore(machine, handle)
}

pub fn eval_gas<S: GasState, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::gas(machine, handle)
}

pub fn eval_log0<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::log(machine, 0, handle)
}

pub fn eval_log1<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::log(machine, 1, handle)
}

pub fn eval_log2<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::log(machine, 2, handle)
}

pub fn eval_log3<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::log(machine, 3, handle)
}

pub fn eval_log4<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::log(machine, 4, handle)
}

pub fn eval_suicide<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::suicide(machine, handle)
}

pub fn eval_chainid<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::chainid(machine, handle)
}

pub fn eval_basefee<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	self::system::basefee(machine, handle)
}

pub fn eval_call_create_trap<S, H, Tr: CallCreateTrap>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	Control::Trap(Tr::call_create_trap(opcode))
}
