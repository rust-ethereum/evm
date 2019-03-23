//! Flow control instructions.

use bigint::{U256, M256};
use crate::{Memory, Patch};
use super::State;

pub fn sload<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, index: U256);
    let value = state.account_state.storage_read(state.context.address, index).unwrap();
    push!(state, value);
}

pub fn sstore<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, index: U256, value: M256);
    state.account_state.storage_write(state.context.address, index, value).unwrap();
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
