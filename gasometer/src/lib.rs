#![cfg_attr(not(feature = "std"), no_std)]

mod consts;
mod costs;
mod memory;

use core::cmp::max;
use primitive_types::{H256, U256};
use evm_core::ExitError;

pub struct Gasometer<'config> {
    gas_limit: usize,
    inner: Result<Inner<'config>, ExitError>
}

impl<'config> Gasometer<'config> {
    fn inner_mut(
        &mut self
    ) -> Result<&mut Inner<'config>, ExitError> {
        self.inner.as_mut().map_err(|e| *e)
    }

    pub fn record(
        &mut self,
        cost: GasCost,
        memory: MemoryCost,
    ) -> Result<(), ExitError> {
        macro_rules! try_or_fail {
            ( $e:expr ) => (
                match $e {
                    Ok(value) => value,
                    Err(e) => {
                        self.inner = Err(e);
                        return Err(e)
                    },
                }
            )
        }

        let memory_cost = try_or_fail!(self.inner_mut()?.memory_cost(memory));
        let memory_gas = try_or_fail!(memory::memory_gas(memory_cost));
        let gas_cost = try_or_fail!(self.inner_mut()?.gas_cost(cost.clone()));
        let gas_stipend = self.inner_mut()?.gas_stipend(cost.clone());
        let gas_refund = self.inner_mut()?.gas_refund(cost.clone());
        let used_gas = self.inner_mut()?.used_gas;

        let all_gas_cost = memory_gas + used_gas + gas_cost;
        if self.gas_limit < all_gas_cost {
            self.inner = Err(ExitError::OutOfGas);
            return Err(ExitError::OutOfGas)
        }

        let after_gas = self.gas_limit - all_gas_cost;
        try_or_fail!(self.inner_mut()?.extra_check(cost, after_gas));

        self.inner_mut()?.used_gas += gas_cost - gas_stipend;
        self.inner_mut()?.memory_cost = memory_cost;
        self.inner_mut()?.refunded_gas += gas_refund;

        Ok(())
    }
}

struct Inner<'config> {
    memory_cost: usize,
    used_gas: usize,
    refunded_gas: isize,
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

    fn extra_check(
        &self,
        cost: GasCost,
        after_gas: usize,
    ) -> Result<(), ExitError> {
        match cost {
            GasCost::Call { gas, .. } => costs::call_extra_check(gas, after_gas, self.config),
            GasCost::CallCode { gas, .. } => costs::call_extra_check(gas, after_gas, self.config),
            GasCost::DelegateCall { gas, .. } => costs::call_extra_check(gas, after_gas, self.config),
            GasCost::StaticCall { gas, .. } => costs::call_extra_check(gas, after_gas, self.config),
            _ => Ok(()),
        }
    }

    fn gas_cost(
        &self,
        cost: GasCost,
    ) -> Result<usize, ExitError> {
        Ok(match cost {
            GasCost::Call { value, target_exists, .. } =>
                costs::call_cost(value, true, true, !target_exists, self.config),
            GasCost::CallCode { value, target_exists, .. } =>
                costs::call_cost(value, true, false, !target_exists, self.config),
            GasCost::DelegateCall { value, target_exists, .. } =>
                costs::call_cost(value, false, false, !target_exists, self.config),
            GasCost::StaticCall { target_exists, .. } =>
                costs::call_cost(U256::zero(), false, true, !target_exists, self.config),
            GasCost::Suicide { value, target_exists, .. } =>
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
            GasCost::Invalid => return Err(ExitError::OutOfGas),

            GasCost::ExtCodeSize => self.config.gas_extcode,
            GasCost::Balance => self.config.gas_balance,
            GasCost::BlockHash => consts::G_BLOCKHASH,
            GasCost::ExtCodeHash => consts::G_EXTCODEHASH,
        })
    }

    fn gas_stipend(
        &self,
        cost: GasCost
    ) -> usize {
        match cost {
            GasCost::Call { value, .. } => costs::call_callcode_stipend(value),
            GasCost::CallCode { value, .. } => costs::call_callcode_stipend(value),
            _ => 0,
        }
    }

    fn gas_refund(
        &self,
        cost: GasCost
    ) -> isize {
        match cost {
            GasCost::SStore { original, current, new } =>
                costs::sstore_refund(original, current, new, self.config),
            GasCost::Suicide { already_removed, .. } =>
                costs::suicide_refund(already_removed),
            _ => 0,
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
    pub has_reduced_sstore_gas_metering: bool,
    /// Whether to throw out of gas error when
    /// CALL/CALLCODE/DELEGATECALL requires more than maximum amount
    /// of gas.
    pub err_on_call_with_more_gas: bool,
    /// Whether empty account is considered exists.
    pub empty_considered_exists: bool,
}

#[derive(Clone)]
pub enum GasCost {
    Zero,
    Base,
    VeryLow,
    Low,
    Mid,
    High,
    Invalid,

    ExtCodeSize,
    Balance,
    BlockHash,
    ExtCodeHash,

    Call { value: U256, gas: U256, target_exists: bool },
    CallCode { value: U256, gas: U256, target_exists: bool },
    DelegateCall { value: U256, gas: U256, target_exists: bool },
    StaticCall { gas: U256, target_exists: bool },
    Suicide { value: U256, target_exists: bool, already_removed: bool },
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
