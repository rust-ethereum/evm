//! Actual opcode evaluation implementations.

#[macro_use]
mod macros;
mod arithmetic;
mod bitwise;
mod misc;
mod system;

use crate::uint::{U256, U256Ext};
use alloc::boxed::Box;
use core::ops::{BitAnd, BitOr, BitXor};

use crate::{
	Control, ExitException, ExitSucceed, Machine, Opcode,
	runtime::{GasState, RuntimeBackend, RuntimeConfig, RuntimeEnvironment, RuntimeState},
	trap::{CallCreateOpcode, CallCreateTrap},
};

/// Do nothing, and continue to the next instruction.
pub fn eval_pass<S, H, Tr>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	Control::Continue(1)
}

/// `STOP`
pub fn eval_stop<S, H, Tr>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	Control::Exit(ExitSucceed::Stopped.into())
}

/// `ADD`
pub fn eval_add<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_tuple!(machine, overflowing_add)
}

/// `MUL`
pub fn eval_mul<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_tuple!(machine, overflowing_mul)
}

/// `SUB`
pub fn eval_sub<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_tuple!(machine, overflowing_sub)
}

/// `DIV`
pub fn eval_div<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::arithmetic::div)
}

/// `SDIV`
pub fn eval_sdiv<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::arithmetic::sdiv)
}

/// `MOD`
pub fn eval_mod<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::arithmetic::rem)
}

/// `SMOD`
pub fn eval_smod<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::arithmetic::srem)
}

/// `ADDMOD`
pub fn eval_addmod<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op3_u256_fn!(machine, self::arithmetic::addmod)
}

/// `MULMOD`
pub fn eval_mulmod<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op3_u256_fn!(machine, self::arithmetic::mulmod)
}

/// `EXP`
pub fn eval_exp<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::arithmetic::exp)
}

/// `SIGNEXTEND`
pub fn eval_signextend<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::arithmetic::signextend)
}

/// `LT`
pub fn eval_lt<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_bool_ref!(machine, lt)
}

/// `GT`
pub fn eval_gt<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_bool_ref!(machine, gt)
}

/// `SLT`
pub fn eval_slt<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::bitwise::slt)
}

/// `SGT`
pub fn eval_sgt<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::bitwise::sgt)
}

/// `EQ`
pub fn eval_eq<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_bool_ref!(machine, eq)
}

/// `ISZERO`
pub fn eval_iszero<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op1_u256_fn!(machine, self::bitwise::iszero)
}

/// `AND`
pub fn eval_and<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256!(machine, bitand)
}

/// `OR`
pub fn eval_or<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256!(machine, bitor)
}

/// `XOR`
pub fn eval_xor<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256!(machine, bitxor)
}

/// `NOT`
pub fn eval_not<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op1_u256_fn!(machine, self::bitwise::not)
}

/// `BYTE`
pub fn eval_byte<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::bitwise::byte)
}

/// `SHL`
pub fn eval_shl<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::bitwise::shl)
}

/// `SHR`
pub fn eval_shr<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::bitwise::shr)
}

/// `SAR`
pub fn eval_sar<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u256_fn!(machine, self::bitwise::sar)
}

/// `CODESIZE`
pub fn eval_codesize<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::codesize(machine)
}

/// `CODECOPY`
pub fn eval_codecopy<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::codecopy(machine)
}

/// `CALLDATALOAD`
pub fn eval_calldataload<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::calldataload(machine)
}

/// `CALLDATASIZE`
pub fn eval_calldatasize<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::calldatasize(machine)
}

/// `CALLDATACOPY`
pub fn eval_calldatacopy<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::calldatacopy(machine)
}

/// `POP`
pub fn eval_pop<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::pop(machine)
}

/// `MLOAD`
pub fn eval_mload<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::mload(machine)
}

/// `MSTORE`
pub fn eval_mstore<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::mstore(machine)
}

/// `MSTORE8`
pub fn eval_mstore8<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::mstore8(machine)
}

/// `JUMP`
pub fn eval_jump<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::jump(machine)
}

/// `JUMPI`
pub fn eval_jumpi<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::jumpi(machine)
}

/// `PC`
pub fn eval_pc<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	position: usize,
) -> Control<Tr> {
	self::misc::pc(machine, position)
}

