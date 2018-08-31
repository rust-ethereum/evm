#![cfg_attr(feature = "bench", feature(test))]
extern crate evm;
extern crate serde_json;
extern crate hexutil;
extern crate bigint;
extern crate env_logger;
extern crate sha3;
extern crate rlp;

#[cfg(feature = "bench")]
extern crate test;

mod blockchain;
pub mod util;

pub use self::blockchain::{JSONBlock, create_block, create_context};

use serde_json::Value;
use std::str::FromStr;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use bigint::{Gas, M256, U256, H256, Address};
use hexutil::*;
use evm::errors::RequireError;
use evm::{VM, SeqContextVM, Context, VMStatus, VMTestPatch};

pub fn fire_with_block(machine: &mut SeqContextVM<VMTestPatch>, block: &JSONBlock) {
    loop {
        match machine.fire() {
            Err(RequireError::Account(address)) => {
                let account = block.request_account(address);
                machine.commit_account(account).unwrap();
            },
            Err(RequireError::AccountCode(address)) => {
                let account = block.request_account_code(address);
                machine.commit_account(account).unwrap();
            },
            Err(RequireError::AccountStorage(address, index)) => {
                let account = block.request_account_storage(address, index);
                machine.commit_account(account).unwrap();
            },
            Err(RequireError::Blockhash(number)) => {
                // The test JSON file doesn't expose any block
                // information. So those numbers are crafted by hand.
                let hash1 = H256::from_str("0xc89efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc6").unwrap();
                let hash2 = H256::from_str("0xad7c5bef027816a800da1736444fb58a807ef4c9603b7848673f7e3a68eb14a5").unwrap();
                let hash256 = H256::from_str("0x6ca54da2c4784ea43fd88b3402de07ae4bced597cbb19f323b7595857a6720ae").unwrap();

                let hash = if number == U256::from(1u64) {
                    hash1
                } else if number == U256::from(2u64) {
                    hash2
                } else if number == U256::from(256u64) {
                    hash256
                } else {
                    panic!();
                };

                machine.commit_blockhash(number, hash).unwrap();
            },
            Ok(()) => return,
        }
    }
}

pub fn apply_to_block(machine: &SeqContextVM<VMTestPatch>, block: &mut JSONBlock) {
    for account in machine.accounts() {
        let account = (*account).clone();
        block.apply_account(account);
    }
    for log in machine.logs() {
        let log = (*log).clone();
        block.apply_log(log);
    }
}

pub fn create_machine(v: &Value, block: &JSONBlock) -> SeqContextVM<VMTestPatch> {
    let transaction = create_context(v);

    SeqContextVM::<VMTestPatch>::new(transaction, block.block_header())
}

