#![cfg_attr(not(feature = "std"), no_std)]

mod consts;
mod costs;

use core::cmp::max;
use primitive_types::U256;
use evm_core::{Opcode, ExternalOpcode, Stack, ExitError};

pub struct Gasometer<'config> {
    gas_limit: usize,
    inner: Result<Inner<'config>, ExitError>
}

struct Inner<'config> {
    memory_cost: usize,
    used_gas: usize,
    refunded_gas: usize,
    config: &'config Config,
}

impl<'config> Inner<'config> {
    fn memory_cost(
        &self,
        memory: MemoryCost,
    ) -> Result<usize, ExitError> {
        let from = memory.offset;
        let len = memory.len;

        if len == U256::zero() {
            return Ok(self.memory_cost)
        }

        let end = from.checked_add(len).ok_or(ExitError::OutOfGas)?;

        if end > U256::from(usize::max_value()) {
            return Err(ExitError::OutOfGas)
        }
        let end = end.as_usize();

        let rem = end % 32;
        let new = if rem == 0 {
            end / 32
        } else {
            end / 32 + 1
        };

        Ok(max(self.memory_cost, new))
    }

    fn gas_cost(
        &self,
        cost: GasCost,
    ) -> usize {
        match cost {
            GasCost::Call(value, new_account) =>
                costs::call_cost(value, true, true, new_account, self.config),
            GasCost::CallCode(value, new_account) =>
                costs::call_cost(value, true, false, new_account, self.config),
            GasCost::DelegateCall(value, new_account) =>
                costs::call_cost(value, false, false, new_account, self.config),
            GasCost::StaticCall(value, new_account) =>
                costs::call_cost(value, false, true, new_account, self.config),

            _ => unimplemented!()
        }
    }
}

pub struct Config {
    /// Gas paid for extcode.
    pub gas_extcode: usize,
    /// Gas paid for BALANCE opcode.
    pub gas_balance: usize,
    /// Gas paid for SLOAD opcode.
    pub gas_sload: usize,
    /// Gas paid for SUICIDE opcode.
    pub gas_suicide: usize,
    /// Gas paid for SUICIDE opcode when it hits a new account.
    pub gas_suicide_new_account: usize,
    /// Gas paid for CALL opcode.
    pub gas_call: usize,
    /// Gas paid for EXP opcode for every byte.
    pub gas_expbyte: usize,
    /// Gas paid for a contract creation transaction.
    pub gas_transaction_create: usize,
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
    /// Whether empty account is considered exists.
    pub empty_considered_exists: bool,
}

pub enum GasCost {
    Zero,
    Base,
    VeryLow,
    Low,
    Mid,
    High,

    ExtCodeSize,
    Balance,
    BlockHash,
    ExtCodeHash,

    Call(U256, bool),
    CallCode(U256, bool),
    DelegateCall(U256, bool),
    StaticCall(U256, bool),
    Suicide,
    SStore,
    Sha3,
    Log,
    ExtCodeCopy,
    CallDataCopy,
    CodeCopy,
    ReturnDataCopy,
    Exp,
    Create,
    Create2,
    JumpDest,
    SLoad,
}

pub struct MemoryCost {
    pub offset: U256,
    pub len: U256,
}
