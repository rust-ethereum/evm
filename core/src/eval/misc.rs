use primitive_types::{H256, U256};
use super::Control;
use crate::{Machine, ExitError, ExitSucceed};

pub fn codesize(state: &mut Machine) -> Control {
	let size = U256::from(state.code.len());
	push_u256!(state, size);
	Control::Continue(1)
}

pub fn codecopy(state: &mut Machine) -> Control {
	pop_u256!(state, memory_offset, code_offset, len);

	match state.memory.copy_large(memory_offset, code_offset, len, &state.code) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(Err(e)),
	}
}

pub fn calldataload(state: &mut Machine) -> Control {
	pop_u256!(state, index);

	let mut load = [0u8; 32];
	for i in 0..32 {
		if let Some(p) = index.checked_add(U256::from(i)) {
			if p <= U256::from(usize::max_value()) {
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

pub fn calldatasize(state: &mut Machine) -> Control {
	push_u256!(state, U256::from(state.data.len()));
	Control::Continue(1)
}

pub fn calldatacopy(state: &mut Machine) -> Control {
	pop_u256!(state, memory_offset, data_offset, len);

	let memory_offset = as_usize_or_fail!(memory_offset);
	let ulen = as_usize_or_fail!(len);

	let data = if let Some(end) = data_offset.checked_add(len) {
		if end > U256::from(usize::max_value()) {
			&[]
		} else {
			let data_offset = data_offset.as_usize();
			let end = end.as_usize();

			if end > state.data.len() {
				&[]
			} else {
				&state.data[data_offset..end]
			}
		}
	} else {
		&[]
	};

	match state.memory.set(memory_offset, data, Some(ulen)) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(Err(e)),
	}
}

pub fn pop(state: &mut Machine) -> Control {
	pop!(state, _any);
	Control::Continue(1)
}

pub fn mload(state: &mut Machine) -> Control {
	pop_u256!(state, index);
	let index = as_usize_or_fail!(index);
	let value = H256::from_slice(&state.memory.get(index, 32)[..]);
	push!(state, value);
	Control::Continue(1)
}

pub fn mstore(state: &mut Machine) -> Control {
	pop_u256!(state, index);
	pop!(state, value);
	let index = as_usize_or_fail!(index);
	match state.memory.set(index, &value[..], Some(32)) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(Err(e)),
	}
}

pub fn mstore8(state: &mut Machine) -> Control {
	pop_u256!(state, index, value);
	let index = as_usize_or_fail!(index);
	let value = (value.low_u32() & 0xff) as u8;
	match state.memory.set(index, &[value], Some(1)) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(Err(e)),
	}
}

pub fn jump(state: &mut Machine) -> Control {
	pop_u256!(state, dest);
	let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);
	Control::Jump(dest)
}

pub fn jumpi(state: &mut Machine) -> Control {
	pop_u256!(state, dest, value);
	let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);
	if value != U256::zero() {
		Control::Jump(dest)
	} else {
		Control::Continue(1)
	}
}

pub fn pc(state: &mut Machine, position: usize) -> Control {
	push_u256!(state, U256::from(position));
	Control::Continue(1)
}

pub fn msize(state: &mut Machine) -> Control {
	push_u256!(state, U256::from(state.memory.len()));
	Control::Continue(1)
}

pub fn push(state: &mut Machine, n: usize, position: usize) -> Control {
	let end = position + 1 + n;
	if end > state.code.len() {
		return Control::Exit(Err(ExitError::PCUnderflow))
	}

	push_u256!(state, U256::from(&state.code[(position + 1)..(position + 1 + n)]));
	Control::Continue(1 + n)
}

pub fn dup(state: &mut Machine, n: usize) -> Control {
	let value = match state.stack.peek(n - 1) {
		Ok(value) => value,
		Err(e) => return Control::Exit(Err(e)),
	};
	push!(state, value);
	Control::Continue(1)
}

pub fn swap(state: &mut Machine, n: usize) -> Control {
	let val1 = match state.stack.peek(0) {
		Ok(value) => value,
		Err(e) => return Control::Exit(Err(e)),
	};
	let val2 = match state.stack.peek(n) {
		Ok(value) => value,
		Err(e) => return Control::Exit(Err(e)),
	};
	match state.stack.set(0, val2) {
		Ok(()) => (),
		Err(e) => return Control::Exit(Err(e)),
	}
	match state.stack.set(n, val1) {
		Ok(()) => (),
		Err(e) => return Control::Exit(Err(e)),
	}
	Control::Continue(1)
}

pub fn ret(state: &mut Machine) -> Control {
	pop_u256!(state, start, len);
	if let Some(end) = start.checked_add(len) {
		state.return_range = start..end;
		Control::Exit(Ok(ExitSucceed::Returned))
	} else {
		Control::Exit(Err(ExitError::InvalidReturnRange))
	}
}

pub fn revert(state: &mut Machine) -> Control {
	pop_u256!(state, start, len);
	if let Some(end) = start.checked_add(len) {
		state.return_range = start..end;
		Control::Exit(Err(ExitError::Reverted))
	} else {
		Control::Exit(Err(ExitError::InvalidReturnRange))
	}
}
