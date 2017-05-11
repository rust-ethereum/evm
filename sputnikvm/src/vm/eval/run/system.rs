use utils::address::Address;
use vm::{Memory, Storage, Log};
use super::State;

use vm::eval::utils::copy_from_memory;

pub fn suicide<M: Memory + Default, S: Storage + Default + Clone>(state: &mut State<M, S>) {
    pop!(state, address: Address);
    let balance = state.account_state.balance(state.context.address).unwrap();
    state.account_state.increase_balance(address, balance);
    state.account_state.remove(state.context.address).unwrap();
}

pub fn log<M: Memory + Default, S: Storage + Default + Clone>(state: &mut State<M, S>, topic_len: usize) {
    pop!(state, index, len);
    let data = copy_from_memory(&state.memory, index, len);
    let mut topics = Vec::new();
    for _ in 0..topic_len {
        topics.push(state.stack.pop().unwrap());
    }

    state.logs.push(Log {
        address: state.context.address,
        data: data,
        topics: topics,
    });
}