/// `MSIZE`
pub fn eval_msize<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::msize(machine)
}

/// `JUMPDEST`
pub fn eval_jumpdest<S, H, Tr>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	Control::Continue(1)
}

/// `MCOPY`
pub fn eval_mcopy<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::mcopy(machine)
}

macro_rules! eval_push {
    ($($num:expr),*) => {
		$(paste::paste! {
			/// `PUSHn`
			pub fn [<eval_push $num>]<S, H, Tr>(
				machine: &mut Machine<S>,
				_handle: &mut H,
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
			/// `DUPn`
			pub fn [<eval_dup $num>]<S, H, Tr>(
				machine: &mut Machine<S>,
				_handle: &mut H,
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
			/// `SWAPn`
			pub fn [<eval_swap $num>]<S, H, Tr>(
				machine: &mut Machine<S>,
				_handle: &mut H,
				_position: usize,
			) -> Control<Tr> {
				self::misc::swap(machine, $num)
			}
		})*
	};
}

eval_swap! { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16 }

/// `RETURN`
pub fn eval_return<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::ret(machine)
}

/// `REVERT`
pub fn eval_revert<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::misc::revert(machine)
}

/// `INVALID`
pub fn eval_invalid<S, H, Tr>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	Control::Exit(ExitException::DesignatedInvalid.into())
}

/// Any unknown opcode.
pub fn eval_unknown<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	position: usize,
) -> Control<Tr> {
	Control::Exit(ExitException::InvalidOpcode(Opcode(machine.code()[position])).into())
}

/// `SHA3`
pub fn eval_sha3<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::sha3(machine)
}

/// `ADDRESS`
pub fn eval_address<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::address(machine)
}

/// `BALANCE`
pub fn eval_balance<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::balance(machine, handle)
}

/// `SELFBALANCE`
pub fn eval_selfbalance<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::selfbalance(machine, handle)
}

/// `ORIGIN`
pub fn eval_origin<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::origin(machine, handle)
}

/// `CALLER`
pub fn eval_caller<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::caller(machine)
}

/// `CALLVALUE`
pub fn eval_callvalue<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::callvalue(machine)
}

/// `GASPRICE`
pub fn eval_gasprice<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::gasprice(machine, handle)
}

/// `EXTCODESIZE`
pub fn eval_extcodesize<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::extcodesize(machine, handle)
}

/// `EXTCODEHASH`
pub fn eval_extcodehash<
	S: AsRef<RuntimeState> + AsRef<RuntimeConfig>,
	H: RuntimeEnvironment + RuntimeBackend,
	Tr,
>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::extcodehash(machine, handle)
}

/// `EXTCODECOPY`
pub fn eval_extcodecopy<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::extcodecopy(machine, handle)
}

/// `RETURNDATASIZE`
pub fn eval_returndatasize<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::returndatasize(machine)
}

/// `RETURNDATACOPY`
pub fn eval_returndatacopy<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::returndatacopy(machine)
}

/// `BLOCKHASH`
pub fn eval_blockhash<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::blockhash(machine, handle)
}

/// `COINBASE`
pub fn eval_coinbase<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::coinbase(machine, handle)
}

/// `TIMESTAMP`
pub fn eval_timestamp<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::timestamp(machine, handle)
}

/// `NUMBER`
pub fn eval_number<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::number(machine, handle)
}

/// `DIFFICULTY`
pub fn eval_difficulty<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::prevrandao(machine, handle)
}

/// `GASLIMIT`
pub fn eval_gaslimit<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::gaslimit(machine, handle)
}

/// `BLOBHASH`
pub fn eval_blobhash<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::blobhash(machine, handle)
}

/// `BLOBBASEFEE`
pub fn eval_blobbasefee<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::blobbasefee(machine, handle)
}

/// `SLOAD`
pub fn eval_sload<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::sload(machine, handle)
}

/// `SSTORE`
pub fn eval_sstore<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::sstore(machine, handle)
}

/// `GAS`
pub fn eval_gas<S: GasState, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::gas(machine, handle)
}

/// `TLOAD`
pub fn eval_tload<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::tload(machine, handle)
}

/// `TSTORE`
pub fn eval_tstore<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::tstore(machine, handle)
}

