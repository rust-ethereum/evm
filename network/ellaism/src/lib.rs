extern crate bigint;
extern crate evm;

use std::marker::PhantomData;
use bigint::{Gas, U256, H160, Address};
use evm::{Precompiled, AccountPatch, Patch,
          ID_PRECOMPILED, ECREC_PRECOMPILED, SHA256_PRECOMPILED, RIP160_PRECOMPILED};

pub static ELLA_PRECOMPILEDS: [(Address, Option<&'static [u8]>, &'static Precompiled); 4] = [
    (H160([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0x01]),
     None,
     &ECREC_PRECOMPILED),
    (H160([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0x02]),
     None,
     &SHA256_PRECOMPILED),
    (H160([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0x03]),
     None,
     &RIP160_PRECOMPILED),
    (H160([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0x04]),
     None,
     &ID_PRECOMPILED),
];

/// Mainnet account patch
pub struct MainnetAccountPatch;
impl AccountPatch for MainnetAccountPatch {
    fn initial_nonce() -> U256 { U256::zero() }
    fn initial_create_nonce() -> U256 { Self::initial_nonce() }
    fn empty_considered_exists() -> bool { true }
}

/// EIP160 patch.
pub struct EIP160Patch<A: AccountPatch>(PhantomData<A>);
pub type MainnetEIP160Patch = EIP160Patch<MainnetAccountPatch>;
impl<A: AccountPatch> Patch for EIP160Patch<A> {
    type Account = A;

    fn code_deposit_limit() -> Option<usize> { None }
    fn callstack_limit() -> usize { 1024 }
    fn gas_extcode() -> Gas { Gas::from(700usize) }
    fn gas_balance() -> Gas { Gas::from(400usize) }
    fn gas_sload() -> Gas { Gas::from(200usize) }
    fn gas_suicide() -> Gas { Gas::from(5000usize) }
    fn gas_suicide_new_account() -> Gas { Gas::from(25000usize) }
    fn gas_call() -> Gas { Gas::from(700usize) }
    fn gas_expbyte() -> Gas { Gas::from(50usize) }
    fn gas_transaction_create() -> Gas { Gas::from(32000usize) }
    fn force_code_deposit() -> bool { false }
    fn has_delegate_call() -> bool { true }
    fn has_static_call() -> bool { false }
    fn has_revert() -> bool { false }
    fn has_return_data() -> bool { false }
    fn has_bitwise_shift() -> bool { false }
    fn err_on_call_with_more_gas() -> bool { false }
    fn call_create_l64_after_gas() -> bool { true }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        &ELLA_PRECOMPILEDS }
}
