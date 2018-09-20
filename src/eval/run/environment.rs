//! Environment instructions

use ::Memory;
use super::State;
use patch::Patch;
use ::Address;
use bigint::{H256, M256};
use sha3::{Keccak256, Digest};


pub fn calldataload<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
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

pub fn extcodehash<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, address: Address);

    if let Some(code) = state.account_state.code_opt_nonexist(address).unwrap() {
        let hash = extcodehash_impl(&code[..]);
        push!(state, hash.into());
    } else {
        push!(state, M256::zero())
    }
}

fn extcodehash_impl(code: &[u8]) -> H256 {
    let mut hasher = Keccak256::default();
    hasher.input(code);
    let output = hasher.result();
    H256::from(&output[..])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extcodehash_empty_code() {
        let code = &[];
        let hash = extcodehash_impl(code);
        assert_eq!(hash, H256::from("0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"));
    }
}
