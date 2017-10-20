//! Patch of a VM, indicating different hard-fork of the Ethereum
//! block range.

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

mod precompiled;

pub use self::precompiled::*;

#[cfg(feature = "std")] use std::ops::Deref;
#[cfg(feature = "std")] use std::str::FromStr;
#[cfg(feature = "std")] use std::marker::PhantomData;
#[cfg(not(feature = "std"))] use core::marker::PhantomData;
use bigint::{Address, Gas, U256};

/// Account patch for account related variables.
pub trait AccountPatch {
    /// Initial nonce for accounts.
    fn initial_nonce() -> U256;
}

/// Mainnet account patch
pub struct MainnetAccountPatch;
impl AccountPatch for MainnetAccountPatch {
    fn initial_nonce() -> U256 { U256::zero() }
}

/// Represents different block range context.
pub trait Patch {
    /// Account patch
    type Account: AccountPatch;

    /// Limit of the call stack.
    fn callstack_limit() -> usize;
    /// Gas paid for extcode.
    fn gas_extcode() -> Gas;
    /// Gas paid for BALANCE opcode.
    fn gas_balance() -> Gas;
    /// Gas paid for SLOAD opcode.
    fn gas_sload() -> Gas;
    /// Gas paid for SUICIDE opcode.
    fn gas_suicide() -> Gas;
    /// Gas paid for SUICIDE opcode when it hits a new account.
    fn gas_suicide_new_account() -> Gas;
    /// Gas paid for CALL opcode.
    fn gas_call() -> Gas;
    /// Gas paid for EXP opcode for every byte.
    fn gas_expbyte() -> Gas;
    /// Gas paid for a contract creation transaction.
    fn gas_transaction_create() -> Gas;
    /// Whether to force code deposit even if it does not have enough
    /// gas.
    fn force_code_deposit() -> bool;
    /// Whether the EVM has DELEGATECALL opcode.
    fn has_delegate_call() -> bool;
    /// Whether to throw out of gas error when
    /// CALL/CALLCODE/DELEGATECALL requires more than maximum amount
    /// of gas.
    fn err_on_call_with_more_gas() -> bool;
    /// If true, only consume at maximum l64(after_gas) when
    /// CALL/CALLCODE/DELEGATECALL.
    fn call_create_l64_after_gas() -> bool;
    /// Maximum size of the memory, in bytes.
    fn memory_limit() -> usize;
    /// Precompiled contracts at given address, with required code,
    /// and its definition.
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)];
}

#[cfg(feature = "std")]
lazy_static! {
    static ref ETC_PRECOMPILEDS: [(Address, Option<&'static [u8]>, &'static Precompiled); 4] = [
        (Address::from_str("0x0000000000000000000000000000000000000001").unwrap(),
         None,
         &ECREC_PRECOMPILED),
        (Address::from_str("0x0000000000000000000000000000000000000002").unwrap(),
         None,
         &SHA256_PRECOMPILED),
        (Address::from_str("0x0000000000000000000000000000000000000003").unwrap(),
         None,
         &RIP160_PRECOMPILED),
        (Address::from_str("0x0000000000000000000000000000000000000004").unwrap(),
         None,
         &ID_PRECOMPILED),
    ];
}

#[cfg(feature = "std")]
/// Frontier patch.
pub struct FrontierPatch<A: AccountPatch>(PhantomData<A>);
#[cfg(feature = "std")]
pub type MainnetFrontierPatch = FrontierPatch<MainnetAccountPatch>;
#[cfg(feature = "std")]
impl<A: AccountPatch> Patch for FrontierPatch<A> {
    type Account = A;

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
    fn err_on_call_with_more_gas() -> bool { true }
    fn call_create_l64_after_gas() -> bool { false }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        ETC_PRECOMPILEDS.deref() }
}

#[cfg(feature = "std")]
/// Homestead patch.
pub struct HomesteadPatch<A: AccountPatch>(PhantomData<A>);
#[cfg(feature = "std")]
pub type MainnetHomesteadPatch = HomesteadPatch<MainnetAccountPatch>;
#[cfg(feature = "std")]
impl<A: AccountPatch> Patch for HomesteadPatch<A> {
    type Account = A;

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
    fn err_on_call_with_more_gas() -> bool { true }
    fn call_create_l64_after_gas() -> bool { false }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        ETC_PRECOMPILEDS.deref() }
}

#[cfg(feature = "std")]
/// Patch sepcific for the `jsontests` crate.
pub struct VMTestPatch;
#[cfg(feature = "std")]
impl Patch for VMTestPatch {
    type Account = MainnetAccountPatch;

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
    fn has_delegate_call() -> bool { false }
    fn err_on_call_with_more_gas() -> bool { true }
    fn call_create_l64_after_gas() -> bool { false }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        ETC_PRECOMPILEDS.deref() }
}

#[cfg(feature = "std")]
/// EIP150 patch.
pub struct EIP150Patch<A: AccountPatch>(PhantomData<A>);
#[cfg(feature = "std")]
pub type MainnetEIP150Patch = EIP150Patch<MainnetAccountPatch>;
#[cfg(feature = "std")]
impl<A: AccountPatch> Patch for EIP150Patch<A> {
    type Account = A;

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
    fn err_on_call_with_more_gas() -> bool { false }
    fn call_create_l64_after_gas() -> bool { true }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        ETC_PRECOMPILEDS.deref() }
}

#[cfg(feature = "std")]
/// EIP160 patch.
pub struct EIP160Patch<A: AccountPatch>(PhantomData<A>);
#[cfg(feature = "std")]
pub type MainnetEIP160Patch = EIP160Patch<MainnetAccountPatch>;
#[cfg(feature = "std")]
impl<A: AccountPatch> Patch for EIP160Patch<A> {
    type Account = A;

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
    fn err_on_call_with_more_gas() -> bool { false }
    fn call_create_l64_after_gas() -> bool { true }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        ETC_PRECOMPILEDS.deref() }
}

static EMBEDDED_PRECOMPILEDS: [(Address, Option<&'static [u8]>, &'static Precompiled); 0] = [];

/// EIP160 patch.
pub struct EmbeddedPatch<A: AccountPatch>(PhantomData<A>);
pub type MainnetEmbeddedPatch = EmbeddedPatch<MainnetAccountPatch>;
impl<A: AccountPatch> Patch for EmbeddedPatch<A> {
    type Account = A;

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
    fn err_on_call_with_more_gas() -> bool { false }
    fn call_create_l64_after_gas() -> bool { true }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        &EMBEDDED_PRECOMPILEDS }
}
