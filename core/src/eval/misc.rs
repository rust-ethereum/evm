use super::Control;
use crate::{ExitError, ExitFatal, ExitRevert, ExitSucceed, Machine};
use core::cmp::{max, min};
use primitive_types::{H256, U256};

#[inline]
pub fn codesize(state: &mut Machine) -> Control {
	let size = U256::from(state.code.len());
	trace_op!("CodeSize: {}", size);
	push_u256!(state, size);
	Control::Continue(1)
}

#[inline]
pub fn codecopy(state: &mut Machine) -> Control {
	pop_u256!(state, memory_offset, code_offset, len);
	trace_op!("CodeCopy: {}", len);

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
			if p <= U256::from(usize::MAX) {
				let p = p.as_usize();
				if p < state.data.len() {
					load[i] = state.data[p];
				}
			}
		}
	}

	push!(state, H256::from(load));
	Control::Continue(1)
}

#[inline]
pub fn calldatasize(state: &mut Machine) -> Control {
	let len = U256::from(state.data.len());
	trace_op!("CallDataSize: {}", len);
	push_u256!(state, len);
	Control::Continue(1)
}

#[inline]
pub fn calldatacopy(state: &mut Machine) -> Control {
	pop_u256!(state, memory_offset, data_offset, len);
	trace_op!("CallDataCopy: {}", len);

	try_or_fail!(state.memory.resize_offset(memory_offset, len));
	if len == U256::zero() {
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
pub fn pop(state: &mut Machine) -> Control {
	pop!(state, _val);
	trace_op!("Pop [@{}]: {}", state.stack.len(), _val);
	Control::Continue(1)
}

#[inline]
pub fn mload(state: &mut Machine) -> Control {
	pop_u256!(state, index);
	trace_op!("MLoad: {}", index);
	try_or_fail!(state.memory.resize_offset(index, U256::from(32)));
	let index = as_usize_or_fail!(index);
	let value = H256::from_slice(&state.memory.get(index, 32)[..]);
	push!(state, value);
	Control::Continue(1)
}

/// Support for EIP-5656: MCOPY instruction.
#[inline]
pub fn mcopy(state: &mut Machine) -> Control {
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
pub fn mstore(state: &mut Machine) -> Control {
	pop_u256!(state, index);
	pop!(state, value);
	trace_op!("MStore: {}, {}", index, value);
	try_or_fail!(state.memory.resize_offset(index, U256::from(32)));
	let index = as_usize_or_fail!(index);
	match state.memory.set(index, &value[..], Some(32)) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

#[inline]
pub fn mstore8(state: &mut Machine) -> Control {
	pop_u256!(state, index, value);
	try_or_fail!(state.memory.resize_offset(index, U256::one()));
	let index = as_usize_or_fail!(index);
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
	trace_op!("Jump: {}", dest);

	if state.valids.is_valid(dest) {
		Control::Jump(dest)
	} else {
		Control::Exit(ExitError::InvalidJump.into())
	}
}

#[inline]
pub fn jumpi(state: &mut Machine) -> Control {
	pop_u256!(state, dest);
	pop!(state, value);

	if value != H256::zero() {
		trace_op!("JumpI: {}", dest);
		let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);
		if state.valids.is_valid(dest) {
			Control::Jump(dest)
		} else {
			Control::Exit(ExitError::InvalidJump.into())
		}
	} else {
		trace_op!("JumpI: skipped");
		Control::Continue(1)
	}
}

#[inline]
pub fn pc(state: &mut Machine, position: usize) -> Control {
	trace_op!("PC");
	push_u256!(state, U256::from(position));
	Control::Continue(1)
}

#[inline]
pub fn msize(state: &mut Machine) -> Control {
	push_u256!(state, state.memory.effective_len());
	Control::Continue(1)
}

#[inline]
pub fn push(state: &mut Machine, n: usize, position: usize) -> Control {
	let end = min(position + 1 + n, state.code.len());
	let slice = &state.code[(position + 1)..end];
	let mut val = [0u8; 32];
	val[(32 - n)..(32 - n + slice.len())].copy_from_slice(slice);

	let result = H256(val);
	push!(state, result);
	trace_op!("Push [@{}]: {}", state.stack.len() - 1, result);
	Control::Continue(1 + n)
}

#[inline]
pub fn dup(state: &mut Machine, n: usize) -> Control {
	let value = match state.stack.peek(n - 1) {
		Ok(value) => value,
		Err(e) => return Control::Exit(e.into()),
	};
	trace_op!("Dup{} [@{}]: {}", n, state.stack.len(), value);
	push!(state, value);
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
	trace_op!("Swap [@0:@{}]: {}, {}", n, val1, val2);
	Control::Continue(1)
}

#[inline]
pub fn ret(state: &mut Machine) -> Control {
	trace_op!("Return");
	pop_u256!(state, start, len);
	try_or_fail!(state.memory.resize_offset(start, len));
	state.return_range = start..(start + len);
	Control::Exit(ExitSucceed::Returned.into())
}

#[inline]
pub fn revert(state: &mut Machine) -> Control {
	trace_op!("Revert");
	pop_u256!(state, start, len);
	try_or_fail!(state.memory.resize_offset(start, len));
	state.return_range = start..(start + len);
	Control::Exit(ExitRevert::Reverted.into())
}
