#[macro_use]
mod macros;
mod arithmetic;
mod bitwise;
mod misc;
mod system;

use crate::{
	ExitException, ExitResult, ExitSucceed, Handler, Machine, Opcode, RuntimeState,
	RuntimeTrapData, Trap,
};
use core::marker::PhantomData;
use core::ops::{BitAnd, BitOr, BitXor, Deref, DerefMut};
use primitive_types::{H256, U256};

/// Control state.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Control<Td> {
	Continue,
	ContinueN(usize),
	Exit(ExitResult),
	Jump(usize),
	Trap(Td),
}

/// Evaluation function type.
pub type Efn<S, H, Td> = fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Td>;

/// The evaluation table for the EVM.
pub struct Etable<S, H, Tr: Trap<S>, F = Efn<S, H, <Tr as Trap<S>>::Data>>(
	[F; 256],
	PhantomData<(S, H, Tr)>,
);

impl<S, H, Tr: Trap<S>, F> Deref for Etable<S, H, Tr, F> {
	type Target = [F; 256];

	fn deref(&self) -> &[F; 256] {
		&self.0
	}
}

impl<S, H, Tr: Trap<S>, F> DerefMut for Etable<S, H, Tr, F> {
	fn deref_mut(&mut self) -> &mut [F; 256] {
		&mut self.0
	}
}

impl<S, H, Tr, F> Etable<S, H, Tr, F>
where
	Tr: Trap<S>,
	F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr::Data>,
{
	/// Wrap to create a new Etable.
	pub fn wrap<FW, FR>(self, wrapper: FW) -> Etable<S, H, Tr, FR>
	where
		FW: Fn(F, Opcode) -> FR,
		FR: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr::Data>,
	{
		let mut current_opcode = Opcode(0);
		Etable(
			self.0.map(|f| {
				let fr = wrapper(f, current_opcode);
				if current_opcode != Opcode(255) {
					current_opcode.0 = current_opcode.0 + 1;
				}
				fr
			}),
			PhantomData,
		)
	}
}

