//! System operations instructions

#[cfg(not(feature = "std"))]
use alloc::Vec;

#[cfg(not(feature = "std"))] use alloc::rc::Rc;
#[cfg(feature = "std")] use std::rc::Rc;

use bigint::{U256, M256, H256, Address, Gas};
use ::{Memory, Log, ValidTransaction, Patch};
use eval::util::{l64, copy_from_memory};
use block_core::TransactionAction;
use super::{Control, State};

#[cfg(feature = "std")] use std::cmp::min;
#[cfg(not(feature = "std"))] use core::cmp::min;

use sha3::{Digest, Keccak256};

pub fn suicide<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, address: Address);
    let balance = state.account_state.balance(state.context.address).unwrap();
    if !state.removed.contains(&state.context.address) {
        state.removed.push(state.context.address);
    }
    state.account_state.increase_balance(address, balance);

    let balance = state.account_state.balance(state.context.address).unwrap();
    state.account_state.decrease_balance(state.context.address, balance);
}

pub fn log<M: Memory + Default, P: Patch>(state: &mut State<M, P>, topic_len: usize) {
    pop!(state, index: U256, len: U256);
    let data = copy_from_memory(&state.memory, index, len);
    let mut topics = Vec::new();
    for _ in 0..topic_len {
        topics.push(H256::from(state.stack.pop().unwrap()));
    }

    state.logs.push(Log {
        address: state.context.address,
        data: data,
        topics: topics,
    });
}

pub fn sha3<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, from: U256, len: U256);
    let data = copy_from_memory(&state.memory, from, len);
    let ret = Keccak256::digest(data.as_slice());
    push!(state, M256::from(ret.as_slice()));
}

macro_rules! try_callstack_limit {
    ( $state:expr, $patch:tt ) => {
        if $state.depth > $patch::callstack_limit() {
            push!($state, M256::zero());
            return None;
        }
    }
}

macro_rules! try_balance {
    ( $state:expr, $value:expr, $gas:expr ) => {
        if $state.account_state.balance($state.context.address).unwrap() < $value {
            push!($state, M256::zero());
            return None;
        }
    }
}

pub fn create<M: Memory + Default, P: Patch>(state: &mut State<M, P>, after_gas: Gas) -> Option<Control> {
    let l64_after_gas = if P::call_create_l64_after_gas() { l64(after_gas) } else { after_gas };

    pop!(state, value: U256);
    pop!(state, init_start: U256, init_len: U256);

    try_callstack_limit!(state, P);
    try_balance!(state, value, Gas::zero());

    let init = Rc::new(copy_from_memory(&state.memory, init_start, init_len));
    let transaction = ValidTransaction {
        caller: Some(state.context.address),
        gas_price: state.context.gas_price,
        gas_limit: l64_after_gas,
        value: value,
        input: init,
        action: TransactionAction::Create,
        nonce: state.account_state.nonce(state.context.address).unwrap(),
    };
    let context = transaction.into_context::<P>(
        Gas::zero(), Some(state.context.origin), &mut state.account_state, true,
        state.context.is_static,
    ).unwrap();

    push!(state, context.address.into());
    Some(Control::InvokeCreate(context))
}

pub fn call<M: Memory + Default, P: Patch>(state: &mut State<M, P>, stipend_gas: Gas, after_gas: Gas, as_self: bool) -> Option<Control> {
    let l64_after_gas = if P::call_create_l64_after_gas() { l64(after_gas) } else { after_gas };

    pop!(state, gas: Gas, to: Address, value: U256);
    pop!(state, in_start: U256, in_len: U256, out_start: U256, out_len: U256);
    let gas_limit = min(gas, l64_after_gas) + stipend_gas;

    try_callstack_limit!(state, P);
    try_balance!(state, value, gas_limit);

    let input = Rc::new(copy_from_memory(&state.memory, in_start, in_len));
    let transaction = ValidTransaction {
        caller: Some(state.context.address),
        gas_price: state.context.gas_price,
        gas_limit: gas_limit,
        value: value,
        input: input,
        action: TransactionAction::Call(to),
        nonce: state.account_state.nonce(state.context.address).unwrap(),
    };

    let mut context = transaction.into_context::<P>(
        Gas::zero(), Some(state.context.origin), &mut state.account_state, true,
        state.context.is_static,
    ).unwrap();
    if as_self {
        context.address = state.context.address;
    }

    push!(state, M256::from(1u64));
    Some(Control::InvokeCall(context, (out_start, out_len)))
}

pub fn static_call<M: Memory + Default, P: Patch>(state: &mut State<M, P>, stipend_gas: Gas, after_gas: Gas) -> Option<Control> {
    let l64_after_gas = if P::call_create_l64_after_gas() { l64(after_gas) } else { after_gas };

    pop!(state, gas: Gas, to: Address);
    pop!(state, in_start: U256, in_len: U256, out_start: U256, out_len: U256);
    let gas_limit = min(gas, l64_after_gas) + stipend_gas;

    try_callstack_limit!(state, P);

    let input = Rc::new(copy_from_memory(&state.memory, in_start, in_len));
    let transaction = ValidTransaction {
        caller: Some(state.context.address),
        gas_price: state.context.gas_price,
        gas_limit: gas_limit,
        value: U256::zero(),
        input: input,
        action: TransactionAction::Call(to),
        nonce: state.account_state.nonce(state.context.address).unwrap(),
    };

    let context = transaction.into_context::<P>(
        Gas::zero(), Some(state.context.origin), &mut state.account_state, true,
        true,
    ).unwrap();

    push!(state, M256::from(1u64));
    Some(Control::InvokeCall(context, (out_start, out_len)))
}

pub fn delegate_call<M: Memory + Default, P: Patch>(state: &mut State<M, P>, after_gas: Gas) -> Option<Control> {
    let l64_after_gas = if P::call_create_l64_after_gas() { l64(after_gas) } else { after_gas };

    pop!(state, gas: Gas, to: Address);
    pop!(state, in_start: U256, in_len: U256, out_start: U256, out_len: U256);
    let gas_limit = min(gas, l64_after_gas);

    try_callstack_limit!(state, P);

    let input = Rc::new(copy_from_memory(&state.memory, in_start, in_len));
    let transaction = ValidTransaction {
        caller: Some(state.context.caller),
        gas_price: state.context.gas_price,
        gas_limit: gas_limit,
        value: state.context.value,
        input: input,
        action: TransactionAction::Call(to),
        nonce: state.account_state.nonce(state.context.address).unwrap(),
    };

    let mut context = transaction.into_context::<P>(
        Gas::zero(), Some(state.context.origin), &mut state.account_state, true,
        state.context.is_static,
    ).unwrap();
    context.value = U256::zero();
    context.address = state.context.address;

    push!(state, M256::from(1u64));
    Some(Control::InvokeCall(context, (out_start, out_len)))
}
