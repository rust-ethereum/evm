#[macro_use]
mod macros;
mod arithmetic;
mod bitwise;
mod misc;

use crate::{ExitError, ExitReason, ExitSucceed, InterpreterHandler, Machine, Opcode};
use core::ops::{BitAnd, BitOr, BitXor};
use primitive_types::{H160, U256};

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Control {
	Continue(usize),
	Exit(ExitReason),
	Jump(usize),
	Trap(Opcode),
}

#[inline]
pub fn eval<H: InterpreterHandler>(
	machine: &mut Machine,
	position: usize,
	handler: &mut H,
	address: &H160,
) -> Control {
	eval_table(machine, position, handler, address)
}

// Table-based interpreter, shows the smallest gas cost.
#[inline]
fn eval_table<H: InterpreterHandler>(
	state: &mut Machine,
	position: usize,
	handler: &mut H,
	address: &H160,
) -> Control {
	static TABLE: [fn(state: &mut Machine, opcode: Opcode, position: usize) -> Control; 256] = {
		fn eval_external(state: &mut Machine, opcode: Opcode, position: usize) -> Control {
			state.position = Ok(position + 1);
			Control::Trap(opcode)
		}
		let mut table = [eval_external as _; 256];
		macro_rules! table_elem {
			($operation:ident, $definition:expr) => {
				table_elem!($operation, _state, $definition)
			};
			($operation:ident, $state:ident, $definition:expr) => {
				table_elem!($operation, $state, _pc, $definition)
			};
			($operation:ident, $state:ident, $pc:ident, $definition:expr) => {
				#[allow(non_snake_case)]
				fn $operation($state: &mut Machine, _opcode: Opcode, $pc: usize) -> Control {
					$definition
				}
				table[Opcode::$operation.as_usize()] = $operation as _;
			};
		}
		table_elem!(ADD, state, op2_u256_tuple!(state, overflowing_add));
		table_elem!(MUL, state, op2_u256_tuple!(state, overflowing_mul));
		table_elem!(SUB, state, op2_u256_tuple!(state, overflowing_sub));
		table_elem!(DIV, state, op2_u256_fn!(state, self::arithmetic::div));
		table_elem!(SDIV, state, op2_u256_fn!(state, self::arithmetic::sdiv));
		table_elem!(EXP, state, op2_u256_fn!(state, self::arithmetic::exp));
		table_elem!(
			SIGNEXTEND,
			state,
			op2_u256_fn!(state, self::arithmetic::signextend)
		);
		table_elem!(LT, state, op2_u256_bool_ref!(state, lt));
		table_elem!(GT, state, op2_u256_bool_ref!(state, gt));
		table_elem!(SLT, state, op2_u256_fn!(state, self::bitwise::slt));
		table_elem!(SGT, state, op2_u256_fn!(state, self::bitwise::sgt));
		table_elem!(EQ, state, op2_u256_bool_ref!(state, eq));
		table_elem!(ISZERO, state, op1_u256_fn!(state, self::bitwise::iszero));
		table_elem!(AND, state, op2_u256!(state, bitand));
		table_elem!(OR, state, op2_u256!(state, bitor));
		table_elem!(XOR, state, op2_u256!(state, bitxor));
		table_elem!(NOT, state, op1_u256_fn!(state, self::bitwise::not));
		table_elem!(BYTE, state, op2_u256_fn!(state, self::bitwise::byte));
		table_elem!(SHL, state, op2_u256_fn!(state, self::bitwise::shl));
		table_elem!(SHR, state, op2_u256_fn!(state, self::bitwise::shr));
		table_elem!(SAR, state, op2_u256_fn!(state, self::bitwise::sar));
		table_elem!(POP, state, self::misc::pop(state));
		table_elem!(PC, state, position, self::misc::pc(state, position));
		table_elem!(MSIZE, state, self::misc::msize(state));
		table_elem!(PUSH0, state, self::misc::push0(state));
		table_elem!(PUSH1, state, position, self::misc::push1(state, position));
		table_elem!(PUSH2, state, position, self::misc::push2(state, position));
		table_elem!(PUSH3, state, position, self::misc::push(state, 3, position));
		table_elem!(PUSH4, state, position, self::misc::push(state, 4, position));
		table_elem!(PUSH5, state, position, self::misc::push(state, 5, position));
		table_elem!(PUSH6, state, position, self::misc::push(state, 6, position));
		table_elem!(PUSH7, state, position, self::misc::push(state, 7, position));
		table_elem!(PUSH8, state, position, self::misc::push(state, 8, position));
		table_elem!(PUSH9, state, position, self::misc::push(state, 9, position));
		table_elem!(
			PUSH10,
			state,
			position,
			self::misc::push(state, 10, position)
		);
		table_elem!(
			PUSH11,
			state,
			position,
			self::misc::push(state, 11, position)
		);
		table_elem!(
			PUSH12,
			state,
			position,
			self::misc::push(state, 12, position)
		);
		table_elem!(
			PUSH13,
			state,
			position,
			self::misc::push(state, 13, position)
		);
		table_elem!(
			PUSH14,
			state,
			position,
			self::misc::push(state, 14, position)
		);
		table_elem!(
			PUSH15,
			state,
			position,
			self::misc::push(state, 15, position)
		);
		table_elem!(
			PUSH16,
			state,
			position,
			self::misc::push(state, 16, position)
		);
		table_elem!(
			PUSH17,
			state,
			position,
			self::misc::push(state, 17, position)
		);
		table_elem!(
			PUSH18,
			state,
			position,
			self::misc::push(state, 18, position)
		);
		table_elem!(
			PUSH19,
			state,
			position,
			self::misc::push(state, 19, position)
		);
		table_elem!(
			PUSH20,
			state,
			position,
			self::misc::push(state, 20, position)
		);
		table_elem!(
			PUSH21,
			state,
			position,
			self::misc::push(state, 21, position)
		);
		table_elem!(
			PUSH22,
			state,
			position,
			self::misc::push(state, 22, position)
		);
		table_elem!(
			PUSH23,
			state,
			position,
			self::misc::push(state, 23, position)
		);
		table_elem!(
			PUSH24,
			state,
			position,
			self::misc::push(state, 24, position)
		);
		table_elem!(
			PUSH25,
			state,
			position,
			self::misc::push(state, 25, position)
		);
		table_elem!(
			PUSH26,
			state,
			position,
			self::misc::push(state, 26, position)
		);
		table_elem!(
			PUSH27,
			state,
			position,
			self::misc::push(state, 27, position)
		);
		table_elem!(
			PUSH28,
			state,
			position,
			self::misc::push(state, 28, position)
		);
		table_elem!(
			PUSH29,
			state,
			position,
			self::misc::push(state, 29, position)
		);
		table_elem!(
			PUSH30,
			state,
			position,
			self::misc::push(state, 30, position)
		);
		table_elem!(
			PUSH31,
			state,
			position,
			self::misc::push(state, 31, position)
		);
		table_elem!(
			PUSH32,
			state,
			position,
			self::misc::push(state, 32, position)
		);
		table_elem!(MOD, state, op2_u256_fn!(state, self::arithmetic::rem));
		table_elem!(SMOD, state, op2_u256_fn!(state, self::arithmetic::srem));
		table_elem!(CODESIZE, state, self::misc::codesize(state));
		table_elem!(CALLDATALOAD, state, self::misc::calldataload(state));
		table_elem!(CALLDATASIZE, state, self::misc::calldatasize(state));
		table_elem!(ADDMOD, state, op3_u256_fn!(state, self::arithmetic::addmod));
		table_elem!(MULMOD, state, op3_u256_fn!(state, self::arithmetic::mulmod));
		table_elem!(MLOAD, state, self::misc::mload(state));
		table_elem!(MSTORE, state, self::misc::mstore(state));
		table_elem!(MSTORE8, state, self::misc::mstore8(state));
		table_elem!(CODECOPY, state, self::misc::codecopy(state));
		table_elem!(CALLDATACOPY, state, self::misc::calldatacopy(state));
		table_elem!(DUP1, state, self::misc::dup(state, 1));
		table_elem!(DUP2, state, self::misc::dup(state, 2));
		table_elem!(DUP3, state, self::misc::dup(state, 3));
		table_elem!(DUP4, state, self::misc::dup(state, 4));
		table_elem!(DUP5, state, self::misc::dup(state, 5));
		table_elem!(DUP6, state, self::misc::dup(state, 6));
		table_elem!(DUP7, state, self::misc::dup(state, 7));
		table_elem!(DUP8, state, self::misc::dup(state, 8));
		table_elem!(DUP9, state, self::misc::dup(state, 9));
		table_elem!(DUP10, state, self::misc::dup(state, 10));
		table_elem!(DUP11, state, self::misc::dup(state, 11));
		table_elem!(DUP12, state, self::misc::dup(state, 12));
		table_elem!(DUP13, state, self::misc::dup(state, 13));
		table_elem!(DUP14, state, self::misc::dup(state, 14));
		table_elem!(DUP15, state, self::misc::dup(state, 15));
		table_elem!(DUP16, state, self::misc::dup(state, 16));
		table_elem!(SWAP1, state, self::misc::swap(state, 1));
		table_elem!(SWAP2, state, self::misc::swap(state, 2));
		table_elem!(SWAP3, state, self::misc::swap(state, 3));
		table_elem!(SWAP4, state, self::misc::swap(state, 4));
		table_elem!(SWAP5, state, self::misc::swap(state, 5));
		table_elem!(SWAP6, state, self::misc::swap(state, 6));
		table_elem!(SWAP7, state, self::misc::swap(state, 7));
		table_elem!(SWAP8, state, self::misc::swap(state, 8));
		table_elem!(SWAP9, state, self::misc::swap(state, 9));
		table_elem!(SWAP10, state, self::misc::swap(state, 10));
		table_elem!(SWAP11, state, self::misc::swap(state, 11));
		table_elem!(SWAP12, state, self::misc::swap(state, 12));
		table_elem!(SWAP13, state, self::misc::swap(state, 13));
		table_elem!(SWAP14, state, self::misc::swap(state, 14));
		table_elem!(SWAP15, state, self::misc::swap(state, 15));
		table_elem!(SWAP16, state, self::misc::swap(state, 16));
		table_elem!(RETURN, state, self::misc::ret(state));
		table_elem!(REVERT, state, self::misc::revert(state));
		table_elem!(INVALID, Control::Exit(ExitError::DesignatedInvalid.into()));
		table_elem!(STOP, Control::Exit(ExitSucceed::Stopped.into()));
		table_elem!(JUMPDEST, Control::Continue(1));
		table_elem!(JUMP, state, self::misc::jump(state));
		table_elem!(JUMPI, state, self::misc::jumpi(state));
		table
	};
	let mut pc = position;
	handler.before_eval();
	loop {
		let op = if let Some(v) = state.code.get(pc) {
			Opcode(*v)
		} else {
			state.position = Err(ExitSucceed::Stopped.into());
			return Control::Exit(ExitSucceed::Stopped.into());
		};
		match handler.before_bytecode(op, pc, state, address) {
			Ok(()) => (),
			Err(e) => {
				state.exit(e.clone().into());
				return Control::Exit(ExitReason::Error(e));
			}
		};
		let control = TABLE[op.as_usize()](state, op, pc);

		#[cfg(feature = "tracing")]
		{
			use crate::Capture;
			let result = match &control {
				Control::Continue(_) | Control::Jump(_) => Ok(()),
				Control::Trap(t) => Err(Capture::Trap(*t)),
				Control::Exit(e) => Err(Capture::Exit(e.clone())),
			};
			handler.after_bytecode(&result, state);
		}
		pc = match control {
			Control::Continue(bytes) => pc + bytes,
			Control::Jump(pos) => pos,
			_ => {
				handler.after_eval();
				return control;
			}
		}
	}
}