impl<S, H, Tr: Trap<S>> Etable<S, H, Tr> {
	/// Default core value for Etable.
	pub const fn core() -> Self {
		let mut table = [eval_unknown as _; 256];

		table[Opcode::STOP.as_usize()] = eval_stop as _;
		table[Opcode::ADD.as_usize()] = eval_add as _;
		table[Opcode::MUL.as_usize()] = eval_mul as _;
		table[Opcode::SUB.as_usize()] = eval_sub as _;
		table[Opcode::DIV.as_usize()] = eval_div as _;
		table[Opcode::SDIV.as_usize()] = eval_sdiv as _;
		table[Opcode::MOD.as_usize()] = eval_mod as _;
		table[Opcode::SMOD.as_usize()] = eval_smod as _;
		table[Opcode::ADDMOD.as_usize()] = eval_addmod as _;
		table[Opcode::MULMOD.as_usize()] = eval_mulmod as _;
		table[Opcode::EXP.as_usize()] = eval_exp as _;
		table[Opcode::SIGNEXTEND.as_usize()] = eval_signextend as _;
		table[Opcode::LT.as_usize()] = eval_lt as _;
		table[Opcode::GT.as_usize()] = eval_gt as _;
		table[Opcode::SLT.as_usize()] = eval_slt as _;
		table[Opcode::SGT.as_usize()] = eval_sgt as _;
		table[Opcode::EQ.as_usize()] = eval_eq as _;
		table[Opcode::ISZERO.as_usize()] = eval_iszero as _;
		table[Opcode::AND.as_usize()] = eval_and as _;
		table[Opcode::OR.as_usize()] = eval_or as _;
		table[Opcode::XOR.as_usize()] = eval_xor as _;
		table[Opcode::NOT.as_usize()] = eval_not as _;
		table[Opcode::BYTE.as_usize()] = eval_byte as _;
		table[Opcode::SHL.as_usize()] = eval_shl as _;
		table[Opcode::SHR.as_usize()] = eval_shr as _;
		table[Opcode::SAR.as_usize()] = eval_sar as _;
		table[Opcode::CODESIZE.as_usize()] = eval_codesize as _;
		table[Opcode::CODECOPY.as_usize()] = eval_codecopy as _;
		table[Opcode::CALLDATALOAD.as_usize()] = eval_calldataload as _;
		table[Opcode::CALLDATASIZE.as_usize()] = eval_calldatasize as _;
		table[Opcode::CALLDATACOPY.as_usize()] = eval_calldatacopy as _;
		table[Opcode::POP.as_usize()] = eval_pop as _;
		table[Opcode::MLOAD.as_usize()] = eval_mload as _;
		table[Opcode::MSTORE.as_usize()] = eval_mstore as _;
		table[Opcode::MSTORE8.as_usize()] = eval_mstore8 as _;
		table[Opcode::JUMP.as_usize()] = eval_jump as _;
		table[Opcode::JUMPI.as_usize()] = eval_jumpi as _;
		table[Opcode::PC.as_usize()] = eval_pc as _;
		table[Opcode::MSIZE.as_usize()] = eval_msize as _;
		table[Opcode::JUMPDEST.as_usize()] = eval_jumpdest as _;

		table[Opcode::PUSH0.as_usize()] = eval_push0 as _;
		table[Opcode::PUSH1.as_usize()] = eval_push1 as _;
		table[Opcode::PUSH2.as_usize()] = eval_push2 as _;
		table[Opcode::PUSH3.as_usize()] = eval_push3 as _;
		table[Opcode::PUSH4.as_usize()] = eval_push4 as _;
		table[Opcode::PUSH5.as_usize()] = eval_push5 as _;
		table[Opcode::PUSH6.as_usize()] = eval_push6 as _;
		table[Opcode::PUSH7.as_usize()] = eval_push7 as _;
		table[Opcode::PUSH8.as_usize()] = eval_push8 as _;
		table[Opcode::PUSH9.as_usize()] = eval_push9 as _;
		table[Opcode::PUSH10.as_usize()] = eval_push10 as _;
		table[Opcode::PUSH11.as_usize()] = eval_push11 as _;
		table[Opcode::PUSH12.as_usize()] = eval_push12 as _;
		table[Opcode::PUSH13.as_usize()] = eval_push13 as _;
		table[Opcode::PUSH14.as_usize()] = eval_push14 as _;
		table[Opcode::PUSH15.as_usize()] = eval_push15 as _;
		table[Opcode::PUSH16.as_usize()] = eval_push16 as _;
		table[Opcode::PUSH17.as_usize()] = eval_push17 as _;
		table[Opcode::PUSH18.as_usize()] = eval_push18 as _;
		table[Opcode::PUSH19.as_usize()] = eval_push19 as _;
		table[Opcode::PUSH20.as_usize()] = eval_push20 as _;
		table[Opcode::PUSH21.as_usize()] = eval_push21 as _;
		table[Opcode::PUSH22.as_usize()] = eval_push22 as _;
		table[Opcode::PUSH23.as_usize()] = eval_push23 as _;
		table[Opcode::PUSH24.as_usize()] = eval_push24 as _;
		table[Opcode::PUSH25.as_usize()] = eval_push25 as _;
		table[Opcode::PUSH26.as_usize()] = eval_push26 as _;
		table[Opcode::PUSH27.as_usize()] = eval_push27 as _;
		table[Opcode::PUSH28.as_usize()] = eval_push28 as _;
		table[Opcode::PUSH29.as_usize()] = eval_push29 as _;
		table[Opcode::PUSH30.as_usize()] = eval_push30 as _;
		table[Opcode::PUSH31.as_usize()] = eval_push31 as _;
		table[Opcode::PUSH32.as_usize()] = eval_push32 as _;

		table[Opcode::DUP1.as_usize()] = eval_dup1 as _;
		table[Opcode::DUP2.as_usize()] = eval_dup2 as _;
		table[Opcode::DUP3.as_usize()] = eval_dup3 as _;
		table[Opcode::DUP4.as_usize()] = eval_dup4 as _;
		table[Opcode::DUP5.as_usize()] = eval_dup5 as _;
		table[Opcode::DUP6.as_usize()] = eval_dup6 as _;
		table[Opcode::DUP7.as_usize()] = eval_dup7 as _;
		table[Opcode::DUP8.as_usize()] = eval_dup8 as _;
		table[Opcode::DUP9.as_usize()] = eval_dup9 as _;
		table[Opcode::DUP10.as_usize()] = eval_dup10 as _;
		table[Opcode::DUP11.as_usize()] = eval_dup11 as _;
		table[Opcode::DUP12.as_usize()] = eval_dup12 as _;
		table[Opcode::DUP13.as_usize()] = eval_dup13 as _;
		table[Opcode::DUP14.as_usize()] = eval_dup14 as _;
		table[Opcode::DUP15.as_usize()] = eval_dup15 as _;
		table[Opcode::DUP16.as_usize()] = eval_dup16 as _;

		table[Opcode::SWAP1.as_usize()] = eval_swap1 as _;
		table[Opcode::SWAP2.as_usize()] = eval_swap2 as _;
		table[Opcode::SWAP3.as_usize()] = eval_swap3 as _;
		table[Opcode::SWAP4.as_usize()] = eval_swap4 as _;
		table[Opcode::SWAP5.as_usize()] = eval_swap5 as _;
		table[Opcode::SWAP6.as_usize()] = eval_swap6 as _;
		table[Opcode::SWAP7.as_usize()] = eval_swap7 as _;
		table[Opcode::SWAP8.as_usize()] = eval_swap8 as _;
		table[Opcode::SWAP9.as_usize()] = eval_swap9 as _;
		table[Opcode::SWAP10.as_usize()] = eval_swap10 as _;
		table[Opcode::SWAP11.as_usize()] = eval_swap11 as _;
		table[Opcode::SWAP12.as_usize()] = eval_swap12 as _;
		table[Opcode::SWAP13.as_usize()] = eval_swap13 as _;
		table[Opcode::SWAP14.as_usize()] = eval_swap14 as _;
		table[Opcode::SWAP15.as_usize()] = eval_swap15 as _;
		table[Opcode::SWAP16.as_usize()] = eval_swap16 as _;

		table[Opcode::RETURN.as_usize()] = eval_return as _;
		table[Opcode::REVERT.as_usize()] = eval_revert as _;
		table[Opcode::INVALID.as_usize()] = eval_invalid as _;

		Self(table, PhantomData)
	}
}

