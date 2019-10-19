use primitive_types::{H256, U256};
use evm_core::ExitError;
use crate::Config;
use crate::consts::*;

pub fn log_cost(n: u8, len: U256) -> Result<usize, ExitError> {
    let gas = U256::from(G_LOG)
        .checked_add(U256::from(G_LOGDATA).checked_mul(len).ok_or(ExitError::OutOfGas)?)
        .ok_or(ExitError::OutOfGas)?
        .checked_add(U256::from(G_LOGTOPIC * n as usize))
        .ok_or(ExitError::OutOfGas)?;

    if gas > U256::from(usize::max_value()) {
        return Err(ExitError::OutOfGas)
    }

    Ok(gas.as_usize())
}

pub fn sha3_cost(len: U256) -> Result<usize, ExitError> {
    let wordd = len / U256::from(32);
    let wordr = len % U256::from(32);

    let gas = U256::from(G_SHA3).checked_add(
        U256::from(G_SHA3WORD).checked_mul(
            if wordr == U256::zero() {
                wordd
            } else {
                wordd + U256::one()
            }
        ).ok_or(ExitError::OutOfGas)?
    ).ok_or(ExitError::OutOfGas)?;

    if gas > U256::from(usize::max_value()) {
        return Err(ExitError::OutOfGas)
    }

    Ok(gas.as_usize())
}

pub fn sstore_cost(original: H256, current: H256, new: H256, config: &Config) -> usize {
    if config.has_reduced_sstore_gas_metering {
        if current == H256::zero() && new != H256::zero() {
            G_SSET
        } else {
            G_SRESET
        }
    } else {
        if new == current {
            G_SNOOP
        } else {
            if original == current {
                if original == H256::zero() {
                    G_SSET
                } else {
                    G_SRESET
                }
            } else {
                G_SNOOP
            }
        }
    }
}

pub fn suicide_cost(value: U256, target_exists: bool, config: &Config) -> usize {
    let eip161 = !config.empty_considered_exists;
    let should_charge_topup = if eip161 {
        value != U256::zero() && !target_exists
    } else {
        !target_exists
    };

    let suicide_gas_topup = if should_charge_topup {
        config.gas_suicide_new_account
    } else {
        0
    };

    config.gas_suicide + suicide_gas_topup
}

pub fn call_cost(
    value: U256,
    is_call_or_callcode: bool,
    is_call_or_staticcall: bool,
    new_account: bool,
    config: &Config,
) -> usize {
    let transfers_value = value != U256::default();
    config.gas_call +
        xfer_cost(is_call_or_callcode, transfers_value) +
        new_cost(is_call_or_staticcall, new_account, transfers_value, config)
}

fn xfer_cost(
    is_call_or_callcode: bool,
    transfers_value: bool
) -> usize {
    if is_call_or_callcode && transfers_value {
        G_CALLVALUE
    } else {
        0
    }
}

fn new_cost(
    is_call_or_staticcall: bool,
    new_account: bool,
    transfers_value: bool,
    config: &Config,
) -> usize {
    let eip161 = !config.empty_considered_exists;
    if is_call_or_staticcall {
        if eip161 {
            if transfers_value && new_account {
                G_NEWACCOUNT
            } else {
                0
            }
        } else if new_account {
            G_NEWACCOUNT
        } else {
            0
        }
    } else {
        0
    }
}
