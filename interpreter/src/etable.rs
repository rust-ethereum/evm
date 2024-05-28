use core::{
	marker::PhantomData,
	ops::{Deref, DerefMut},
};

use crate::{
	error::{CallCreateTrap, ExitResult, TrapConstruct},
	eval::*,
	machine::Machine,
	opcode::Opcode,
	runtime::{GasState, RuntimeBackend, RuntimeEnvironment, RuntimeState},
};

pub trait EtableSet {
	type State;
	type Handle;
	type Trap;

	fn eval(
		&self,
		machine: &mut Machine<Self::State>,
		handle: &mut Self::Handle,
		opcode: Opcode,
		position: usize,
	) -> Control<Self::Trap>;
}

impl<S, H, Tr, F> EtableSet for Etable<S, H, Tr, F>
where
	F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
{
	type State = S;
	type Handle = H;
	type Trap = Tr;

	fn eval(
		&self,
		machine: &mut Machine<S>,
		handle: &mut H,
		opcode: Opcode,
		position: usize,
	) -> Control<Tr> {
		self[opcode.as_usize()](machine, handle, opcode, position)
	}
}

impl<S, H, Tr, F1, F2> EtableSet for (Etable<S, H, Tr, F1>, Etable<S, H, Tr, F2>)
where
	F1: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	F2: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
{
	type State = S;
	type Handle = H;
	type Trap = Tr;

	fn eval(
		&self,
		machine: &mut Machine<S>,
		handle: &mut H,
		opcode: Opcode,
		position: usize,
	) -> Control<Tr> {
		let mut ret = self.0[opcode.as_usize()](machine, handle, opcode, position);

		if matches!(ret, Control::Continue) {
			ret = self.1[opcode.as_usize()](machine, handle, opcode, position);
		}

		ret
	}
}

/// Evaluation function type.
pub type Efn<S, H, Tr> = fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>;

/// The evaluation table for the EVM.
pub struct Etable<S, H, Tr, F = Efn<S, H, Tr>>([F; 256], PhantomData<(S, H, Tr)>);

unsafe impl<S, H, Tr, F: Send> Send for Etable<S, H, Tr, F> {}
unsafe impl<S, H, Tr, F: Sync> Sync for Etable<S, H, Tr, F> {}

impl<S, H, Tr, F> Deref for Etable<S, H, Tr, F> {
	type Target = [F; 256];

	fn deref(&self) -> &[F; 256] {
		&self.0
	}
}

impl<S, H, Tr, F> DerefMut for Etable<S, H, Tr, F> {
	fn deref_mut(&mut self) -> &mut [F; 256] {
		&mut self.0
	}
}

impl<S, H, Tr, F> Etable<S, H, Tr, F>
where
	F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
{
	pub const fn single(f: F) -> Self
	where
		F: Copy,
	{
		Self([f; 256], PhantomData)
	}

	/// Wrap to create a new Etable.
	pub fn wrap<FW, FR>(self, wrapper: FW) -> Etable<S, H, Tr, FR>
	where
		FW: Fn(F, Opcode) -> FR,
		FR: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		let mut current_opcode = Opcode(0);
		Etable(
			self.0.map(|f| {
				let fr = wrapper(f, current_opcode);
				if current_opcode != Opcode(255) {
					current_opcode.0 += 1;
				}
				fr
			}),
			PhantomData,
		)
	}
}

impl<S, H, Tr> Etable<S, H, Tr> {
	#[must_use]
	pub const fn none() -> Self {
		Self([eval_unknown as _; 256], PhantomData)
	}

	#[must_use]
	pub const fn pass() -> Self {
		Self([eval_pass as _; 256], PhantomData)
	}

	/// Default core value for Etable.
	#[must_use]
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

