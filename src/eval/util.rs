//! Eval utilities

#[cfg(not(feature = "std"))]
use alloc::Vec;

use bigint::{U256, M256, Gas};
use ::Memory;
use errors::OnChainError;

#[cfg(feature = "std")] use std::cmp::min;
#[cfg(not(feature = "std"))] use core::cmp::min;

pub fn l64(gas: Gas) -> Gas {
    gas - gas / Gas::from(64u64)
}

pub fn check_range(start: U256, len: U256) -> Result<(), OnChainError> {
    if M256::from(start) + M256::from(len) < M256::from(start) {
        Err(OnChainError::InvalidRange)
    } else {
        Ok(())
    }
}

pub fn copy_from_memory<M: Memory>(memory: &M, start: U256, len: U256) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let mut i = start;
    while i < start + len {
        result.push(memory.read_raw(i));
        i = i + U256::from(1u64);
    }

    result
}

pub fn copy_into_memory<M: Memory>(memory: &mut M, values: &[u8], start: U256, value_start: U256, len: U256) {
    let value_len = U256::from(values.len());
    let mut i = start;
    let mut j = value_start;
    while i < start + len {
        if j < value_len {
            let ju: usize = j.as_usize();
            memory.write_raw(i, values[ju]).unwrap();
            j = j + U256::from(1u64);
        } else {
            memory.write_raw(i, 0u8).unwrap();
        }
        i = i + U256::from(1u64);
    }
}

pub fn copy_into_memory_apply<M: Memory>(memory: &mut M, values: &[u8], start: U256, len: U256) {
    let value_len = U256::from(values.len());
    let actual_len = min(len, value_len);
    let mut i = start;
    let mut j = 0;
    while i < start + actual_len {
        memory.write_raw(i, values[j]).unwrap();
        i = i + U256::from(1u64);
        j = j + 1;
    }
}
