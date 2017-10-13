//! Flow control instructions.

use ::Memory;
use bigint::{U256, M256};
use super::State;
use patch::Patch;

pub fn sload<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, index: U256);
    let value = state.account_state.storage(state.context.address).unwrap().read(index).unwrap();
    push!(state, value);
}

pub fn sstore<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, index: U256, value: M256);
    state.account_state.storage_mut(state.context.address).unwrap().write(index, value).unwrap();
}

pub fn mload<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, index: U256);
    let value = state.memory.read(index);
    push!(state, value);
}

pub fn mstore<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, index: U256, value: M256);
    state.memory.write(index, value).unwrap();
}

pub fn mstore8<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, index: U256, value: M256);
    state.memory.write_raw(index, (value.0.low_u32() & 0xFF) as u8).unwrap();
}