impl<S, H: Handler, Tr> Etable<S, H, Tr>
where
	S: AsRef<RuntimeState>,
	H: Handler,
	Tr: Trap<S>,
	Tr::Data: From<RuntimeTrapData>,
{
	/// Runtime Etable.
	pub const fn runtime() -> Self {
		let mut table = Self::core();

		table.0[Opcode::SHA3.as_usize()] = eval_sha3 as _;
		table.0[Opcode::ADDRESS.as_usize()] = eval_address as _;
		table.0[Opcode::BALANCE.as_usize()] = eval_balance as _;
		table.0[Opcode::SELFBALANCE.as_usize()] = eval_selfbalance as _;
		table.0[Opcode::ORIGIN.as_usize()] = eval_origin as _;
		table.0[Opcode::CALLER.as_usize()] = eval_caller as _;
		table.0[Opcode::CALLVALUE.as_usize()] = eval_callvalue as _;
		table.0[Opcode::GASPRICE.as_usize()] = eval_gasprice as _;
		table.0[Opcode::EXTCODESIZE.as_usize()] = eval_extcodesize as _;
		table.0[Opcode::EXTCODEHASH.as_usize()] = eval_extcodehash as _;
		table.0[Opcode::EXTCODECOPY.as_usize()] = eval_extcodecopy as _;
		table.0[Opcode::RETURNDATASIZE.as_usize()] = eval_returndatasize as _;
		table.0[Opcode::RETURNDATACOPY.as_usize()] = eval_returndatacopy as _;
		table.0[Opcode::BLOCKHASH.as_usize()] = eval_blockhash as _;
		table.0[Opcode::COINBASE.as_usize()] = eval_coinbase as _;
		table.0[Opcode::TIMESTAMP.as_usize()] = eval_timestamp as _;
		table.0[Opcode::NUMBER.as_usize()] = eval_number as _;
		table.0[Opcode::DIFFICULTY.as_usize()] = eval_difficulty as _;
		table.0[Opcode::GASLIMIT.as_usize()] = eval_gaslimit as _;
		table.0[Opcode::SLOAD.as_usize()] = eval_sload as _;
		table.0[Opcode::SSTORE.as_usize()] = eval_sstore as _;
		table.0[Opcode::GAS.as_usize()] = eval_gas as _;
		table.0[Opcode::LOG0.as_usize()] = eval_log0 as _;
		table.0[Opcode::LOG1.as_usize()] = eval_log1 as _;
		table.0[Opcode::LOG2.as_usize()] = eval_log2 as _;
		table.0[Opcode::LOG3.as_usize()] = eval_log3 as _;
		table.0[Opcode::LOG4.as_usize()] = eval_log4 as _;
		table.0[Opcode::SUICIDE.as_usize()] = eval_suicide as _;
		table.0[Opcode::CHAINID.as_usize()] = eval_chainid as _;
		table.0[Opcode::BASEFEE.as_usize()] = eval_basefee as _;

		// table.0[Opcode::CREATE.as_usize()] = eval_trap as _;
		// table.0[Opcode::CREATE2.as_usize()] = eval_trap as _;
		// table.0[Opcode::CALL.as_usize()] = eval_trap as _;
		// table.0[Opcode::CALLCODE.as_usize()] = eval_trap as _;
		// table.0[Opcode::DELEGATECALL.as_usize()] = eval_trap as _;
		// table.0[Opcode::STATICCALL.as_usize()] = eval_trap as _;

		table
	}
}

