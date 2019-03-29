//! Patch of a VM, indicating different hard-fork of the Ethereum
//! block range.

mod dynamic;
mod precompiled;

pub use self::dynamic::*;
pub use self::precompiled::*;

use bigint::{Address, Gas, H160, U256};

/// Account patch for account related variables.
/// Account patch is always static, as it's usually stays constant for any given network.
pub trait AccountPatch {
    /// Initial nonce for accounts.
    fn initial_nonce(&self) -> U256;
    /// Initial create nonce for accounts. (EIP161.a)
    fn initial_create_nonce(&self) -> U256;
    /// Whether empty accounts are considered to be existing. (EIP161.b/EIP161.c/EIP161.d)
    fn empty_considered_exists(&self) -> bool;
    /// Whether to allow partial change IncreaseBalance.
    fn allow_partial_change(&self) -> bool {
        self.empty_considered_exists()
    }
}

/// Mainnet account patch
#[derive(Default, Copy, Clone)]
pub struct EmbeddedAccountPatch;

#[rustfmt::skip]
impl AccountPatch for EmbeddedAccountPatch {
    fn initial_nonce(&self) -> U256 { U256::zero() }
    fn initial_create_nonce(&self) -> U256 { self.initial_nonce() }
    fn empty_considered_exists(&self) -> bool { true }
}

/// Represents different block range context.
pub trait Patch {
    /// Account patch
    type Account: AccountPatch;

    /// Get account patch
    fn account_patch(&self) -> &Self::Account;
    /// Maximum contract size.
    fn code_deposit_limit(&self) -> Option<usize>;
    /// Limit of the call stack.
    fn callstack_limit(&self) -> usize;
    /// Gas paid for extcode.
    fn gas_extcode(&self) -> Gas;
    /// Gas paid for BALANCE opcode.
    fn gas_balance(&self) -> Gas;
    /// Gas paid for SLOAD opcode.
    fn gas_sload(&self) -> Gas;
    /// Gas paid for SUICIDE opcode.
    fn gas_suicide(&self) -> Gas;
    /// Gas paid for SUICIDE opcode when it hits a new account.
    fn gas_suicide_new_account(&self) -> Gas;
    /// Gas paid for CALL opcode.
    fn gas_call(&self) -> Gas;
    /// Gas paid for EXP opcode for every byte.
    fn gas_expbyte(&self) -> Gas;
    /// Gas paid for a contract creation transaction.
    fn gas_transaction_create(&self) -> Gas;
    /// Whether to force code deposit even if it does not have enough
    /// gas.
    fn force_code_deposit(&self) -> bool;
    /// Whether the EVM has DELEGATECALL opcode.
    fn has_delegate_call(&self) -> bool;
    /// Whether the EVM has STATICCALL opcode.
    fn has_static_call(&self) -> bool;
    /// Whether the EVM has REVERT opcode.
    fn has_revert(&self) -> bool;
    /// Whether the EVM has RETURNDATASIZE and RETURNDATACOPY opcode.
    fn has_return_data(&self) -> bool;
    /// Whether the EVM has SHL, SHR and SAR
    fn has_bitwise_shift(&self) -> bool;
    /// Whether the EVM has CREATE2
    fn has_create2(&self) -> bool;
    /// Whether the EVM has EXTCODEHASH
    fn has_extcodehash(&self) -> bool;
    /// Whether EVM should implement the EIP1283 gas metering scheme for SSTORE opcode
    fn has_reduced_sstore_gas_metering(&self) -> bool;
    /// Whether to throw out of gas error when
    /// CALL/CALLCODE/DELEGATECALL requires more than maximum amount
    /// of gas.
    fn err_on_call_with_more_gas(&self) -> bool;
    /// If true, only consume at maximum l64(after_gas) when
    /// CALL/CALLCODE/DELEGATECALL.
    fn call_create_l64_after_gas(&self) -> bool;
    /// Maximum size of the memory, in bytes.
    /// NOTE: **NOT** runtime-configurable by block number
    fn memory_limit(&self) -> usize;
    /// Check if the precompiled contract enabled
    fn is_precompiled_contract_enabled(&self, address: &Address) -> bool;
    /// Precompiled contracts at given address, with required code,
    /// and its definition.
    fn precompileds(&self) -> &[(Address, Option<&[u8]>, &dyn Precompiled)];
}

/// Default precompiled collections.
#[rustfmt::skip]
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
#[derive(Default, Copy, Clone)]
pub struct VMTestPatch(pub EmbeddedAccountPatch);

#[rustfmt::skip]
impl Patch for VMTestPatch {
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
    fn has_reduced_sstore_gas_metering(&self) -> bool { false }
    fn err_on_call_with_more_gas(&self) -> bool { true }
    fn call_create_l64_after_gas(&self) -> bool { false }
    fn memory_limit(&self) -> usize { usize::max_value() }
    fn is_precompiled_contract_enabled(&self, address: &Address) -> bool {
        match address.low_u64() {
            0x1 | 0x2 | 0x3 | 0x4 => true,
            _ => false,
        }
    }
    fn precompileds(&self) -> &'static [(Address, Option<&'static [u8]>, &'static dyn Precompiled)] {
        &EMBEDDED_PRECOMPILEDS
    }
}

/// Embedded patch.
#[derive(Default, Copy, Clone)]
pub struct EmbeddedPatch(pub EmbeddedAccountPatch);

#[rustfmt::skip]
impl Patch for EmbeddedPatch {
    type Account = EmbeddedAccountPatch;

    fn account_patch(&self) -> &Self::Account { &self.0 }
    fn code_deposit_limit(&self) -> Option<usize> { None }
    fn callstack_limit(&self) -> usize { 1024 }
    fn gas_extcode(&self) -> Gas { Gas::from(700usize) }
    fn gas_balance(&self) -> Gas { Gas::from(400usize) }
    fn gas_sload(&self) -> Gas { Gas::from(200usize) }
    fn gas_suicide(&self) -> Gas { Gas::from(5000usize) }
    fn gas_suicide_new_account(&self) -> Gas { Gas::from(25000usize) }
    fn gas_call(&self) -> Gas { Gas::from(700usize) }
    fn gas_expbyte(&self) -> Gas { Gas::from(50usize) }
    fn gas_transaction_create(&self) -> Gas { Gas::from(32000usize) }
    fn force_code_deposit(&self) -> bool { false }
    fn has_delegate_call(&self) -> bool { true }
    fn has_static_call(&self) -> bool { false }
    fn has_revert(&self) -> bool { false }
    fn has_return_data(&self) -> bool { false }
    fn has_bitwise_shift(&self) -> bool { false }
    fn has_create2(&self) -> bool { false }
    fn has_extcodehash(&self) -> bool { false }
    fn has_reduced_sstore_gas_metering(&self) -> bool { false }
    fn err_on_call_with_more_gas(&self) -> bool { false }
    fn call_create_l64_after_gas(&self) -> bool { true }
    fn memory_limit(&self) -> usize { usize::max_value() }
    fn is_precompiled_contract_enabled(&self, address: &Address) -> bool {
        match address.low_u64() {
            0x1 | 0x2 | 0x3 | 0x4 => true,
            _ => false,
        }
    }
    fn precompileds(&self) -> &'static [(Address, Option<&'static [u8]>, &'static dyn Precompiled)] {
        &EMBEDDED_PRECOMPILEDS
    }
}
