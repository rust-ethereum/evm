use bigint::{Address, Gas, U256};
use smallvec::SmallVec;

use crate::patch::{AccountPatch, Patch, Precompiled};

#[derive(Copy, Clone)]
/// AccountPatch that can be configured in client code runtime
pub struct DynamicAccountPatch {
    /// Initial nonce for accounts.
    pub initial_nonce: U256,
    /// Initial create nonce for accounts. (EIP161.a)
    pub initial_create_nonce: U256,
    /// Whether empty accounts are considered to be existing. (EIP161.b/EIP161.c/EIP161.d)
    pub empty_considered_exists: bool,
    /// Whether to allow partial change IncreaseBalance.
    pub allow_partial_change: bool,
}

impl AccountPatch for DynamicAccountPatch {
    fn initial_nonce(&self) -> U256 {
        self.initial_nonce
    }

    fn initial_create_nonce(&self) -> U256 {
        self.initial_create_nonce
    }

    fn empty_considered_exists(&self) -> bool {
        self.empty_considered_exists
    }

    /// Whether to allow partial change IncreaseBalance.
    fn allow_partial_change(&self) -> bool {
        self.allow_partial_change
    }
}

#[derive(Clone)]
/// Patch that can be configured in client code runtime
pub struct DynamicPatch {
    /// AccountPatch
    pub account_patch: DynamicAccountPatch,
    /// Maximum contract size.
    pub code_deposit_limit: Option<usize>,
    /// Limit of the call stack.
    pub callstack_limit: usize,
    /// Gas paid for extcode.
    pub gas_extcode: Gas,
    /// Gas paid for BALANCE opcode.
    pub gas_balance: Gas,
    /// Gas paid for SLOAD opcode.
    pub gas_sload: Gas,
    /// Gas paid for SUICIDE opcode.
    pub gas_suicide: Gas,
    /// Gas paid for SUICIDE opcode when it hits a new account.
    pub gas_suicide_new_account: Gas,
    /// Gas paid for CALL opcode.
    pub gas_call: Gas,
    /// Gas paid for EXP opcode for every byte.
    pub gas_expbyte: Gas,
    /// Gas paid for a contract creation transaction.
    pub gas_transaction_create: Gas,
    /// Whether to force code deposit even if it does not have enough
    /// gas.
    pub force_code_deposit: bool,
    /// Whether the EVM has DELEGATECALL opcode.
    pub has_delegate_call: bool,
    /// Whether the EVM has STATICCALL opcode.
    pub has_static_call: bool,
    /// Whether the EVM has REVERT opcode.
    pub has_revert: bool,
    /// Whether the EVM has RETURNDATASIZE and RETURNDATACOPY opcode.
    pub has_return_data: bool,
    /// Whether the EVM has SHL, SHR and SAR
    pub has_bitwise_shift: bool,
    /// Whether the EVM has EXTCODEHASH
    pub has_extcodehash: bool,
    /// Whether the EVM has CREATE2
    pub has_create2: bool,
    /// Whether EVM should implement the EIP1283 gas metering scheme for SSTORE opcode
    pub has_reduced_sstore_gas_metering: bool,
    /// Whether to throw out of gas error when
    /// CALL/CALLCODE/DELEGATECALL requires more than maximum amount
    /// of gas.
    pub err_on_call_with_more_gas: bool,
    /// If true, only consume at maximum l64(after_gas) when
    /// CALL/CALLCODE/DELEGATECALL.
    pub call_create_l64_after_gas: bool,
    /// Maximum size of the memory, in bytes.
    /// NOTE: **NOT** runtime-configurable by block number
    pub memory_limit: usize,
    /// Check if precompiled contract is enabled
    pub enabled_precompileds: SmallVec<[Address; 8]>,
    /// Precompiled contracts at given address, with required code,
    /// and its definition.
    pub precompileds: &'static [(Address, Option<&'static [u8]>, &'static dyn Precompiled)],
}

#[rustfmt::skip]
impl Patch for DynamicPatch {
    type Account = DynamicAccountPatch;
    fn account_patch(&self) -> &Self::Account { &self.account_patch }
    fn code_deposit_limit(&self) -> Option<usize> { self.code_deposit_limit }
    fn callstack_limit(&self) -> usize { self.callstack_limit }
    fn gas_extcode(&self) -> Gas { self.gas_extcode }
    fn gas_balance(&self) -> Gas { self.gas_balance }
    fn gas_sload(&self) -> Gas { self.gas_sload }
    fn gas_suicide(&self) -> Gas { self.gas_suicide }
    fn gas_suicide_new_account(&self) -> Gas { self.gas_suicide_new_account }
    fn gas_call(&self) -> Gas { self.gas_call }
    fn gas_expbyte(&self) -> Gas { self.gas_expbyte }
    fn gas_transaction_create(&self) -> Gas { self.gas_transaction_create }
    fn force_code_deposit(&self) -> bool { self.force_code_deposit }
    fn has_delegate_call(&self) -> bool { self.has_delegate_call }
    fn has_static_call(&self) -> bool { self.has_static_call }
    fn has_revert(&self) -> bool { self.has_revert }
    fn has_return_data(&self) -> bool { self.has_return_data }
    fn has_bitwise_shift(&self) -> bool { self.has_bitwise_shift }
    fn has_create2(&self) -> bool { self.has_create2 }
    fn has_extcodehash(&self) -> bool { self.has_extcodehash }
    fn has_reduced_sstore_gas_metering(&self) -> bool { self.has_reduced_sstore_gas_metering }
    fn err_on_call_with_more_gas(&self) -> bool { self.err_on_call_with_more_gas }
    fn call_create_l64_after_gas(&self) -> bool { self.call_create_l64_after_gas }
    fn memory_limit(&self) -> usize { self.memory_limit }
    fn is_precompiled_contract_enabled(&self, address: &Address) -> bool {
        self.enabled_precompileds.iter().find(|&a| a == address).is_some()
    }
    fn precompileds(&self) -> &[(Address, Option<&'static [u8]>, &'static dyn Precompiled)] {
        &self.precompileds
    }
}
