#[macro_use]
mod macros;
mod arithmetic;
mod bitwise;
mod misc;
mod system;

use core::ops::{BitAnd, BitOr, BitXor};

use primitive_types::{H256, U256};

use crate::{
	error::{CallCreateTrap, ExitException, ExitSucceed, TrapConstruct},
	etable::Control,
	machine::Machine,
	opcode::Opcode,
	runtime::{GasState, RuntimeBackend, RuntimeEnvironment, RuntimeState},
};

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

macro_rules! eval_push {
    ($($num:expr),*) => {
		$(paste::paste! {
			pub fn [<eval_push $num>]<S, H, Tr>(
				machine: &mut Machine<S>,
				_handle: &mut H,
				_opcode: Opcode,
				position: usize,
			) -> Control<Tr> {
				self::misc::push(machine, $num, position)
			}
		})*
	};
}

eval_push! {
	0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
	17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
}

macro_rules! eval_dup {
    ($($num:expr),*) => {
		$(paste::paste! {
			pub fn [<eval_dup $num>]<S, H, Tr>(
				machine: &mut Machine<S>,
				_handle: &mut H,
				_opcode: Opcode,
				_position: usize,
			) -> Control<Tr> {
				self::misc::dup(machine, $num)
			}
		})*
	};
}

eval_dup! { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16 }

macro_rules! eval_swap {
    ($($num:expr),*) => {
		$(paste::paste! {
			pub fn [<eval_swap $num>]<S, H, Tr>(
				machine: &mut Machine<S>,
				_handle: &mut H,
				_opcode: Opcode,
				_position: usize,
			) -> Control<Tr> {
				self::misc::swap(machine, $num)
			}
		})*
	};
}

eval_swap! { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16 }

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

macro_rules! eval_log {
    ($($num:expr),*) => {
		$(paste::paste! {
			pub fn [<eval_log $num>]<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
				machine: &mut Machine<S>,
				handle: &mut H,
				_opcode: Opcode,
				_position: usize,
			) -> Control<Tr> {
				self::system::log(machine, $num, handle)
			}
		})*
	};
}

eval_log! { 0, 1, 2, 3, 4 }

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

pub fn eval_call_create_trap<S, H, Tr: TrapConstruct<CallCreateTrap>>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	opcode: Opcode,
	_position: usize,
) -> Control<Tr> {
	let trap = match opcode {
		Opcode::CREATE => CallCreateTrap::Create,
		Opcode::CREATE2 => CallCreateTrap::Create2,
		Opcode::CALL => CallCreateTrap::Call,
		Opcode::CALLCODE => CallCreateTrap::CallCode,
		Opcode::DELEGATECALL => CallCreateTrap::DelegateCall,
		Opcode::STATICCALL => CallCreateTrap::StaticCall,
		_ => return Control::Exit(Err(ExitException::InvalidOpcode(opcode).into())),
	};

	Control::Trap(Tr::construct(trap))
}