macro_rules! eval_log {
    ($($num:expr),*) => {
		$(paste::paste! {
			/// `LOGn`
			pub fn [<eval_log $num>]<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
				machine: &mut Machine<S>,
				handle: &mut H,
				_position: usize,
			) -> Control<Tr> {
				self::system::log(machine, $num, handle)
			}
		})*
	};
}

eval_log! { 0, 1, 2, 3, 4 }

/// `SUICIDE`
pub fn eval_suicide<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::suicide(machine, handle)
}

/// `CHAINID`
pub fn eval_chainid<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::chainid(machine, handle)
}

/// `BASEFEE`
pub fn eval_basefee<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	self::system::basefee(machine, handle)
}

/// `CREATE`, `CREATE2`, `CALL`, `CALLCODE`, `DELEGATECALL`, `STATICCALL`
pub fn eval_call_create_trap<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H,
	Tr: From<CallCreateTrap>,
>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	position: usize,
) -> Control<Tr> {
	let raw_opcode = Opcode(machine.code()[position]);

	let opcode = match raw_opcode {
		Opcode::CREATE => CallCreateOpcode::Create,
		Opcode::CREATE2 => CallCreateOpcode::Create2,
		Opcode::CALL => CallCreateOpcode::Call,
		Opcode::CALLCODE => CallCreateOpcode::CallCode,
		Opcode::DELEGATECALL => CallCreateOpcode::DelegateCall,
		Opcode::STATICCALL => CallCreateOpcode::StaticCall,
		_ => return Control::Exit(Err(ExitException::InvalidOpcode(raw_opcode).into())),
	};

	let trap = match CallCreateTrap::new_from(opcode, machine) {
		Ok(trap) => trap,
		Err(err) => return Control::Exit(Err(err)),
	};

	Control::Trap(Box::new(trap.into()))
}

