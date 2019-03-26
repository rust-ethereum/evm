use bigint::{Address, Gas, H160, U256};
use evm::{
    AccountPatch, Patch, Precompiled, ECREC_PRECOMPILED, ID_PRECOMPILED, RIP160_PRECOMPILED, SHA256_PRECOMPILED,
};
use evm_precompiled_bn128::{BN128_ADD_PRECOMPILED, BN128_MUL_PRECOMPILED, BN128_PAIRING_PRECOMPILED};
use evm_precompiled_modexp::MODEXP_PRECOMPILED;
use std::marker::PhantomData;

/// Mainnet account patch
pub struct MainnetAccountPatch;

#[rustfmt::skip]
impl AccountPatch for MainnetAccountPatch {
    fn initial_nonce() -> U256 { U256::zero() }
    fn initial_create_nonce() -> U256 { Self::initial_nonce() }
    fn empty_considered_exists() -> bool { true }
}

pub struct MordenAccountPatch;

#[rustfmt::skip]
impl AccountPatch for MordenAccountPatch {
    fn initial_nonce() -> U256 { U256::from(1048576) }
    fn initial_create_nonce() -> U256 { Self::initial_nonce() }
    fn empty_considered_exists() -> bool { true }
}

#[rustfmt::skip]
pub static ETC_PRECOMPILEDS: [(Address, Option<&'static [u8]>, &'static Precompiled); 4] = [
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

#[rustfmt::skip]
pub static BYZANTIUM_PRECOMPILEDS: [(Address, Option<&'static [u8]>, &'static Precompiled); 8] = [
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
    (H160([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0x05]),
     None,
     &MODEXP_PRECOMPILED),
    (H160([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0x06]),
     None,
     &BN128_ADD_PRECOMPILED),
    (H160([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0x07]),
     None,
     &BN128_MUL_PRECOMPILED),
    (H160([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0x08]),
     None,
     &BN128_PAIRING_PRECOMPILED),
];

/// Frontier patch.
pub struct FrontierPatch<A: AccountPatch>(PhantomData<A>);
pub type MainnetFrontierPatch = FrontierPatch<MainnetAccountPatch>;
pub type MordenFrontierPatch = FrontierPatch<MordenAccountPatch>;

#[rustfmt::skip]
impl<A: AccountPatch> Patch for FrontierPatch<A> {
    type Account = A;

    fn code_deposit_limit() -> Option<usize> { None }
    fn callstack_limit() -> usize { 1024 }
    fn gas_extcode() -> Gas { Gas::from(20usize) }
    fn gas_balance() -> Gas { Gas::from(20usize) }
    fn gas_sload() -> Gas { Gas::from(50usize) }
    fn gas_suicide() -> Gas { Gas::from(0usize) }
    fn gas_suicide_new_account() -> Gas { Gas::from(0usize) }
    fn gas_call() -> Gas { Gas::from(40usize) }
    fn gas_expbyte() -> Gas { Gas::from(10usize) }
    fn gas_transaction_create() -> Gas { Gas::from(0usize) }
    fn force_code_deposit() -> bool { true }
    fn has_delegate_call() -> bool { false }
    fn has_static_call() -> bool { false }
    fn has_revert() -> bool { false }
    fn has_return_data() -> bool { false }
    fn has_bitwise_shift() -> bool { false }
    fn has_create2() -> bool { false }
    fn has_extcodehash() -> bool { false }
    fn has_reduced_sstore_gas_metering() -> bool { false }
    fn err_on_call_with_more_gas() -> bool { true }
    fn call_create_l64_after_gas() -> bool { false }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        &ETC_PRECOMPILEDS }
}

/// Homestead patch.
pub struct HomesteadPatch<A: AccountPatch>(PhantomData<A>);
pub type MainnetHomesteadPatch = HomesteadPatch<MainnetAccountPatch>;
pub type MordenHomesteadPatch = HomesteadPatch<MordenAccountPatch>;

#[rustfmt::skip]
impl<A: AccountPatch> Patch for HomesteadPatch<A> {
    type Account = A;

    fn code_deposit_limit() -> Option<usize> { None }
    fn callstack_limit() -> usize { 1024 }
    fn gas_extcode() -> Gas { Gas::from(20usize) }
    fn gas_balance() -> Gas { Gas::from(20usize) }
    fn gas_sload() -> Gas { Gas::from(50usize) }
    fn gas_suicide() -> Gas { Gas::from(0usize) }
    fn gas_suicide_new_account() -> Gas { Gas::from(0usize) }
    fn gas_call() -> Gas { Gas::from(40usize) }
    fn gas_expbyte() -> Gas { Gas::from(10usize) }
    fn gas_transaction_create() -> Gas { Gas::from(32000usize) }
    fn force_code_deposit() -> bool { false }
    fn has_delegate_call() -> bool { true }
    fn has_static_call() -> bool { false }
    fn has_revert() -> bool { false }
    fn has_return_data() -> bool { false }
    fn has_bitwise_shift() -> bool { false }
    fn has_create2() -> bool { false }
    fn has_extcodehash() -> bool { false }
    fn has_reduced_sstore_gas_metering() -> bool { false }
    fn err_on_call_with_more_gas() -> bool { true }
    fn call_create_l64_after_gas() -> bool { false }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        &ETC_PRECOMPILEDS }
}

