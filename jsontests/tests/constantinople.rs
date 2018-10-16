#![cfg_attr(feature = "bench", feature(test))]
#![allow(non_snake_case)]
#![allow(unused)]

#[macro_use]
extern crate jsontests_derive;
extern crate jsontests;
extern crate evm;
extern crate bigint;

#[cfg(feature = "bench")]
extern crate test;

use sputnikvm::{EmbeddedAccountPatch, Patch, EMBEDDED_PRECOMPILEDS, Precompiled};
use bigint::{Address, Gas};

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmConstantinopleTests"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct ConstantinopleTests;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmEIP1283"]
#[test_with = "jsontests::util::run_test"]
#[patch = "EIP1283Patch"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct EIP1283Tests;

struct EIP1283Patch;
impl Patch for EIP1283Patch {
    type Account = EmbeddedAccountPatch;

    fn code_deposit_limit() -> Option<usize> { None }
    fn callstack_limit() -> usize { 2 }
    fn gas_extcode() -> Gas { Gas::from(20usize) }
    fn gas_balance() -> Gas { Gas::from(20usize) }
    fn gas_sload() -> Gas { Gas::from(50usize) }
    fn gas_suicide() -> Gas { Gas::from(0usize) }
    fn gas_suicide_new_account() -> Gas { Gas::from(0usize) }
    fn gas_call() -> Gas { Gas::from(40usize) }
    fn gas_expbyte() -> Gas { Gas::from(10usize) }
    fn gas_transaction_create() -> Gas { Gas::from(0usize) }
    fn force_code_deposit() -> bool { true }
    fn has_delegate_call() -> bool { true }
    fn has_static_call() -> bool { true }
    fn has_revert() -> bool { true }
    fn has_return_data() -> bool { true }
    fn has_bitwise_shift() -> bool { true }
    fn has_extcodehash() -> bool { true }
    fn has_reduced_sstore_gas_metering() -> bool { true }
    fn err_on_call_with_more_gas() -> bool { true }
    fn call_create_l64_after_gas() -> bool { false }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        &EMBEDDED_PRECOMPILEDS }
}