/// Eval any known opcode, uses `match`.
pub fn eval_any<S, H, Tr>(machine: &mut Machine<S>, handle: &mut H, position: usize) -> Control<Tr>
where
	S: AsRef<RuntimeState> + AsMut<RuntimeState> + AsRef<RuntimeConfig> + GasState,
	H: RuntimeEnvironment + RuntimeBackend,
	Tr: From<CallCreateTrap>,
{
	let opcode = Opcode(machine.code()[position]);

	match opcode {
		Opcode::STOP => eval_stop(machine, handle, position),
		Opcode::ADD => eval_add(machine, handle, position),
		Opcode::MUL => eval_mul(machine, handle, position),
		Opcode::SUB => eval_sub(machine, handle, position),
		Opcode::DIV => eval_div(machine, handle, position),
		Opcode::SDIV => eval_sdiv(machine, handle, position),
		Opcode::MOD => eval_mod(machine, handle, position),
		Opcode::SMOD => eval_smod(machine, handle, position),
		Opcode::ADDMOD => eval_addmod(machine, handle, position),
		Opcode::MULMOD => eval_mulmod(machine, handle, position),
		Opcode::EXP => eval_exp(machine, handle, position),
		Opcode::SIGNEXTEND => eval_signextend(machine, handle, position),

		Opcode::LT => eval_lt(machine, handle, position),
		Opcode::GT => eval_gt(machine, handle, position),
		Opcode::SLT => eval_slt(machine, handle, position),
		Opcode::SGT => eval_sgt(machine, handle, position),
		Opcode::EQ => eval_eq(machine, handle, position),
		Opcode::ISZERO => eval_iszero(machine, handle, position),
		Opcode::AND => eval_and(machine, handle, position),
		Opcode::OR => eval_or(machine, handle, position),
		Opcode::XOR => eval_xor(machine, handle, position),
		Opcode::NOT => eval_not(machine, handle, position),
		Opcode::BYTE => eval_byte(machine, handle, position),

		Opcode::SHL => eval_shl(machine, handle, position),
		Opcode::SHR => eval_shr(machine, handle, position),
		Opcode::SAR => eval_sar(machine, handle, position),

		Opcode::CALLDATALOAD => eval_calldataload(machine, handle, position),
		Opcode::CALLDATASIZE => eval_calldatasize(machine, handle, position),
		Opcode::CALLDATACOPY => eval_calldatacopy(machine, handle, position),
		Opcode::CODESIZE => eval_codesize(machine, handle, position),
		Opcode::CODECOPY => eval_codecopy(machine, handle, position),

		Opcode::POP => eval_pop(machine, handle, position),
		Opcode::MLOAD => eval_mload(machine, handle, position),
		Opcode::MSTORE => eval_mstore(machine, handle, position),
		Opcode::MSTORE8 => eval_mstore8(machine, handle, position),

		Opcode::JUMP => eval_jump(machine, handle, position),
		Opcode::JUMPI => eval_jumpi(machine, handle, position),
		Opcode::PC => eval_pc(machine, handle, position),
		Opcode::MSIZE => eval_msize(machine, handle, position),

		Opcode::JUMPDEST => eval_jumpdest(machine, handle, position),
		Opcode::MCOPY => eval_mcopy(machine, handle, position),

		Opcode::PUSH0 => eval_push0(machine, handle, position),
		Opcode::PUSH1 => eval_push1(machine, handle, position),
		Opcode::PUSH2 => eval_push2(machine, handle, position),
		Opcode::PUSH3 => eval_push3(machine, handle, position),
		Opcode::PUSH4 => eval_push4(machine, handle, position),
		Opcode::PUSH5 => eval_push5(machine, handle, position),
		Opcode::PUSH6 => eval_push6(machine, handle, position),
		Opcode::PUSH7 => eval_push7(machine, handle, position),
		Opcode::PUSH8 => eval_push8(machine, handle, position),
		Opcode::PUSH9 => eval_push9(machine, handle, position),
		Opcode::PUSH10 => eval_push10(machine, handle, position),
		Opcode::PUSH11 => eval_push11(machine, handle, position),
		Opcode::PUSH12 => eval_push12(machine, handle, position),
		Opcode::PUSH13 => eval_push13(machine, handle, position),
		Opcode::PUSH14 => eval_push14(machine, handle, position),
		Opcode::PUSH15 => eval_push15(machine, handle, position),
		Opcode::PUSH16 => eval_push16(machine, handle, position),
		Opcode::PUSH17 => eval_push17(machine, handle, position),
		Opcode::PUSH18 => eval_push18(machine, handle, position),
		Opcode::PUSH19 => eval_push19(machine, handle, position),
		Opcode::PUSH20 => eval_push20(machine, handle, position),
		Opcode::PUSH21 => eval_push21(machine, handle, position),
		Opcode::PUSH22 => eval_push22(machine, handle, position),
		Opcode::PUSH23 => eval_push23(machine, handle, position),
		Opcode::PUSH24 => eval_push24(machine, handle, position),
		Opcode::PUSH25 => eval_push25(machine, handle, position),
		Opcode::PUSH26 => eval_push26(machine, handle, position),
		Opcode::PUSH27 => eval_push27(machine, handle, position),
		Opcode::PUSH28 => eval_push28(machine, handle, position),
		Opcode::PUSH29 => eval_push29(machine, handle, position),
		Opcode::PUSH30 => eval_push30(machine, handle, position),
		Opcode::PUSH31 => eval_push31(machine, handle, position),
		Opcode::PUSH32 => eval_push32(machine, handle, position),

		Opcode::DUP1 => eval_dup1(machine, handle, position),
		Opcode::DUP2 => eval_dup2(machine, handle, position),
		Opcode::DUP3 => eval_dup3(machine, handle, position),
		Opcode::DUP4 => eval_dup4(machine, handle, position),
		Opcode::DUP5 => eval_dup5(machine, handle, position),
		Opcode::DUP6 => eval_dup6(machine, handle, position),
		Opcode::DUP7 => eval_dup7(machine, handle, position),
		Opcode::DUP8 => eval_dup8(machine, handle, position),
		Opcode::DUP9 => eval_dup9(machine, handle, position),
		Opcode::DUP10 => eval_dup10(machine, handle, position),
		Opcode::DUP11 => eval_dup11(machine, handle, position),
		Opcode::DUP12 => eval_dup12(machine, handle, position),
		Opcode::DUP13 => eval_dup13(machine, handle, position),
		Opcode::DUP14 => eval_dup14(machine, handle, position),
		Opcode::DUP15 => eval_dup15(machine, handle, position),
		Opcode::DUP16 => eval_dup16(machine, handle, position),

		Opcode::SWAP1 => eval_swap1(machine, handle, position),
		Opcode::SWAP2 => eval_swap2(machine, handle, position),
		Opcode::SWAP3 => eval_swap3(machine, handle, position),
		Opcode::SWAP4 => eval_swap4(machine, handle, position),
		Opcode::SWAP5 => eval_swap5(machine, handle, position),
		Opcode::SWAP6 => eval_swap6(machine, handle, position),
		Opcode::SWAP7 => eval_swap7(machine, handle, position),
		Opcode::SWAP8 => eval_swap8(machine, handle, position),
		Opcode::SWAP9 => eval_swap9(machine, handle, position),
		Opcode::SWAP10 => eval_swap10(machine, handle, position),
		Opcode::SWAP11 => eval_swap11(machine, handle, position),
		Opcode::SWAP12 => eval_swap12(machine, handle, position),
		Opcode::SWAP13 => eval_swap13(machine, handle, position),
		Opcode::SWAP14 => eval_swap14(machine, handle, position),
		Opcode::SWAP15 => eval_swap15(machine, handle, position),
		Opcode::SWAP16 => eval_swap16(machine, handle, position),

		Opcode::RETURN => eval_return(machine, handle, position),

		Opcode::REVERT => eval_revert(machine, handle, position),

		Opcode::INVALID => eval_invalid(machine, handle, position),

		Opcode::SHA3 => eval_sha3(machine, handle, position),

		Opcode::ADDRESS => eval_address(machine, handle, position),
		Opcode::BALANCE => eval_balance(machine, handle, position),
		Opcode::ORIGIN => eval_origin(machine, handle, position),
		Opcode::CALLER => eval_caller(machine, handle, position),
		Opcode::CALLVALUE => eval_callvalue(machine, handle, position),

		Opcode::GASPRICE => eval_gasprice(machine, handle, position),
		Opcode::EXTCODESIZE => eval_extcodesize(machine, handle, position),
		Opcode::EXTCODECOPY => eval_extcodecopy(machine, handle, position),
		Opcode::RETURNDATASIZE => eval_returndatasize(machine, handle, position),
		Opcode::RETURNDATACOPY => eval_returndatacopy(machine, handle, position),
		Opcode::EXTCODEHASH => eval_extcodehash(machine, handle, position),

		Opcode::BLOCKHASH => eval_blockhash(machine, handle, position),
		Opcode::COINBASE => eval_coinbase(machine, handle, position),
		Opcode::TIMESTAMP => eval_timestamp(machine, handle, position),
		Opcode::NUMBER => eval_number(machine, handle, position),
		Opcode::DIFFICULTY => eval_difficulty(machine, handle, position),
		Opcode::GASLIMIT => eval_gaslimit(machine, handle, position),
		Opcode::CHAINID => eval_chainid(machine, handle, position),
		Opcode::SELFBALANCE => eval_selfbalance(machine, handle, position),
		Opcode::BASEFEE => eval_basefee(machine, handle, position),
		Opcode::BLOBHASH => eval_blobhash(machine, handle, position),
		Opcode::BLOBBASEFEE => eval_blobbasefee(machine, handle, position),

		Opcode::SLOAD => eval_sload(machine, handle, position),
		Opcode::SSTORE => eval_sstore(machine, handle, position),

		Opcode::GAS => eval_gas(machine, handle, position),

		Opcode::TLOAD => eval_tload(machine, handle, position),
		Opcode::TSTORE => eval_tstore(machine, handle, position),

		Opcode::LOG0 => eval_log0(machine, handle, position),
		Opcode::LOG1 => eval_log1(machine, handle, position),
		Opcode::LOG2 => eval_log2(machine, handle, position),
		Opcode::LOG3 => eval_log3(machine, handle, position),
		Opcode::LOG4 => eval_log4(machine, handle, position),

		Opcode::CREATE => eval_call_create_trap(machine, handle, position),
		Opcode::CALL => eval_call_create_trap(machine, handle, position),
		Opcode::CALLCODE => eval_call_create_trap(machine, handle, position),

		Opcode::DELEGATECALL => eval_call_create_trap(machine, handle, position),
		Opcode::CREATE2 => eval_call_create_trap(machine, handle, position),

		Opcode::STATICCALL => eval_call_create_trap(machine, handle, position),

		Opcode::SUICIDE => eval_suicide(machine, handle, position),

		_ => eval_unknown(machine, handle, position),
	}
}
