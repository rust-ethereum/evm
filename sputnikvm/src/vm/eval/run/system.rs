use utils::address::Address;
use utils::bigint::M256;
use utils::gas::Gas;
use vm::{Memory, Storage, Log, Context};
use super::State;
use utils::rlp::WriteRLP;

use crypto::sha3::Sha3;
use crypto::digest::Digest;
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

pub fn sha3<M: Memory + Default, S: Storage + Default + Clone>(state: &mut State<M, S>) {
    pop!(state, from, len);
    let data = copy_from_memory(&state.memory, from, len);
    let mut sha3 = Sha3::keccak256();
    sha3.input(data.as_slice());
    let mut ret = [0u8; 32];
    sha3.result(&mut ret);
    push!(state, M256::from(ret.as_ref()));
}

pub fn create<M: Memory + Default, S: Storage + Default + Clone>(state: &mut State<M, S>, after_gas: Gas) -> Context {
    pop!(state, value, init_start, init_len);
    let init = copy_from_memory(&state.memory, init_start, init_len);
    let nonce = state.account_state.nonce(state.context.address).unwrap();
    let mut sha3 = Sha3::keccak256();
    let mut rlp: Vec<u8> = Vec::new();
    let mut address_array = [0u8; 32];
    state.context.address.write_rlp(&mut rlp);
    nonce.write_rlp(&mut rlp);
    sha3.input(rlp.as_slice());
    sha3.result(&mut address_array);
    let address = Address::from(M256::from(address_array));
    let context = Context {
        address: address,
        caller: state.context.address,
        code: init,
        data: Vec::new(),
        gas_limit: after_gas,
        gas_price: state.context.gas_price,
        origin: state.context.origin,
        value: value.into(),
    };
    push!(state, address.into());
    context
}
