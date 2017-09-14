//! Patch of a VM, indicating different hard-fork of the Ethereum
//! block range.

mod precompiled;

pub use self::precompiled::*;

use std::ops::Deref;
use std::str::FromStr;
use bigint::{Address, Gas};

/// Represents different block range context.
pub trait Patch {
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
    /// Precompiled contracts at given address, with required code,
    /// and its definition.
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, Box<Precompiled>)];
}

lazy_static! {
    static ref ETC_PRECOMPILEDS: [(Address, Option<&'static [u8]>, Box<Precompiled>); 4] = [
        (Address::from_str("0x0000000000000000000000000000000000000001").unwrap(),
         None,
         Box::new(ECRECPrecompiled)),
        (Address::from_str("0x0000000000000000000000000000000000000002").unwrap(),
         None,
         Box::new(SHA256Precompiled)),
        (Address::from_str("0x0000000000000000000000000000000000000003").unwrap(),
         None,
         Box::new(RIP160Precompiled)),
        (Address::from_str("0x0000000000000000000000000000000000000004").unwrap(),
         None,
         Box::new(IDPrecompiled)),
    ];
}

/// Frontier patch.
pub struct FrontierPatch;
impl Patch for FrontierPatch {
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
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, Box<Precompiled>)] {
        ETC_PRECOMPILEDS.deref() }
}

/// Homestead patch.
pub struct HomesteadPatch;
impl Patch for HomesteadPatch {
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
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, Box<Precompiled>)] {
        ETC_PRECOMPILEDS.deref() }
}

/// Patch sepcific for the `jsontests` crate.
pub struct VMTestPatch;
impl Patch for VMTestPatch {
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
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, Box<Precompiled>)] {
        ETC_PRECOMPILEDS.deref() }
}

/// EIP150 patch.
pub struct EIP150Patch;
impl Patch for EIP150Patch {
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
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, Box<Precompiled>)] {
        ETC_PRECOMPILEDS.deref() }
}

/// EIP160 patch.
pub struct EIP160Patch;
impl Patch for EIP160Patch {
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
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, Box<Precompiled>)] {
        ETC_PRECOMPILEDS.deref() }
}
