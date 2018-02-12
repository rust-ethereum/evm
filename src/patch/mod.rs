//! Patch of a VM, indicating different hard-fork of the Ethereum
//! block range.

mod precompiled;

pub use self::precompiled::*;

use bigint::{Address, Gas, U256, H160};

/// Account patch for account related variables.
pub trait AccountPatch {
    /// Initial nonce for accounts.
    fn initial_nonce() -> U256;
    /// Initial create nonce for accounts. (EIP161.a)
    fn initial_create_nonce() -> U256;
    /// Whether empty accounts are considered to be existing. (EIP161.b/EIP161.c/EIP161.d)
    fn empty_considered_exists() -> bool;
    /// Whether to allow partial change IncreaseBalance.
    fn allow_partial_change() -> bool {
        Self::empty_considered_exists()
    }
}

/// Mainnet account patch
pub struct EmbeddedAccountPatch;
impl AccountPatch for EmbeddedAccountPatch {
    fn initial_nonce() -> U256 { U256::zero() }
    fn initial_create_nonce() -> U256 { Self::initial_nonce() }
    fn empty_considered_exists() -> bool { true }
}

/// Mainnet account patch
pub struct EmbeddedByzantiumAccountPatch;
impl AccountPatch for EmbeddedByzantiumAccountPatch {
    fn initial_nonce() -> U256 { U256::zero() }
    fn initial_create_nonce() -> U256 { Self::initial_nonce() + U256::one() }
    fn empty_considered_exists() -> bool { false }
}

/// Represents different block range context.
pub trait Patch {
    /// Account patch
    type Account: AccountPatch;

    /// Maximum contract size.
    fn code_deposit_limit() -> Option<usize>;
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
    /// Whether the EVM has STATICCALL opcode.
    fn has_static_call() -> bool;
    /// Whether the EVM has REVERT opcode.
    fn has_revert() -> bool;
    /// Whether the EVM has RETURNDATASIZE and RETURNDATACOPY opcode.
    fn has_return_data() -> bool;
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

/// Default precompiled collections.
pub static EMBEDDED_PRECOMPILEDS: [(Address, Option<&'static [u8]>, &'static Precompiled); 4] = [
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

/// Patch sepcific for the `jsontests` crate.
pub struct VMTestPatch;
impl Patch for VMTestPatch {
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
    fn has_delegate_call() -> bool { false }
    fn has_static_call() -> bool { false }
    fn has_revert() -> bool { false }
    fn has_return_data() -> bool { false }
    fn err_on_call_with_more_gas() -> bool { true }
    fn call_create_l64_after_gas() -> bool { false }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        &EMBEDDED_PRECOMPILEDS }
}

/// Embedded patch.
pub struct EmbeddedPatch;
impl Patch for EmbeddedPatch {
    type Account = EmbeddedAccountPatch;

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
    fn err_on_call_with_more_gas() -> bool { false }
    fn call_create_l64_after_gas() -> bool { true }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        &EMBEDDED_PRECOMPILEDS }
}

/// Embedded patch.
pub struct EmbeddedByzantiumPatch;
impl Patch for EmbeddedByzantiumPatch {
    type Account = EmbeddedAccountPatch;

    fn code_deposit_limit() -> Option<usize> { Some(0x6000) }
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
    fn err_on_call_with_more_gas() -> bool { false }
    fn call_create_l64_after_gas() -> bool { true }
    fn memory_limit() -> usize { usize::max_value() }
    fn precompileds() -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        &EMBEDDED_PRECOMPILEDS }
}
