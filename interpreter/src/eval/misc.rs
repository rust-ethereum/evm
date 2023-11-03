use super::Control;
use crate::{ExitError, ExitException, ExitFatal, ExitSucceed, Machine};
use core::cmp::min;
use primitive_types::{H256, U256};

#[inline]
pub fn codesize<S>(state: &mut Machine<S>) -> Control {
	let size = U256::from(state.code.len());
	push_u256!(state, size);
	Control::Continue
}

#[inline]
pub fn codecopy<S>(state: &mut Machine<S>) -> Control {
	pop_u256!(state, memory_offset, code_offset, len);

	try_or_fail!(state.memory.resize_offset(memory_offset, len));
	match state
		.memory
		.copy_large(memory_offset, code_offset, len, &state.code)
	{
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

#[inline]
pub fn calldataload<S>(state: &mut Machine<S>) -> Control {
	pop_u256!(state, index);

	let mut load = [0u8; 32];
	#[allow(clippy::needless_range_loop)]
	for i in 0..32 {
		if let Some(p) = index.checked_add(U256::from(i)) {
			if p <= U256::from(usize::MAX) {
				let p = p.as_usize();
				if p < state.data.len() {
					load[i] = state.data[p];
				}
			}
		}
	}

	push!(state, H256::from(load));
	Control::Continue
}

#[inline]
pub fn calldatasize<S>(state: &mut Machine<S>) -> Control {
	let len = U256::from(state.data.len());
	push_u256!(state, len);
	Control::Continue
}

#[inline]
pub fn calldatacopy<S>(state: &mut Machine<S>) -> Control {
	pop_u256!(state, memory_offset, data_offset, len);

	try_or_fail!(state.memory.resize_offset(memory_offset, len));
	if len == U256::zero() {
		return Control::Continue;
	}

	match state
		.memory
		.copy_large(memory_offset, data_offset, len, &state.data)
	{
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

#[inline]
pub fn pop<S>(state: &mut Machine<S>) -> Control {
	pop!(state, _val);
	Control::Continue
}

#[inline]
pub fn mload<S>(state: &mut Machine<S>) -> Control {
	pop_u256!(state, index);
	try_or_fail!(state.memory.resize_offset(index, U256::from(32)));
	let index = as_usize_or_fail!(index);
	let value = H256::from_slice(&state.memory.get(index, 32)[..]);
	push!(state, value);
	Control::Continue
}

#[inline]
pub fn mstore<S>(state: &mut Machine<S>) -> Control {
	pop_u256!(state, index);
	pop!(state, value);
	try_or_fail!(state.memory.resize_offset(index, U256::from(32)));
	let index = as_usize_or_fail!(index);
	match state.memory.set(index, &value[..], Some(32)) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

#[inline]
pub fn mstore8<S>(state: &mut Machine<S>) -> Control {
	pop_u256!(state, index, value);
	try_or_fail!(state.memory.resize_offset(index, U256::one()));
	let index = as_usize_or_fail!(index);
	let value = (value.low_u32() & 0xff) as u8;
	match state.memory.set(index, &[value], Some(1)) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

#[inline]
pub fn jump<S>(state: &mut Machine<S>) -> Control {
	pop_u256!(state, dest);
	let dest = as_usize_or_fail!(dest, ExitException::InvalidJump);

	if state.valids.is_valid(dest) {
		Control::Jump(dest)
	} else {
		Control::Exit(ExitException::InvalidJump.into())
	}
}

#[inline]
pub fn jumpi<S>(state: &mut Machine<S>) -> Control {
	pop_u256!(state, dest);
	pop!(state, value);

	if value != H256::zero() {
		let dest = as_usize_or_fail!(dest, ExitException::InvalidJump);
		if state.valids.is_valid(dest) {
			Control::Jump(dest)
		} else {
			Control::Exit(ExitException::InvalidJump.into())
		}
	} else {
		Control::Continue
	}
}

#[inline]
pub fn pc<S>(state: &mut Machine<S>, position: usize) -> Control {
	push_u256!(state, U256::from(position));
	Control::Continue
}

#[inline]
pub fn msize<S>(state: &mut Machine<S>) -> Control {
	push_u256!(state, state.memory.effective_len());
	Control::Continue
}

#[inline]
pub fn push<S>(state: &mut Machine<S>, n: usize, position: usize) -> Control {
	let end = min(position + 1 + n, state.code.len());
	let slice = &state.code[(position + 1)..end];
	let mut val = [0u8; 32];
	val[(32 - n)..(32 - n + slice.len())].copy_from_slice(slice);

	let result = H256(val);
	push!(state, result);
	Control::ContinueN(1 + n)
}

#[inline]
pub fn dup<S>(state: &mut Machine<S>, n: usize) -> Control {
	let value = match state.stack.peek(n - 1) {
		Ok(value) => value,
		Err(e) => return Control::Exit(e.into()),
	};
	push!(state, value);
	Control::Continue
}

#[inline]
pub fn swap<S>(state: &mut Machine<S>, n: usize) -> Control {
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
	Control::Continue
}

#[inline]
pub fn ret<S>(state: &mut Machine<S>) -> Control {
	pop_u256!(state, start, len);
	try_or_fail!(state.memory.resize_offset(start, len));
	state.memory.resize_to_range(start..(start + len));
	Control::Exit(ExitSucceed::Returned.into())
}

#[inline]
pub fn revert<S>(state: &mut Machine<S>) -> Control {
	pop_u256!(state, start, len);
	try_or_fail!(state.memory.resize_offset(start, len));
	state.memory.resize_to_range(start..(start + len));
	Control::Exit(ExitError::Reverted.into())
}
