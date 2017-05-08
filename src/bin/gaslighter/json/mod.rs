mod blockchain;

pub use self::blockchain::{JSONBlock, create_block, create_context};

use serde_json::Value;
use std::str::FromStr;
use sputnikvm::{Gas, M256, U256, Address, read_hex};
use sputnikvm::vm::{VM, SeqMachine, AccountCommitment, Context, ExecutionError, ExecutionResult, Account, HashMapStorage};

pub fn fire_with_block(machine: &mut SeqMachine, block: &JSONBlock) -> ExecutionResult<()> {
    loop {
        match machine.fire() {
            Err(ExecutionError::RequireAccount(address)) => {
                let account = block.request_account(address);
                machine.commit_account(account);
            },
            Err(ExecutionError::RequireAccountCode(address)) => {
                let account = block.request_account_code(address);
                machine.commit_account(account);
            },
            Err(ExecutionError::RequireBlockhash(number)) => {
                // The test JSON file doesn't expose any block
                // information. So those numbers are crafted by hand.
                let hash1 = M256::from_str("0xc89efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc6").unwrap();
                let hash2 = M256::from_str("0xad7c5bef027816a800da1736444fb58a807ef4c9603b7848673f7e3a68eb14a5").unwrap();
                let hash256 = M256::from_str("0x6ca54da2c4784ea43fd88b3402de07ae4bced597cbb19f323b7595857a6720ae").unwrap();

                let hash = if number == M256::from(1u64) {
                    hash1
                } else if number == M256::from(2u64) {
                    hash2
                } else if number == M256::from(256u64) {
                    hash256
                } else {
                    return Err(ExecutionError::RequireBlockhash(number));
                };

                machine.commit_blockhash(number, hash);
            },
            out => { return out; },
        }
    }
}

pub fn apply_to_block(machine: &SeqMachine, block: &mut JSONBlock) {
    for account in machine.accounts() {
        let account = (*account).clone();
        block.apply_account(account);
    }
    for log in machine.appending_logs() {
        let log = (*log).clone();
        block.apply_log(log);
    }
}

pub fn create_machine(v: &Value, block: &JSONBlock) -> SeqMachine {
    let transaction = create_context(v);

    SeqMachine::new(transaction, block.block_header())
}

pub fn test_machine(v: &Value, machine: &SeqMachine, block: &JSONBlock, debug: bool) -> bool {
    let out = v["out"].as_str();
    let gas = v["gas"].as_str();

    if out.is_some() {
        let out = read_hex(out.unwrap()).unwrap();
        let out_ref: &[u8] = out.as_ref();
        if machine.return_values() != out_ref {
            if debug {
                print!("\n");
                println!("Return value check failed. {:?} != {:?}", machine.return_values(), out_ref);
            }

            return false;
        }
    }

    if gas.is_some() {
        let gas = Gas::from_str(gas.unwrap()).unwrap();
        if machine.available_gas() != gas {
            if debug {
                print!("\n");
                println!("Gas check failed, VM returned: 0x{:x}, expected: 0x{:x}",
                         machine.available_gas(), gas);
            }

            return false;
        }
    }

    let ref post_addresses = v["post"];

    for (address, data) in post_addresses.as_object().unwrap() {
        let address = Address::from_str(address.as_str()).unwrap();
        let balance = U256::from_str(data["balance"].as_str().unwrap()).unwrap();
        let code = read_hex(data["code"].as_str().unwrap()).unwrap();
        let code_ref: &[u8] = code.as_ref();

        if code_ref != block.account_code(address) {
            if debug {
                print!("\n");
                println!("Account code check failed for address 0x{:x}.", address);
            }

            return false;
        }
        if balance != block.balance(address) {
            if debug {
                print!("\n");
                println!("Balance check failed for address 0x{:x}.", address);
            }

            return false;
        }

        let storage = data["storage"].as_object().unwrap();
        for (index, value) in storage {
            let index = M256::from_str(index.as_str()).unwrap();
            let value = M256::from_str(value.as_str().unwrap()).unwrap();
            if value != block.account_storage(address, index) {
                if debug {
                    print!("\n");
                    println!("Storage check failed for address 0x{:x} in storage index 0x{:x}",
                             address, index);
                    println!("Expected: 0x{:x}", value);
                    println!("Actual:   0x{:x}", block.account_storage(address, index));
                }
                return false;
            }
        }
    }

    let ref expect = v["expect"];

    if expect.as_object().is_some() {
        for (address, data) in expect.as_object().unwrap() {
            let address = Address::from_str(address.as_str()).unwrap();

            let storage = data["storage"].as_object().unwrap();
            for (index, value) in storage {
                let index = M256::from_str(index.as_str()).unwrap();
                let value = M256::from_str(value.as_str().unwrap()).unwrap();
                if value != block.account_storage(address, index) {
                    if debug {
                        print!("\n");
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

    let ref logs = v["logs"].as_array();

    if logs.is_some() {
        let logs = logs.unwrap();

        for log in logs {
            let log = log.as_object().unwrap();

            let address = Address::from_str(log["address"].as_str().unwrap()).unwrap();
            let data = read_hex(log["data"].as_str().unwrap()).unwrap();
            let mut topics: Vec<M256> = Vec::new();

            for topic in log["topics"].as_array().unwrap() {
                topics.push(M256::from_str(topic.as_str().unwrap()).unwrap());
            }

            if !block.find_log(address, data.as_slice(), topics.as_slice()) {
                if debug {
                    print!("\n");
                    println!("Log match failed.");
                }
                return false;
            }
        }
    }

    return true;
}
