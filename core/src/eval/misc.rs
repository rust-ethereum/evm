use primitive_types::{H256, U256};
use super::Control;
use crate::{VM, ExitReason, ExitError, ExitSucceed};

pub fn codesize(state: &mut VM) -> Control {
    let size = U256::from(state.code.len());
    push_u256!(state, size);
    Control::Continue(1)
}

pub fn codecopy(state: &mut VM) -> Control {
    pop_u256!(state, memory_offset, code_offset, len);

    let memory_offset = as_usize_or_fail!(memory_offset);
    let ulen = as_usize_or_fail!(len);

    let code = if let Some(end) = code_offset.checked_add(len) {
        if end > U256::from(usize::max_value()) {
            &[]
        } else {
            let code_offset = code_offset.as_usize();
            let end = end.as_usize();

            if end > state.code.len() {
                &[]
            } else {
                &state.code[code_offset..end]
            }
        }
    } else {
        &[]
    };

    match state.memory.set(memory_offset, code, Some(ulen)) {
        Ok(()) => Control::Continue(1),
        Err(e) => Control::Exit(e.into()),
    }
}

pub fn calldataload(state: &mut VM) -> Control {
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

pub fn calldatasize(state: &mut VM) -> Control {
    push_u256!(state, U256::from(state.data.len()));
    Control::Continue(1)
}

pub fn calldatacopy(state: &mut VM) -> Control {
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
        Err(e) => Control::Exit(e.into()),
    }
}

pub fn pop(state: &mut VM) -> Control {
    pop!(state, _any);
    Control::Continue(1)
}

pub fn mload(state: &mut VM) -> Control {
    pop_u256!(state, index);
    let index = as_usize_or_fail!(index);
    let value = H256::from_slice(&state.memory.get(index, 32)[..]);
    push!(state, value);
    Control::Continue(1)
}

pub fn mstore(state: &mut VM) -> Control {
    pop_u256!(state, index);
    pop!(state, value);
    let index = as_usize_or_fail!(index);
    match state.memory.set(index, &value[..], Some(32)) {
        Ok(()) => Control::Continue(1),
        Err(e) => Control::Exit(e.into()),
    }
}

pub fn mstore8(state: &mut VM) -> Control {
    pop_u256!(state, index, value);
    let index = as_usize_or_fail!(index);
    let value = (value.low_u32() & 0xff) as u8;
    match state.memory.set(index, &[value], Some(1)) {
        Ok(()) => Control::Continue(1),
        Err(e) => Control::Exit(e.into()),
    }
}

pub fn jump(state: &mut VM) -> Control {
    pop_u256!(state, dest);
    let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);
    Control::Jump(dest)
}

pub fn jumpi(state: &mut VM) -> Control {
    pop_u256!(state, dest, value);
    let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);
    if value != U256::zero() {
        Control::Jump(dest)
    } else {
        Control::Continue(1)
    }
}

pub fn pc(state: &mut VM, position: usize) -> Control {
    push_u256!(state, U256::from(position));
    Control::Continue(1)
}

pub fn msize(state: &mut VM) -> Control {
    push_u256!(state, U256::from(state.memory.len()));
    Control::Continue(1)
}

pub fn push(state: &mut VM, n: usize, position: usize) -> Control {
    let end = position + 1 + n;
    if end > state.code.len() {
        return Control::Exit(ExitError::PCUnderflow.into())
    }

    push_u256!(state, U256::from(&state.code[(position + 1)..(position + 1 + n)]));
    Control::Continue(1 + n)
}

pub fn dup(state: &mut VM, n: usize) -> Control {
    let value = match state.stack.peek(n - 1) {
        Ok(value) => value,
        Err(e) => return Control::Exit(e.into()),
    };
    push!(state, value);
    Control::Continue(1)
}

pub fn swap(state: &mut VM, n: usize) -> Control {
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

pub fn ret(state: &mut VM) -> Control {
    pop_u256!(state, start, len);
    if let Some(end) = start.checked_add(len) {
        state.return_range = start..end;
        Control::Exit(ExitSucceed::Returned.into())
    } else {
        Control::Exit(ExitError::InvalidReturnRange.into())
    }
}

pub fn revert(state: &mut VM) -> Control {
    pop_u256!(state, start, len);
    if let Some(end) = start.checked_add(len) {
        state.return_range = start..end;
        Control::Exit(ExitError::Reverted.into())
    } else {
        Control::Exit(ExitError::InvalidReturnRange.into())
    }
}
