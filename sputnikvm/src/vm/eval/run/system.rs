//! System operations instructions

use utils::address::Address;
use utils::bigint::{U256, M256};
use utils::gas::Gas;
use vm::{Memory, Log, Context, Transaction};
use super::State;

use crypto::sha3::Sha3;
use crypto::digest::Digest;
use vm::eval::utils::copy_from_memory;

pub fn suicide<M: Memory + Default>(state: &mut State<M>) {
    pop!(state, address: Address);
    let balance = state.account_state.balance(state.context.address).unwrap();
    state.account_state.increase_balance(address, balance);
    state.account_state.remove(state.context.address).unwrap();
}

pub fn log<M: Memory + Default>(state: &mut State<M>, topic_len: usize) {
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

pub fn sha3<M: Memory + Default>(state: &mut State<M>) {
    pop!(state, from, len);
    let data = copy_from_memory(&state.memory, from, len);
    let mut sha3 = Sha3::keccak256();
    sha3.input(data.as_slice());
    let mut ret = [0u8; 32];
    sha3.result(&mut ret);
    push!(state, M256::from(ret.as_ref()));
}

pub fn create<M: Memory + Default>(state: &mut State<M>, after_gas: Gas) -> Option<Context> {
    pop!(state, value: U256);
    pop!(state, init_start, init_len);
    if state.account_state.balance(state.context.address).unwrap() < value {
        push!(state, M256::zero());
        return None;
    }

    let init = copy_from_memory(&state.memory, init_start, init_len);
    let transaction = Transaction::ContractCreation {
        caller: state.context.address,
        gas_price: state.context.gas_price,
        gas_limit: after_gas,
        value: value,
        init: init,
    };
    let context = transaction.into_context(
        Gas::zero(), Some(state.context.origin), &state.account_state
    ).unwrap();

    push!(state, context.address.into());
    Some(context)
}

#[allow(unused_variables)]
pub fn call<M: Memory + Default>(state: &mut State<M>, stipend_gas: Gas, after_gas: Gas) -> Option<(Context, (M256, M256))> {
    pop!(state, gas: Gas, to: Address, value: U256);
    pop!(state, in_start, in_len, out_start, out_len);
    if state.account_state.balance(state.context.address).unwrap() < value {
        push!(state, M256::zero());
        return None;
    }

    let input = copy_from_memory(&state.memory, in_start, in_len);
    let gas_limit = gas + stipend_gas;

    let transaction = Transaction::MessageCall {
        address: to,
        caller: state.context.address,
        gas_price: state.context.gas_price,
        gas_limit: gas_limit,
        value: value,
        data: input,
    };
    let context = transaction.into_context(
        Gas::zero(), Some(state.context.origin), &state.account_state
    ).unwrap();
    push!(state, M256::zero());
    Some((context, (out_start, out_len)))
}

#[allow(unused_variables)]
pub fn callcode<M: Memory + Default>(state: &mut State<M>, stipend_gas: Gas, after_gas: Gas) -> Option<(Context, (M256, M256))> {
    pop!(state, gas: Gas, to: Address, value: U256);
    pop!(state, in_start, in_len, out_start, out_len);
    if state.account_state.balance(state.context.address).unwrap() < value {
        push!(state, M256::zero());
        return None;
    }

    let input = copy_from_memory(&state.memory, in_start, in_len);
    let gas_limit = gas + stipend_gas;
    let transaction = Transaction::MessageCall {
        address: state.context.address,
        caller: state.context.address,
        gas_price: state.context.gas_price,
        gas_limit: gas_limit,
        value: value,
        data: input,
    };
    let context = transaction.into_context(
        Gas::zero(), Some(state.context.origin), &state.account_state
    ).unwrap();
    push!(state, M256::zero());
    Some((context, (out_start, out_len)))
}
