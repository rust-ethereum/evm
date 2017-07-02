//! Flow control instructions.

use vm::Memory;
use super::State;

pub fn sload<M: Memory + Default>(state: &mut State<M>) {
    pop!(state, index);
    let value = state.account_state.storage(state.context.address).unwrap().read(index).unwrap();
    push!(state, value);
}

pub fn sstore<M: Memory + Default>(state: &mut State<M>) {
    pop!(state, index, value);
    state.account_state.storage_mut(state.context.address).unwrap().write(index, value).unwrap();
}

pub fn mload<M: Memory + Default>(state: &mut State<M>) {
    pop!(state, index);
    let value = state.memory.read(index);
    push!(state, value);
}

pub fn mstore<M: Memory + Default>(state: &mut State<M>) {
    pop!(state, index, value);
    state.memory.write(index, value).unwrap();
}

pub fn mstore8<M: Memory + Default>(state: &mut State<M>) {
    pop!(state, index, value);
    state.memory.write_raw(index, (value.0.low_u32() & 0xFF) as u8).unwrap();
}