fn eval_stop<S, H, Td>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	Control::Exit(ExitSucceed::Stopped.into())
}

fn eval_add<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_tuple!(machine, overflowing_add)
}

fn eval_mul<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_tuple!(machine, overflowing_mul)
}

fn eval_sub<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_tuple!(machine, overflowing_sub)
}

fn eval_div<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_fn!(machine, self::arithmetic::div)
}

fn eval_sdiv<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_fn!(machine, self::arithmetic::sdiv)
}

fn eval_mod<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_fn!(machine, self::arithmetic::rem)
}

fn eval_smod<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_fn!(machine, self::arithmetic::srem)
}

fn eval_addmod<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op3_u256_fn!(machine, self::arithmetic::addmod)
}

fn eval_mulmod<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op3_u256_fn!(machine, self::arithmetic::mulmod)
}

fn eval_exp<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_fn!(machine, self::arithmetic::exp)
}

fn eval_signextend<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_fn!(machine, self::arithmetic::signextend)
}

fn eval_lt<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_bool_ref!(machine, lt)
}

fn eval_gt<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_bool_ref!(machine, gt)
}

fn eval_slt<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_fn!(machine, self::bitwise::slt)
}

fn eval_sgt<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_fn!(machine, self::bitwise::sgt)
}

fn eval_eq<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_bool_ref!(machine, eq)
}

fn eval_iszero<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op1_u256_fn!(machine, self::bitwise::iszero)
}

fn eval_and<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256!(machine, bitand)
}

fn eval_or<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256!(machine, bitor)
}

fn eval_xor<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256!(machine, bitxor)
}

fn eval_not<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op1_u256_fn!(machine, self::bitwise::not)
}

fn eval_byte<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_fn!(machine, self::bitwise::byte)
}

fn eval_shl<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_fn!(machine, self::bitwise::shl)
}

fn eval_shr<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_fn!(machine, self::bitwise::shr)
}

fn eval_sar<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	op2_u256_fn!(machine, self::bitwise::sar)
}

fn eval_codesize<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::codesize(machine)
}

fn eval_codecopy<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::codecopy(machine)
}

fn eval_calldataload<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::calldataload(machine)
}

fn eval_calldatasize<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::calldatasize(machine)
}

fn eval_calldatacopy<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::calldatacopy(machine)
}

fn eval_pop<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::pop(machine)
}

fn eval_mload<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::mload(machine)
}

fn eval_mstore<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::mstore(machine)
}

fn eval_mstore8<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::mstore8(machine)
}

fn eval_jump<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::jump(machine)
}

fn eval_jumpi<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::jumpi(machine)
}

fn eval_pc<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::pc(machine, position)
}

fn eval_msize<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::msize(machine)
}

fn eval_jumpdest<S, H, Td>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	Control::Continue
}

fn eval_push0<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 0, position)
}

fn eval_push1<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 1, position)
}

fn eval_push2<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 2, position)
}

fn eval_push3<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 3, position)
}

fn eval_push4<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 4, position)
}

fn eval_push5<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 5, position)
}

fn eval_push6<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 6, position)
}

fn eval_push7<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 7, position)
}

fn eval_push8<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 8, position)
}

fn eval_push9<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 9, position)
}

fn eval_push10<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 10, position)
}

fn eval_push11<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 11, position)
}

fn eval_push12<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 12, position)
}

fn eval_push13<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 13, position)
}

fn eval_push14<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 14, position)
}

fn eval_push15<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 15, position)
}

fn eval_push16<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 16, position)
}