// TODO: consider refactoring
#[cfg_attr(feature = "cargo-clippy", allow(cyclomatic_complexity))]
#[cfg_attr(feature = "cargo-clippy", allow(collapsible_if))]
pub fn test_machine(v: &Value, machine: &SeqContextVM<VMTestPatch>, block: &JSONBlock, history: &[Context], debug: bool) -> bool {
    let callcreates = &v["callcreates"];

    if callcreates.as_array().is_some() {
        for (i, callcreate) in callcreates.as_array().unwrap().into_iter().enumerate() {
            let data = read_hex(callcreate["data"].as_str().unwrap()).unwrap();
            let destination = {
                let destination = callcreate["destination"].as_str().unwrap();
                if destination == "" {
                    None
                } else {
                    Some(Address::from_str(destination).unwrap())
                }
            };
            let gas_limit = Gas::from(read_u256(callcreate["gasLimit"].as_str().unwrap()));
            let value = read_u256(callcreate["value"].as_str().unwrap());

            if i >= history.len() {
                if debug {
                    println!();
                    println!("Transaction check failed, expected more than {} items.", i);
                }
                return false;
            }
            let transaction = &history[i];
            if destination.is_some() {
                if transaction.address != destination.unwrap() {
                    if debug {
                        println!();
                        println!("Transaction address mismatch. 0x{:x} != 0x{:x}.", transaction.address, destination.unwrap());
                    }
                    return false;
                }
            }
            if transaction.gas_limit != gas_limit || transaction.value != value || if destination.is_some() { transaction.data.deref() != &data } else { transaction.code.deref() != &data } {
                if debug {
                    println!();
                    println!("Transaction mismatch. gas limit 0x{:x} =?= 0x{:x}, value 0x{:x} =?= 0x{:x}, data {:?} =?= {:?}", transaction.gas_limit, gas_limit, transaction.value, value, transaction.data, data);
                }
                return false;
            }
        }
    }

    let out = v["out"].as_str();
    let gas = v["gas"].as_str();

    if let Some(out) = out {
        let out = read_hex(out).unwrap();
        let out_ref: &[u8] = out.as_ref();
        if machine.out() != out_ref {
            if debug {
                println!();
                println!("Return value check failed. {:?} != {:?}", machine.out(), out_ref);
            }

            return false;
        }
    }

    if let Some(gas) = gas {
        let gas = Gas::from(read_u256(gas));
        if machine.available_gas() != gas {
            if debug {
                println!();
                println!("Gas check failed, VM returned: 0x{:x}, expected: 0x{:x}",
                         machine.available_gas(), gas);
            }

            return false;
        }
    }

    let post_addresses = &v["post"];

    for (address, data) in post_addresses.as_object().unwrap() {
        let address = Address::from_str(address.as_str()).unwrap();
        let balance = read_u256(data["balance"].as_str().unwrap());
        let nonce = read_u256(data["nonce"].as_str().unwrap());
        let code = read_hex(data["code"].as_str().unwrap()).unwrap();
        let code_ref: &[u8] = code.as_ref();

        if code_ref != block.account_code(address) {
            if debug {
                println!();
                println!("Account code check failed for address 0x{:x}.", address);
            }

            return false;
        }

        if balance != block.balance(address) {
            if debug {
                println!();
                println!("Balance check failed for address 0x{:x}.", address);
                println!("Expected: 0x{:x}", balance);
                println!("Actual:   0x{:x}", block.balance(address));
            }

            return false;
        }

        if nonce != block.nonce(address) {
            if debug {
                println!();
                println!("Nonce check failed for address 0x{:x}.", address);
                println!("Expected: 0x{:x}", nonce);
                println!("Actual:   0x{:x}", block.nonce(address));
            }

            return false;
        }

        let storage = data["storage"].as_object().unwrap();
        for (index, value) in storage {
            let index = read_u256(index.as_str());
            let value = M256::from_str(value.as_str().unwrap()).unwrap();
            if value != block.account_storage(address, index) {
                if debug {
                    println!();
                    println!("Storage check failed for address 0x{:x} in storage index 0x{:x}",
                             address, index);
                    println!("Expected: 0x{:x}", value);
                    println!("Actual:   0x{:x}", block.account_storage(address, index));
                }
                return false;
            }
        }
    }

    let expect = &v["expect"];

    if expect.as_object().is_some() {
        for (address, data) in expect.as_object().unwrap() {
            let address = Address::from_str(address.as_str()).unwrap();

            let storage = data["storage"].as_object().unwrap();
            for (index, value) in storage {
                let index = read_u256(index.as_str());
                let value = M256::from_str(value.as_str().unwrap()).unwrap();
                if value != block.account_storage(address, index) {
                    if debug {
                        println!();
                        println!("Storage check (expect) failed for address 0x{:x} in storage index 0x{:x}",
                                 address, index);
                        println!("Expected: 0x{:x}", value);
                        println!("Actual:   0x{:x}", block.account_storage(address, index));
                    }
                    return false;
                }
            }
        }
    }


    let logs_hash = v["logs"].as_str().map(read_u256);

    if logs_hash.is_some() {
        let logs_hash = logs_hash.unwrap();
        let vm_logs_hash = block.logs_rlp_hash();
        if logs_hash != vm_logs_hash {
            if debug {
                println!();
                println!("Logs check failed (hashes mismatch)");
                println!("Expected: 0x{:x}", logs_hash);
                println!("Actual: 0x{:x}", vm_logs_hash);
            }
            return false;
        }
    }

    true
}

fn is_ok(status: &VMStatus) -> bool {
    match *status {
        VMStatus::ExitedOk => true,
        _ => false,
    }
}

pub fn test_transaction(_name: &str, v: &Value, debug: bool) -> Result<bool, VMStatus> {
    let _ = env_logger::try_init();

    let mut block = create_block(v);
    let history: Arc<Mutex<Vec<Context>>> = Arc::new(Mutex::new(Vec::new()));
    let history_closure = history.clone();
    let mut machine = create_machine(v, &block);
    machine.add_context_history_hook(move |context| {
        history_closure.lock().unwrap().push(context.clone());
    });
    fire_with_block(&mut machine, &block);
    apply_to_block(&machine, &mut block);

    if debug {
        println!("status: {:?}", machine.status());
    }
    let out = v["out"].as_str();

    if out.is_some() {
        if is_ok(&machine.status()) {
            if test_machine(v, &machine, &block, &history.lock().unwrap(), debug) {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(machine.status())
        }
    } else if !is_ok(&machine.status()) {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Read U256 number exactly the way go big.Int parses strings
/// except for base 2 and 8 which are not used in tests
pub fn read_u256(number: &str) -> U256 {
    if number.starts_with("0x") {
        U256::from_str(number).unwrap()
    } else {
        U256::from_dec_str(number).unwrap()
    }
}
