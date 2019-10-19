use primitive_types::U256;
use crate::Config;
use crate::consts::*;

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
    let eip161 = config.empty_considered_exists;
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
