#![cfg_attr(feature = "bench", feature(test))]
#![allow(non_snake_case)]
#![allow(unused)]

#[macro_use]
extern crate jsontests_derive;

use bigint::{Address, Gas};
use evm::{EmbeddedAccountPatch, Patch, Precompiled, EMBEDDED_PRECOMPILEDS};

// Shifting opcodes tests
#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmEIP215"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct EIP215;

// EXTCODEHASH tests
#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmEIP1052"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct EIP1052;

// CREATE2 tests
#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmEIP1014"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct EIP1014;

// Gas metering changes tests
#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmEIP1283"]
#[test_with = "jsontests::util::run_test"]
#[patch = "crate::EIP1283Patch"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct EIP1283;

#[derive(Copy, Clone, Default)]
struct EIP1283Patch(pub EmbeddedAccountPatch);

#[rustfmt::skip]
impl Patch for EIP1283Patch {
    type Account = EmbeddedAccountPatch;

    fn account_patch(&self) -> &Self::Account { &self.0 }
    fn code_deposit_limit(&self) -> Option<usize> { None }
    fn callstack_limit(&self) -> usize { 2 }
    fn gas_extcode(&self) -> Gas { Gas::from(20usize) }
    fn gas_balance(&self) -> Gas { Gas::from(20usize) }
    fn gas_sload(&self) -> Gas { Gas::from(50usize) }
    fn gas_suicide(&self) -> Gas { Gas::from(0usize) }
    fn gas_suicide_new_account(&self) -> Gas { Gas::from(0usize) }
    fn gas_call(&self) -> Gas { Gas::from(40usize) }
    fn gas_expbyte(&self) -> Gas { Gas::from(10usize) }
    fn gas_transaction_create(&self) -> Gas { Gas::from(0usize) }
    fn force_code_deposit(&self) -> bool { true }
    fn has_delegate_call(&self) -> bool { true }
    fn has_static_call(&self) -> bool { true }
    fn has_revert(&self) -> bool { true }
    fn has_return_data(&self) -> bool { true }
    fn has_bitwise_shift(&self) -> bool { true }
    fn has_create2(&self) -> bool { true }
    fn has_extcodehash(&self) -> bool { true }
    fn has_reduced_sstore_gas_metering(&self) -> bool { true }
    fn err_on_call_with_more_gas(&self) -> bool { true }
    fn call_create_l64_after_gas(&self) -> bool { false }
    fn memory_limit(&self) -> usize { usize::max_value() }
    fn is_precompiled_contract_enabled(&self, address: &Address) -> bool {
        match address.low_u64() {
            0x1 | 0x2 | 0x3 | 0x4 => true,
            _ => false,
        }
    }
    fn precompileds(&self) -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        &EMBEDDED_PRECOMPILEDS
    }
}