/// EIP150 patch.
pub struct EIP150Patch<A: AccountPatch>(PhantomData<A>);
pub type MainnetEIP150Patch = EIP150Patch<MainnetAccountPatch>;
pub type MordenEIP150Patch = EIP150Patch<MordenAccountPatch>;

#[rustfmt::skip]
impl<A: AccountPatch> Patch for EIP150Patch<A> {
    type Account = A;

    fn code_deposit_limit() -> Option<usize> { None }
    fn callstack_limit() -> usize { 1024 }
    fn gas_extcode() -> Gas { Gas::from(700usize) }
    fn gas_balance() -> Gas { Gas::from(400usize) }
    fn gas_sload() -> Gas { Gas::from(200usize) }
    fn gas_suicide() -> Gas { Gas::from(5000usize) }
    fn gas_suicide_new_account() -> Gas { Gas::from(25000usize) }
    fn gas_call() -> Gas { Gas::from(700usize) }
    fn gas_expbyte() -> Gas { Gas::from(10usize) }
    fn gas_transaction_create() -> Gas { Gas::from(32000usize) }
    fn force_code_deposit() -> bool { false }
    fn has_delegate_call() -> bool { true }
    fn has_static_call() -> bool { false }
    fn has_revert() -> bool { false }
    fn has_return_data() -> bool { false }
    fn has_bitwise_shift() -> bool { false }
    fn has_create2() -> bool { false }
    fn has_extcodehash() -> bool { false }
    fn has_reduced_sstore_gas_metering() -> bool { false }
    fn err_on_call_with_more_gas() -> bool { false }
    fn call_create_l64_after_gas() -> bool { true }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        &ETC_PRECOMPILEDS }
}

/// EIP160 patch.
pub struct EIP160Patch<A: AccountPatch>(PhantomData<A>);
pub type MainnetEIP160Patch = EIP160Patch<MainnetAccountPatch>;
pub type MordenEIP160Patch = EIP160Patch<MordenAccountPatch>;

#[rustfmt::skip]
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
    fn has_create2() -> bool { false }
    fn has_extcodehash() -> bool { false }
    fn has_reduced_sstore_gas_metering() -> bool { false }
    fn err_on_call_with_more_gas() -> bool { false }
    fn call_create_l64_after_gas() -> bool { true }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        &ETC_PRECOMPILEDS }
}

/// Byzantium patch.
pub struct ByzantiumPatch<A: AccountPatch>(PhantomData<A>);
pub type MainnetByzantiumPatch = ByzantiumPatch<MainnetAccountPatch>;
pub type MordenByzantiumPatch = ByzantiumPatch<MordenAccountPatch>;

#[rustfmt::skip]
impl<A: AccountPatch> Patch for ByzantiumPatch<A> {
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
    fn has_static_call() -> bool { true }
    fn has_revert() -> bool { true }
    fn has_return_data() -> bool { true }
    fn has_bitwise_shift() -> bool { false }
    fn has_create2() -> bool { false }
    fn has_extcodehash() -> bool { false }
    fn has_reduced_sstore_gas_metering() -> bool { false }
    fn err_on_call_with_more_gas() -> bool { false }
    fn call_create_l64_after_gas() -> bool { true }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        &BYZANTIUM_PRECOMPILEDS }
}

/// Constantinople patch (includes Byzantium changes)
pub struct ConstantinoplePatch<A: AccountPatch>(PhantomData<A>);
pub type MainnetConstantinoplePatch = ConstantinoplePatch<MainnetAccountPatch>;
pub type MordenConstantinoplePatch = ConstantinoplePatch<MordenAccountPatch>;

#[rustfmt::skip]
impl<A: AccountPatch> Patch for ConstantinoplePatch<A> {
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
    fn has_static_call() -> bool { true }
    fn has_revert() -> bool { true }
    fn has_return_data() -> bool { true }
    fn has_bitwise_shift() -> bool { true }
    fn has_create2() -> bool { true }
    fn has_extcodehash() -> bool { true }
    fn has_reduced_sstore_gas_metering() -> bool { true }
    fn err_on_call_with_more_gas() -> bool { false }
    fn call_create_l64_after_gas() -> bool { true }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        &BYZANTIUM_PRECOMPILEDS }
}
