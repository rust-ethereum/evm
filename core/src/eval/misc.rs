use super::Control;
use crate::utils::USIZE_MAX;
use crate::{ExitError, ExitRevert, ExitSucceed, Machine};
use core::cmp::min;
use primitive_types::{H256, U256};

#[inline]
pub fn codesize(state: &mut Machine) -> Control {
	let size = U256::from(state.code.len());
	push_u256!(state, size);
	Control::Continue(1)
}

#[inline]
pub fn codecopy(state: &mut Machine) -> Control {
	pop_u256!(state, memory_offset);
	pop_u256!(state, code_offset);
	pop_usize!(state, len);

	// If `len` is zero then nothing happens, regardless of the
	// value of the other parameters. In particular, `memory_offset`
	// might be larger than `usize::MAX`, hence why we check this first.
	if len == 0 {
		return Control::Continue(1);
	}

	let memory_offset = memory_offset.as_usize();

	try_or_fail!(state.memory.resize_offset(memory_offset, len));
	match state
		.memory
		.copy_large(memory_offset, code_offset, len, &state.code)
	{
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

#[inline]
pub fn calldataload(state: &mut Machine) -> Control {
	pop_u256!(state, index);

	let mut load = [0u8; 32];
	#[allow(clippy::needless_range_loop)]
	for i in 0..32 {
		if let Some(p) = index.checked_add(U256::from(i)) {
			if p <= USIZE_MAX {
				let p = p.as_usize();
				if p < state.data.len() {
					load[i] = state.data[p];
				}
			}
		}
	}

	push_h256!(state, H256::from(load));
	Control::Continue(1)
}

#[inline]
pub fn calldatasize(state: &mut Machine) -> Control {
	let len = U256::from(state.data.len());
	push_u256!(state, len);
	Control::Continue(1)
}

#[inline]
pub fn calldatacopy(state: &mut Machine) -> Control {
	pop_u256!(state, memory_offset);
	pop_u256!(state, data_offset);
	pop_usize!(state, len);

	// See comment on `codecopy` about the `len == 0` case.
	if len == 0 {
		return Control::Continue(1);
	}
	let memory_offset = memory_offset.as_usize();
	try_or_fail!(state.memory.resize_offset(memory_offset, len));

	match state
		.memory
		.copy_large(memory_offset, data_offset, len, &state.data)
	{
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

#[inline]
pub fn pop(state: &mut Machine) -> Control {
	pop_u256!(state, _val);
	Control::Continue(1)
}

#[inline]
pub fn mload(state: &mut Machine) -> Control {
	pop_usize!(state, index);
	try_or_fail!(state.memory.resize_offset(index, 32));
	let value = state.memory.get_h256(index);
	push_h256!(state, value);
	Control::Continue(1)
}

#[inline]
pub fn mstore(state: &mut Machine) -> Control {
	pop_usize!(state, index);
	pop_h256!(state, value);
	try_or_fail!(state.memory.resize_offset(index, 32));
	match state.memory.set(index, &value[..], Some(32)) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

#[inline]
pub fn mstore8(state: &mut Machine) -> Control {
	pop_usize!(state, index);
	pop_u256!(state, value);
	try_or_fail!(state.memory.resize_offset(index, 1));
	let value = (value.low_u32() & 0xff) as u8;
	match state.memory.set(index, &[value], Some(1)) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

#[inline]
pub fn jump(state: &mut Machine) -> Control {
	pop_u256!(state, dest);
	let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);

	if state.valids.is_valid(dest) {
		Control::Jump(dest)
	} else {
		Control::Exit(ExitError::InvalidJump.into())
	}
}

#[inline]
pub fn jumpi(state: &mut Machine) -> Control {
	pop_u256!(state, dest);
	pop_u256!(state, value);

	if value == U256::zero() {
		Control::Continue(1)
	} else {
		let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);
		if state.valids.is_valid(dest) {
			Control::Jump(dest)
		} else {
			Control::Exit(ExitError::InvalidJump.into())
		}
	}
}

#[inline]
pub fn pc(state: &mut Machine, position: usize) -> Control {
	push_u256!(state, U256::from(position));
	Control::Continue(1)
}

#[inline]
pub fn msize(state: &mut Machine) -> Control {
	push_u256!(state, state.memory.effective_len().into());
	Control::Continue(1)
}

#[inline]
pub fn push(state: &mut Machine, n: usize, position: usize) -> Control {
	let end = min(position + 1 + n, state.code.len());
	let slice = &state.code[(position + 1)..end];
	let mut val = [0u8; 32];
	val[(32 - n)..(32 - n + slice.len())].copy_from_slice(slice);
	let val = U256::from_big_endian(&val);

	push_u256!(state, val);
	Control::Continue(1 + n)
}

#[inline]
pub fn push0(state: &mut Machine) -> Control {
	let val = U256::zero();

	push_u256!(state, val);
	Control::Continue(1)
}

#[inline]
pub fn push1(state: &mut Machine, position: usize) -> Control {
	let b0 = u64::from(*state.code.get(position + 1).unwrap_or(&0));
	let val = U256::from(b0);

	push_u256!(state, val);
	Control::Continue(2)
}

#[inline]
pub fn push2(state: &mut Machine, position: usize) -> Control {
	let b0 = u64::from(*state.code.get(position + 1).unwrap_or(&0));
	let b1 = u64::from(*state.code.get(position + 2).unwrap_or(&0));
	let val = U256::from((b0 << 8) | b1);

	push_u256!(state, val);
	Control::Continue(3)
}

#[inline]
pub fn dup(state: &mut Machine, n: usize) -> Control {
	let value = match state.stack.peek(n - 1) {
		Ok(value) => value,
		Err(e) => return Control::Exit(e.into()),
	};
	push_u256!(state, value);
	Control::Continue(1)
}

#[inline]
pub fn swap(state: &mut Machine, n: usize) -> Control {
	let val1 = match state.stack.peek(0) {
		Ok(value) => value,
		Err(e) => return Control::Exit(e.into()),
	};
	let val2 = match state.stack.peek(n) {
		Ok(value) => value,
		Err(e) => return Control::Exit(e.into()),
	};
	match state.stack.set(0, val2) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	}
	match state.stack.set(n, val1) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	}
	Control::Continue(1)
}

#[inline]
pub fn ret(state: &mut Machine) -> Control {
	pop_u256!(state, start);
	pop_usize!(state, len);
	if len > 0 {
		try_or_fail!(state.memory.resize_offset(start.as_usize(), len));
	}
	state.return_range = start..(start + U256::from(len));
	Control::Exit(ExitSucceed::Returned.into())
}

#[inline]
pub fn revert(state: &mut Machine) -> Control {
	pop_u256!(state, start);
	pop_usize!(state, len);
	if len > 0 {
		try_or_fail!(state.memory.resize_offset(start.as_usize(), len));
	}
	state.return_range = start..(start + U256::from(len));
	Control::Exit(ExitRevert::Reverted.into())
}
