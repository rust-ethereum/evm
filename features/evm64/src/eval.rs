use core::ops::{BitAnd, BitOr, BitXor};

use evm::interpreter::{Control, ExitException, Machine};
use primitive_types::U256;

macro_rules! pop_u64 {
	( $machine:expr, $( $x:ident ),* ) => (
		$(
			let $x = match $machine.stack.pop() {
				Ok(value) => value.0[0],
				Err(e) => return Control::Exit(e.into()),
			};
		)*
	);
}

macro_rules! push_u64 {
	( $machine:expr, $( $x:expr ),* ) => (
		$(
			match $machine.stack.push(U256([$x, 0, 0, 0])) {
				Ok(()) => (),
				Err(e) => return Control::Exit(e.into()),
			}
		)*
	)
}

macro_rules! op1_u64_fn {
	($machine:expr, $op:path) => {{
		pop_u64!($machine, op1);
		let ret = $op(op1);
		push_u64!($machine, ret);

		Control::Continue(1)
	}};
}

macro_rules! op2_u64_bool_ref {
	($machine:expr, $op:ident) => {{
		pop_u64!($machine, op1, op2);
		let ret = op1.$op(&op2);
		push_u64!($machine, if ret { 1 } else { 0 });

		Control::Continue(1)
	}};
}

macro_rules! op2_u64 {
	($machine:expr, $op:ident) => {{
		pop_u64!($machine, op1, op2);
		let ret = op1.$op(op2);
		push_u64!($machine, ret);

		Control::Continue(1)
	}};
}

macro_rules! op2_u64_tuple {
	($machine:expr, $op:ident) => {{
		pop_u64!($machine, op1, op2);
		let (ret, ..) = op1.$op(op2);
		push_u64!($machine, ret);

		Control::Continue(1)
	}};
}

macro_rules! op2_u64_fn {
	($machine:expr, $op:path) => {{
		pop_u64!($machine, op1, op2);
		let ret = $op(op1, op2);
		push_u64!($machine, ret);

		Control::Continue(1)
	}};
}

macro_rules! op3_u64_fn {
	($machine:expr, $op:path) => {{
		pop_u64!($machine, op1, op2, op3);
		let ret = $op(op1, op2, op3);
		push_u64!($machine, ret);

		Control::Continue(1)
	}};
}

pub fn eval_add<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_tuple!(machine, overflowing_add)
}

pub fn eval_mul<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_tuple!(machine, overflowing_mul)
}

pub fn eval_sub<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_tuple!(machine, overflowing_sub)
}

#[inline]
fn div(op1: u64, op2: u64) -> u64 {
	if op2 == 0 {
		0
	} else {
		op1 / op2
	}
}

pub fn eval_div<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_fn!(machine, div)
}

#[inline]
pub fn sdiv(op1: u64, op2: u64) -> u64 {
	let op1 = op1 as i64;
	let op2 = op2 as i64;
	let ret = op1 / op2;
	ret as u64
}

pub fn eval_sdiv<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_fn!(machine, sdiv)
}

#[inline]
pub fn rem(op1: u64, op2: u64) -> u64 {
	if op2 == 0 {
		0
	} else {
		// For unsigned integers overflow never occurs.
		op1.overflowing_rem(op2).0
	}
}

pub fn eval_mod<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_fn!(machine, rem)
}

#[inline]
pub fn srem(op1: u64, op2: u64) -> u64 {
	if op2 == 0 {
		0
	} else {
		let op1 = op1 as i64;
		let op2 = op2 as i64;
		let ret = op1.overflowing_rem(op2).0;
		ret as u64
	}
}

pub fn eval_smod<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_fn!(machine, srem)
}

#[inline]
pub fn addmod(op1: u64, op2: u64, op3: u64) -> u64 {
	let op1 = op1 as u128;
	let op2 = op2 as u128;
	let op3 = op3 as u128;

	if op3 == 0 {
		0
	} else {
		let v = (op1 + op2) % op3;
		v as u64
	}
}

pub fn eval_addmod<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op3_u64_fn!(machine, addmod)
}

#[inline]
pub fn mulmod(op1: u64, op2: u64, op3: u64) -> u64 {
	let op1 = op1 as u128;
	let op2 = op2 as u128;
	let op3 = op3 as u128;

	if op3 == 0 {
		0
	} else {
		let v = (op1 * op2) % op3;
		v as u64
	}
}

pub fn eval_mulmod<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op3_u64_fn!(machine, mulmod)
}

#[inline]
pub fn exp(op1: u64, op2: u64) -> u64 {
	let mut op1 = op1;
	let mut op2 = op2;
	let mut r = 1u64;

	while op2 != 0 {
		if op2 & 1 != 0 {
			r = r.overflowing_mul(op1).0;
		}
		op2 >>= 1;
		op1 = op1.overflowing_mul(op1).0;
	}

	r
}

pub fn eval_exp<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_fn!(machine, exp)
}

pub fn eval_lt<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_bool_ref!(machine, lt)
}

pub fn eval_gt<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_bool_ref!(machine, gt)
}

#[inline]
pub fn slt(op1: u64, op2: u64) -> u64 {
	let op1 = op1 as i64;
	let op2 = op2 as i64;

	if op1.lt(&op2) {
		1
	} else {
		0
	}
}

pub fn eval_slt<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_fn!(machine, slt)
}

#[inline]
pub fn sgt(op1: u64, op2: u64) -> u64 {
	let op1 = op1 as i64;
	let op2 = op2 as i64;

	if op1.gt(&op2) {
		1
	} else {
		0
	}
}

pub fn eval_sgt<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_fn!(machine, sgt)
}

pub fn eval_eq<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_bool_ref!(machine, eq)
}

#[inline]
pub fn iszero(op1: u64) -> u64 {
	if op1 == 0 {
		1
	} else {
		0
	}
}

pub fn eval_iszero<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op1_u64_fn!(machine, iszero)
}

pub fn eval_and<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64!(machine, bitand)
}

pub fn eval_or<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64!(machine, bitor)
}

pub fn eval_xor<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64!(machine, bitxor)
}

#[inline]
pub fn not(op1: u64) -> u64 {
	!op1
}

pub fn eval_not<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op1_u64_fn!(machine, not)
}

#[inline]
pub fn shl(shift: u64, value: u64) -> u64 {
	if value == 0 || shift >= 64 {
		0
	} else {
		value << shift as usize
	}
}

pub fn eval_shl<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_fn!(machine, shl)
}

#[inline]
pub fn shr(shift: u64, value: u64) -> u64 {
	if value == 0 || shift >= 64 {
		0
	} else {
		value >> shift as usize
	}
}

pub fn eval_shr<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_fn!(machine, shr)
}

#[inline]
pub fn sar(shift: u64, value: u64) -> u64 {
	let value = value as i64;

	let ret = if value == 0 || shift >= 64 {
		if value >= 0 {
			0
		} else {
			-1
		}
	} else {
		value >> shift as usize
	};

	ret as u64
}

pub fn eval_sar<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	op2_u64_fn!(machine, sar)
}

pub fn eval_jump<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	pop_u64!(machine, dest);

	if let Ok(dest) = usize::try_from(dest) {
		Control::Jump(dest)
	} else {
		Control::Exit(ExitException::InvalidJump.into())
	}
}

pub fn eval_jumpi<S, H, Tr>(
	machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	pop_u64!(machine, dest, value);

	if value == 0 {
		Control::Continue(1)
	} else if let Ok(dest) = usize::try_from(dest) {
		Control::Jump(dest)
	} else {
		Control::Exit(ExitException::InvalidJump.into())
	}
}