fn eval_push17<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 17, position)
}

fn eval_push18<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 18, position)
}

fn eval_push19<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 19, position)
}

fn eval_push20<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 20, position)
}

fn eval_push21<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 21, position)
}

fn eval_push22<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 22, position)
}

fn eval_push23<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 23, position)
}

fn eval_push24<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 24, position)
}

fn eval_push25<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 25, position)
}

fn eval_push26<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 26, position)
}

fn eval_push27<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 27, position)
}

fn eval_push28<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 28, position)
}

fn eval_push29<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 29, position)
}

fn eval_push30<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 30, position)
}

fn eval_push31<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 31, position)
}

fn eval_push32<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	position: usize,
) -> Control<Td> {
	self::misc::push(machine, 32, position)
}

fn eval_dup1<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 1)
}

fn eval_dup2<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 2)
}

fn eval_dup3<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 3)
}

fn eval_dup4<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 4)
}

fn eval_dup5<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 5)
}

fn eval_dup6<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 6)
}

fn eval_dup7<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 7)
}

fn eval_dup8<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 8)
}

fn eval_dup9<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 9)
}

fn eval_dup10<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 10)
}

fn eval_dup11<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 11)
}

fn eval_dup12<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 12)
}

fn eval_dup13<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 13)
}

fn eval_dup14<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 14)
}

fn eval_dup15<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 15)
}

fn eval_dup16<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::dup(machine, 16)
}

fn eval_swap1<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 1)
}

fn eval_swap2<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 2)
}

fn eval_swap3<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 3)
}

fn eval_swap4<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 4)
}

fn eval_swap5<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 5)
}

fn eval_swap6<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 6)
}

fn eval_swap7<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 7)
}

fn eval_swap8<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 8)
}

fn eval_swap9<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 9)
}

fn eval_swap10<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 10)
}

fn eval_swap11<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 11)
}

fn eval_swap12<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 12)
}

fn eval_swap13<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 13)
}

fn eval_swap14<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 14)
}

fn eval_swap15<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 15)
}

fn eval_swap16<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::swap(machine, 16)
}

fn eval_return<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::ret(machine)
}

fn eval_revert<S, H, Td>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::misc::revert(machine)
}

fn eval_invalid<S, H, Td>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	Control::Exit(ExitException::DesignatedInvalid.into())
}

fn eval_unknown<S, H, Td>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	Control::Exit(ExitException::InvalidOpcode(opcode).into())
}

fn eval_sha3<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::sha3(machine)
}

fn eval_address<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::address(machine)
}

fn eval_balance<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::balance(machine, handle)
}

fn eval_selfbalance<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::selfbalance(machine, handle)
}

fn eval_origin<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::origin(machine, handle)
}

fn eval_caller<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::caller(machine)
}

fn eval_callvalue<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::callvalue(machine)
}

fn eval_gasprice<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::gasprice(machine, handle)
}

fn eval_extcodesize<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::extcodesize(machine, handle)
}

fn eval_extcodehash<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::extcodehash(machine, handle)
}

fn eval_extcodecopy<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::extcodecopy(machine, handle)
}

fn eval_returndatasize<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::returndatasize(machine)
}

fn eval_returndatacopy<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::returndatacopy(machine)
}

fn eval_blockhash<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::blockhash(machine, handle)
}

fn eval_coinbase<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::coinbase(machine, handle)
}

fn eval_timestamp<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::timestamp(machine, handle)
}

fn eval_number<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::number(machine, handle)
}

fn eval_difficulty<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::prevrandao(machine, handle)
}

fn eval_gaslimit<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::gaslimit(machine, handle)
}

fn eval_sload<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::sload(machine, handle)
}

fn eval_sstore<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::sstore(machine, handle)
}

fn eval_gas<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::gas(machine, handle)
}

fn eval_log0<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::log(machine, 0, handle)
}

fn eval_log1<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::log(machine, 1, handle)
}

fn eval_log2<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::log(machine, 2, handle)
}

fn eval_log3<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::log(machine, 3, handle)
}

fn eval_log4<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::log(machine, 4, handle)
}

fn eval_suicide<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::suicide(machine, handle)
}

fn eval_chainid<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::chainid(machine, handle)
}

fn eval_basefee<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handle: &mut H,
	_opcode: Opcode,
	_position: usize,
) -> Control<Td> {
	self::system::basefee(machine, handle)
}