		table[Opcode::CALLDATALOAD.as_usize()] = eval_calldataload as _;
		table[Opcode::CALLDATASIZE.as_usize()] = eval_calldatasize as _;
		table[Opcode::CALLDATACOPY.as_usize()] = eval_calldatacopy as _;
		table[Opcode::CODESIZE.as_usize()] = eval_codesize as _;
		table[Opcode::CODECOPY.as_usize()] = eval_codecopy as _;

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

impl<S, H: RuntimeEnvironment + RuntimeBackend, Tr: TrapConstruct<CallCreateTrap>> Etable<S, H, Tr>
where
	S: AsRef<RuntimeState> + GasState,
{
	/// Runtime Etable.
	#[must_use]
	pub const fn runtime() -> Self {
		let mut table = Self::core();

		table.0[Opcode::SHA3.as_usize()] = eval_sha3 as _;

		table.0[Opcode::ADDRESS.as_usize()] = eval_address as _;
		table.0[Opcode::BALANCE.as_usize()] = eval_balance as _;
		table.0[Opcode::ORIGIN.as_usize()] = eval_origin as _;
		table.0[Opcode::CALLER.as_usize()] = eval_caller as _;
		table.0[Opcode::CALLVALUE.as_usize()] = eval_callvalue as _;

		table.0[Opcode::GASPRICE.as_usize()] = eval_gasprice as _;
		table.0[Opcode::EXTCODESIZE.as_usize()] = eval_extcodesize as _;
		table.0[Opcode::EXTCODECOPY.as_usize()] = eval_extcodecopy as _;
		table.0[Opcode::RETURNDATASIZE.as_usize()] = eval_returndatasize as _;
		table.0[Opcode::RETURNDATACOPY.as_usize()] = eval_returndatacopy as _;
		table.0[Opcode::EXTCODEHASH.as_usize()] = eval_extcodehash as _;

		table.0[Opcode::BLOCKHASH.as_usize()] = eval_blockhash as _;
		table.0[Opcode::COINBASE.as_usize()] = eval_coinbase as _;
		table.0[Opcode::TIMESTAMP.as_usize()] = eval_timestamp as _;
		table.0[Opcode::NUMBER.as_usize()] = eval_number as _;
		table.0[Opcode::DIFFICULTY.as_usize()] = eval_difficulty as _;
		table.0[Opcode::GASLIMIT.as_usize()] = eval_gaslimit as _;
		table.0[Opcode::CHAINID.as_usize()] = eval_chainid as _;
		table.0[Opcode::SELFBALANCE.as_usize()] = eval_selfbalance as _;
		table.0[Opcode::BASEFEE.as_usize()] = eval_basefee as _;

		table.0[Opcode::SLOAD.as_usize()] = eval_sload as _;
		table.0[Opcode::SSTORE.as_usize()] = eval_sstore as _;

		table.0[Opcode::GAS.as_usize()] = eval_gas as _;

		table.0[Opcode::LOG0.as_usize()] = eval_log0 as _;
		table.0[Opcode::LOG1.as_usize()] = eval_log1 as _;
		table.0[Opcode::LOG2.as_usize()] = eval_log2 as _;
		table.0[Opcode::LOG3.as_usize()] = eval_log3 as _;
		table.0[Opcode::LOG4.as_usize()] = eval_log4 as _;

		table.0[Opcode::CREATE.as_usize()] = eval_call_create_trap as _;
		table.0[Opcode::CALL.as_usize()] = eval_call_create_trap as _;
		table.0[Opcode::CALLCODE.as_usize()] = eval_call_create_trap as _;

		table.0[Opcode::DELEGATECALL.as_usize()] = eval_call_create_trap as _;
		table.0[Opcode::CREATE2.as_usize()] = eval_call_create_trap as _;

		table.0[Opcode::STATICCALL.as_usize()] = eval_call_create_trap as _;

		table.0[Opcode::SUICIDE.as_usize()] = eval_suicide as _;

		table
	}
}

/// Control state.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Control<Trap> {
	Continue,
	ContinueN(usize),
	Exit(ExitResult),
	Jump(usize),
	Trap(Trap),
}
