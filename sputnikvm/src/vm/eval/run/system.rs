use utils::address::Address;
use vm::{Memory, Storage};
use super::State;

pub fn suicide<M: Memory + Default, S: Storage + Default + Clone>(state: &mut State<M, S>) {
    pop!(state, address: Address);
    let balance = state.account_state.balance(state.context.address).unwrap();
    state.account_state.increase_balance(address, balance);
    state.account_state.remove(state.context.address).unwrap();
}
