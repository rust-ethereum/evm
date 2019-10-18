use primitive_types::{H256, U256};
use super::Control;
use crate::{VM, ExitReason};

pub fn codesize(state: &mut VM) -> Control {
    let size = U256::from(state.code.len());
    push_u256!(state, size);
    Control::Continue(1)
}

pub fn codecopy(state: &mut VM) -> Control {
    pop_u256!(state, memory_offset, code_offset, len);

    let memory_offset = as_usize_or_fail!(memory_offset);
    let code_offset = as_usize_or_fail!(code_offset);
    let len = as_usize_or_fail!(len);

    let code = if let Some(end) = code_offset.checked_add(len) {
        if end > state.code.len() {
            &[]
        } else {
            &state.code[code_offset..end]
        }
    } else {
        &[]
    };

    match state.memory.set(memory_offset, code, Some(len)) {
        Ok(()) => Control::Continue(1),
        Err(e) => Control::Exit(e),
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
        Err(e) => Control::Exit(e),
    }
}

pub fn mstore8(state: &mut VM) -> Control {
    pop_u256!(state, index, value);
    let index = as_usize_or_fail!(index);
    let value = (value.low_u32() & 0xff) as u8;
    match state.memory.set(index, &[value], Some(1)) {
        Ok(()) => Control::Continue(1),
        Err(e) => Control::Exit(e),
    }
}

pub fn jump(state: &mut VM) -> Control {
    pop_u256!(state, dest);
    let dest = as_usize_or_fail!(dest, ExitReason::InvalidJump);
    Control::Jump(dest)
}

pub fn jumpi(state: &mut VM) -> Control {
    pop_u256!(state, dest, value);
    let dest = as_usize_or_fail!(dest, ExitReason::InvalidJump);
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
        return Control::Exit(ExitReason::PCUnderflow)
    }

    push_u256!(state, U256::from(&state.code[(position + 1)..(position + 1 + n)]));
    Control::Continue(1 + n)
}

pub fn dup(state: &mut VM, n: usize) -> Control {
    let value = match state.stack.peek(n - 1) {
        Ok(value) => value,
        Err(e) => return Control::Exit(e),
    };
    push!(state, value);
    Control::Continue(1)
}

pub fn swap(state: &mut VM, n: usize) -> Control {
    let val1 = match state.stack.peek(0) {
        Ok(value) => value,
        Err(e) => return Control::Exit(e),
    };
    let val2 = match state.stack.peek(n) {
        Ok(value) => value,
        Err(e) => return Control::Exit(e),
    };
    match state.stack.set(0, val2) {
        Ok(()) => (),
        Err(e) => return Control::Exit(e),
    }
    match state.stack.set(n, val1) {
        Ok(()) => (),
        Err(e) => return Control::Exit(e),
    }
    Control::Continue(1)
}

pub fn ret(state: &mut VM) -> Control {
    pop_u256!(state, start, len);
    if let Some(end) = start.checked_add(len) {
        state.return_range = start..end;
        Control::Exit(ExitReason::Returned)
    } else {
        Control::Exit(ExitReason::InvalidReturnRange)
    }
}

pub fn revert(state: &mut VM) -> Control {
    pop_u256!(state, start, len);
    if let Some(end) = start.checked_add(len) {
        state.return_range = start..end;
        Control::Exit(ExitReason::Reverted)
    } else {
        Control::Exit(ExitReason::InvalidReturnRange)
    }
}
