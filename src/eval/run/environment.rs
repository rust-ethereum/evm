//! Environment instructions

use ::Memory;
use super::State;

pub fn calldataload<M: Memory + Default>(state: &mut State<M>) {
    pop!(state, index);
    let index: Option<usize> = if index > usize::max_value().into() {
        None
    } else {
        Some(index.as_usize())
    };
    let data = state.context.data.as_slice();
    let mut load: [u8; 32] = [0u8; 32];
    for i in 0..32 {
        if index.is_some() && index.unwrap() + i < data.len() {
            load[i] = data[index.unwrap() + i];
        }
    }
    push!(state, load.as_ref().into());
}
