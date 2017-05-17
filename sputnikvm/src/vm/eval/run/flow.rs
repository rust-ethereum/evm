//! Flow control instructions.

use vm::{Memory, Storage};
use super::State;

pub fn sload<M: Memory + Default, S: Storage + Default + Clone>(state: &mut State<M, S>) {
    pop!(state, index);
    let value = state.account_state.storage(state.context.address).unwrap().read(index);
    push!(state, value);
}

pub fn sstore<M: Memory + Default, S: Storage + Default + Clone>(state: &mut State<M, S>) {
    pop!(state, index, value);
    state.account_state.storage_mut(state.context.address).unwrap().write(index, value).unwrap();
}

pub fn mload<M: Memory + Default, S: Storage + Default + Clone>(state: &mut State<M, S>) {
    pop!(state, index);
    let value = state.memory.read(index);
    push!(state, value);
}

pub fn mstore<M: Memory + Default, S: Storage + Default + Clone>(state: &mut State<M, S>) {
    pop!(state, index, value);
    state.memory.write(index, value).unwrap();
}

pub fn mstore8<M: Memory + Default, S: Storage + Default + Clone>(state: &mut State<M, S>) {
    pop!(state, index, value);
    let value: [u8; 32] = value.into();
    state.memory.write_raw(index, value[31]).unwrap();
}
