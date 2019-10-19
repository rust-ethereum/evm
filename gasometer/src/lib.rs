#![cfg_attr(not(feature = "std"), no_std)]

mod consts;

use core::cmp::max;
use primitive_types::U256;
use evm_core::{Opcode, ExternalOpcode, Stack, ExitError};

pub struct Gasometer<'config>(Result<GasometerInner<'config>, ExitError>);

struct GasometerInner<'config> {
    memory_cost: usize,
    used_gas: usize,
    refunded_gas: usize,
    gas_limit: usize,
    config: &'config Config,
}

impl<'config> GasometerInner<'config> {
    fn record_memory(
        &mut self,
        memory: MemoryCost,
    ) -> Result<(), ExitError> {
        let from = memory.offset;
        let len = memory.len;

        if len == U256::zero() {
            return Ok(())
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

        self.memory_cost = max(self.memory_cost, new);
        Ok(())
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

    Call,
    CallCode,
    DelegateCall,
    StaticCall,
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

impl<'config> Gasometer<'config> {
    pub fn record_memory(
        &mut self,
        memory: MemoryCost,
    ) -> Result<(), ExitError> {
        match self.0.as_mut().map_err(|e| *e)?.record_memory(memory) {
            Ok(()) => Ok(()),
            Err(e) => {
                self.0 = Err(e);
                Err(e)
            },
        }
    }
}
