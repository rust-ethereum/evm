#![cfg_attr(not(feature = "std"), no_std)]

mod consts;
mod costs;

use core::cmp::max;
use primitive_types::{H256, U256};
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
    ) -> Result<usize, ExitError> {
        Ok(match cost {
            GasCost::Call { value, target_exists } =>
                costs::call_cost(value, true, true, !target_exists, self.config),
            GasCost::CallCode { value, target_exists } =>
                costs::call_cost(value, true, false, !target_exists, self.config),
            GasCost::DelegateCall { value, target_exists } =>
                costs::call_cost(value, false, false, !target_exists, self.config),
            GasCost::StaticCall { value, target_exists } =>
                costs::call_cost(value, false, true, !target_exists, self.config),
            GasCost::Suicide { value, target_exists } =>
                costs::suicide_cost(value, target_exists, self.config),
            GasCost::SStore { original, current, new } =>
                costs::sstore_cost(original, current, new, self.config),

            GasCost::Sha3 { len } => costs::sha3_cost(len)?,
            GasCost::Log { n, len } => costs::log_cost(n, len)?,
            GasCost::ExtCodeCopy { len } => costs::extcodecopy_cost(len, self.config)?,
            GasCost::VeryLowCopy { len } => costs::verylowcopy_cost(len)?,
            GasCost::Exp { power } => costs::exp_cost(power, self.config)?,
            GasCost::Create => consts::G_CREATE,
            GasCost::Create2 { len } => costs::create2_cost(len)?,
            GasCost::JumpDest => consts::G_JUMPDEST,
            GasCost::SLoad => self.config.gas_sload,

            GasCost::Zero => consts::G_ZERO,
            GasCost::Base => consts::G_BASE,
            GasCost::VeryLow => consts::G_VERYLOW,
            GasCost::Low => consts::G_LOW,
            GasCost::Mid => consts::G_MID,
            GasCost::High => consts::G_HIGH,

            GasCost::ExtCodeSize => self.config.gas_extcode,
            GasCost::Balance => self.config.gas_balance,
            GasCost::BlockHash => consts::G_BLOCKHASH,
            GasCost::ExtCodeHash => consts::G_EXTCODEHASH,
        })
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

    Call { value: U256, target_exists: bool },
    CallCode { value: U256, target_exists: bool },
    DelegateCall { value: U256, target_exists: bool },
    StaticCall { value: U256, target_exists: bool },
    Suicide { value: U256, target_exists: bool },
    SStore { original: H256, current: H256, new: H256 },
    Sha3 { len: U256 },
    Log { n: u8, len: U256 },
    ExtCodeCopy { len: U256 },
    VeryLowCopy { len: U256 },
    Exp { power: U256 },
    Create,
    Create2 { len: U256 },
    JumpDest,
    SLoad,
}

pub struct MemoryCost {
    pub offset: U256,
    pub len: U256,
}
