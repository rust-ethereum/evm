use super::Control;
use crate::{utils::AdvangeManagedVec, ExitError, ExitFatal, ExitRevert, ExitSucceed, Machine};
use core::cmp::min;
use elrond_wasm::{api::ManagedTypeApi, types::ManagedVec};
use eltypes::{ManagedVecforEH256, ToEH256};
use primitive_types::{H256, U256};

#[inline]
pub fn codesize<M: ManagedTypeApi>(state: &mut Machine<M>) -> Control {
	let size = U256::from(state.code.len());
	push_u256!(state, size);
	Control::Continue(1)
}

#[inline]
pub fn codecopy<M: ManagedTypeApi>(state: &mut Machine<M>) -> Control {
	pop_u256!(state, memory_offset, code_offset, len);

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
pub fn calldataload<M: ManagedTypeApi>(state: &mut Machine<M>) -> Control {
	pop_u256!(state, index);

	let mut load = [0u8; 32];
	#[allow(clippy::needless_range_loop)]
	for i in 0..32 {
		if let Some(p) = index.checked_add(U256::from(i)) {
			if p <= U256::from(usize::MAX) {
				let p = p.as_usize();
				if p < state.data.len() {
					load[i] = state.data.get(p);
				}
			}
		}
	}

	push!(state, H256::from(load).to_eh256());
	Control::Continue(1)
}

#[inline]
pub fn calldatasize<M: ManagedTypeApi>(state: &mut Machine<M>) -> Control {
	let len = U256::from(state.data.len());
	push_u256!(state, len);
	Control::Continue(1)
}

#[inline]
pub fn calldatacopy<M: ManagedTypeApi>(state: &mut Machine<M>) -> Control {
	pop_u256!(state, memory_offset, data_offset, len);

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
pub fn pop<M: ManagedTypeApi>(state: &mut Machine<M>) -> Control {
	pop!(state, _val);
	Control::Continue(1)
}

#[inline]
pub fn mload<M: ManagedTypeApi>(state: &mut Machine<M>) -> Control {
	pop_u256!(state, index);
	try_or_fail!(state.memory.resize_offset(index, U256::from(32)));
	let index = as_usize_or_fail!(index);
	let value = H256::from_slice(&state.memory.get(index, 32).as_bytes());
	push!(state, value.to_eh256());
	Control::Continue(1)
}

#[inline]
pub fn mstore<M: ManagedTypeApi>(state: &mut Machine<M>) -> Control {
	pop_u256!(state, index);
	pop!(state, value);
	try_or_fail!(state.memory.resize_offset(index, U256::from(32)));
	let index = as_usize_or_fail!(index);
	match state.memory.set(index, &value.managedvec_bytes(), Some(32)) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

#[inline]
pub fn mstore8<M: ManagedTypeApi>(state: &mut Machine<M>) -> Control {
	pop_u256!(state, index, value);
	try_or_fail!(state.memory.resize_offset(index, U256::one()));
	let index = as_usize_or_fail!(index);
	let value = (value.low_u32() & 0xff) as u8;
	let vec = ManagedVec::new();
	vec.push(value);
	match state.memory.set(index, &vec, Some(1)) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

#[inline]
pub fn jump<M: ManagedTypeApi>(state: &mut Machine<M>) -> Control {
	pop_u256!(state, dest);
	let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);

	if state.valids.is_valid(dest) {
		Control::Jump(dest)
	} else {
		Control::Exit(ExitError::InvalidJump.into())
	}
}

#[inline]
pub fn jumpi<M: ManagedTypeApi>(state: &mut Machine<M>) -> Control {
	pop_u256!(state, dest);
	pop!(state, value);

	if value.to_h256() != H256::zero() {
		let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);
		if state.valids.is_valid(dest) {
			Control::Jump(dest)
		} else {
			Control::Exit(ExitError::InvalidJump.into())
		}
	} else {
		Control::Continue(1)
	}
}

#[inline]
pub fn pc<M: ManagedTypeApi>(state: &mut Machine<M>, position: usize) -> Control {
	push_u256!(state, U256::from(position));
	Control::Continue(1)
}

#[inline]
pub fn msize<M: ManagedTypeApi>(state: &mut Machine<M>) -> Control {
	push_u256!(state, state.memory.effective_len());
	Control::Continue(1)
}

#[inline]
pub fn push<M: ManagedTypeApi>(state: &mut Machine<M>, n: usize, position: usize) -> Control {
	let end = min(position + 1 + n, state.code.len());
	let slice = &state.code.slice(position + 1, end).unwrap();
	let mut val = [0u8; 32];

	for i in 0..slice.len() {
		val[i] = slice.get(i);
	}
	push!(state, eltypes::EH256 { data: val });
	Control::Continue(1 + n)
}

#[inline]
pub fn dup<M: ManagedTypeApi>(state: &mut Machine<M>, n: usize) -> Control {
	let value = match state.stack.peek(n - 1) {
		Ok(value) => value,
		Err(e) => return Control::Exit(e.into()),
	};
	push!(state, value);
	Control::Continue(1)
}

#[inline]
pub fn swap<M: ManagedTypeApi>(state: &mut Machine<M>, n: usize) -> Control {
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
pub fn ret<M: ManagedTypeApi>(state: &mut Machine<M>) -> Control {
	pop_u256!(state, start, len);
	try_or_fail!(state.memory.resize_offset(start, len));
	state.return_range = start..(start + len);
	Control::Exit(ExitSucceed::Returned.into())
}

#[inline]
pub fn revert<M: ManagedTypeApi>(state: &mut Machine<M>) -> Control {
	pop_u256!(state, start, len);
	try_or_fail!(state.memory.resize_offset(start, len));
	state.return_range = start..(start + len);
	Control::Exit(ExitRevert::Reverted.into())
}
