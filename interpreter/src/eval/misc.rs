use core::cmp::{max, min};

use crate::uint::{H256, U256, U256Ext};
use crate::{Control, ExitError, ExitException, ExitFatal, ExitSucceed, Machine};

#[inline]
pub fn codesize<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	let stack = &mut state.stack;
	let code = &state.code;

	match stack.perform_pop0_push1(|| {
		let size = U256::from(code.len());
		Ok((size, ()))
	}) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(Err(e)),
	}
}

#[inline]
pub fn codecopy<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	let stack = &mut state.stack;
	let memory = &mut state.memory;
	let code = &state.code;

	match stack.perform_pop3_push0(|memory_offset, code_offset, len| {
		memory.copy_large(*memory_offset, *code_offset, *len, code)?;
		Ok(((), ()))
	}) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(Err(e)),
	}
}

#[inline]
pub fn calldataload<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	pop_u256!(state, index);

	let mut load = [0u8; 32];
	#[allow(clippy::needless_range_loop)]
	for i in 0..32 {
		if let Some(p) = index.checked_add(U256::from(i))
			&& p <= U256::from(usize::MAX)
		{
			let p = p.as_usize();
			if p < state.data.len() {
				load[i] = state.data[p];
			}
		}
	}

	push_h256!(state, H256::from(load));
	Control::Continue(1)
}

#[inline]
pub fn calldatasize<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	let len = U256::from(state.data.len());
	push_u256!(state, len);
	Control::Continue(1)
}

#[inline]
pub fn calldatacopy<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	pop_u256!(state, memory_offset, data_offset, len);

	try_or_fail!(state.memory.resize_offset(memory_offset, len));
	if len == U256::ZERO {
		return Control::Continue(1);
	}

	match state
		.memory
		.copy_large(memory_offset, data_offset, len, &state.data)
	{
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

#[inline]
pub fn pop<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	pop_h256!(state, _val);
	Control::Continue(1)
}

#[inline]
pub fn mload<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	pop_u256!(state, index);
	try_or_fail!(state.memory.resize_offset(index, U256::from(32)));
	let index = as_usize_or_fail!(index);
	let value = H256::from_slice(&state.memory.get(index, 32)[..]);
	push_h256!(state, value);
	Control::Continue(1)
}

/// Support for EIP-5656: MCOPY instruction.
#[inline]
pub fn mcopy<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	pop_u256!(state, dst, src, len);
	try_or_fail!(state.memory.resize_offset(max(dst, src), len));

	if len.is_zero() {
		return Control::Continue(1);
	}

	let dst = as_usize_or_fail!(dst);
	let src = as_usize_or_fail!(src);
	let len = as_usize_or_fail!(len);
	state.memory.copy(dst, src, len);
	Control::Continue(1)
}

#[inline]
pub fn mstore<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	pop_u256!(state, index);
	pop_h256!(state, value);
	try_or_fail!(state.memory.resize_offset(index, U256::from(32)));
	let index = as_usize_or_fail!(index);
	match state.memory.set(index, &value[..], Some(32)) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

#[inline]
pub fn mstore8<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	pop_u256!(state, index, value);
	try_or_fail!(state.memory.resize_offset(index, U256::ONE));
	let index = as_usize_or_fail!(index);
	let value = (value.low_u32() & 0xff) as u8;
	match state.memory.set(index, &[value], Some(1)) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

#[inline]
pub fn jump<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	pop_u256!(state, dest);
	let dest = as_usize_or_fail!(dest, ExitException::InvalidJump);

	Control::Jump(dest)
}

#[inline]
pub fn jumpi<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	pop_u256!(state, dest);
	pop_h256!(state, value);

	if value == H256::zero() {
		Control::Continue(1)
	} else {
		let dest = as_usize_or_fail!(dest, ExitException::InvalidJump);
		Control::Jump(dest)
	}
}

#[inline]
pub fn pc<S, Tr>(state: &mut Machine<S>, position: usize) -> Control<Tr> {
	push_u256!(state, U256::from(position));
	Control::Continue(1)
}

#[inline]
pub fn msize<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	push_u256!(state, state.memory.effective_len());
	Control::Continue(1)
}

#[inline]
pub fn push<S, Tr>(state: &mut Machine<S>, n: usize, position: usize) -> Control<Tr> {
	let end = min(position + 1 + n, state.code.len());
	let slice = &state.code[(position + 1)..end];
	let mut val = [0u8; 32];
	val[(32 - n)..(32 - n + slice.len())].copy_from_slice(slice);

	let result = H256(val);
	push_h256!(state, result);
	Control::Continue(1 + n)
}

#[inline]
pub fn dup<S, Tr>(state: &mut Machine<S>, n: usize) -> Control<Tr> {
	let value = match state.stack.peek(n - 1) {
		Ok(value) => value,
		Err(e) => return Control::Exit(e.into()),
	};
	push_u256!(state, value);
	Control::Continue(1)
}

#[inline]
pub fn swap<S, Tr>(state: &mut Machine<S>, n: usize) -> Control<Tr> {
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
pub fn ret<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	pop_u256!(state, start, len);
	try_or_fail!(state.memory.resize_offset(start, len));
	state.memory.resize_to_range(start..(start + len));
	state.memory.swap_and_clear(&mut state.retval);
	Control::Exit(ExitSucceed::Returned.into())
}

#[inline]
pub fn revert<S, Tr>(state: &mut Machine<S>) -> Control<Tr> {
	pop_u256!(state, start, len);
	try_or_fail!(state.memory.resize_offset(start, len));
	state.memory.resize_to_range(start..(start + len));
	state.memory.swap_and_clear(&mut state.retval);
	Control::Exit(ExitError::Reverted.into())
}
