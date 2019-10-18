#![cfg_attr(not(feature = "std"), no_std)]

mod consts;
mod memory;

use primitive_types::U256;
use evm_core::{Opcode, ExternalOpcode, Stack, ExitError};

pub struct Gasometer {
    memory_cost: usize,
    used_gas: usize,
    refunded_gas: isize,
    gas_limit: usize,
}

impl Gasometer {
    pub fn record(
        &mut self,
        opcode: Result<Opcode, ExternalOpcode>,
        stack: &Stack
    ) -> Result<(), ExitError> {
        unimplemented!()
    }
}
