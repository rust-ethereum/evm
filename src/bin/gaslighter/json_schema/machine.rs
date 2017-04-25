use sputnikvm::{read_hex, Gas, M256, U256, Address};
use sputnikvm::vm::{Machine, VectorMachine};
use sputnikvm::blockchain::Block;
use sputnikvm::transaction::{Transaction, VectorTransaction};

use super::{create_block, create_transaction, JSONVectorBlock};

use serde_json::{Value, Error};
use std::str::FromStr;

pub fn create_machine(v: &Value) -> VectorMachine<JSONVectorBlock, Box<JSONVectorBlock>> {
    let block = create_block(v);
    let transaction = create_transaction(v);

    let gas = Gas::from_str(v["exec"]["gas"].as_str().unwrap()).unwrap();
    let code = read_hex(v["exec"]["code"].as_str().unwrap()).unwrap();
    let data = read_hex(v["exec"]["data"].as_str().unwrap()).unwrap();

    VectorMachine::new(code.as_ref(), data.as_ref(), gas,
                       transaction, Box::new(block))
}

pub fn test_machine(v: &Value, machine: &VectorMachine<JSONVectorBlock, Box<JSONVectorBlock>>, debug: bool) -> bool {
    let out = v["out"].as_str();

    if out.is_some() {
        let out = read_hex(out.unwrap()).unwrap();
        let out_ref: &[u8] = out.as_ref();
        if machine.return_values() != out_ref {
            print!("\n");
            println!("Return value check failed.");

            return false;
        }
    }

    let ref post_addresses = v["post"];

    for (address, data) in post_addresses.as_object().unwrap() {
        let address = Address::from_str(address.as_str()).unwrap();
        let balance = U256::from_str(data["balance"].as_str().unwrap()).unwrap();
        let code = read_hex(data["code"].as_str().unwrap()).unwrap();
        let code_ref: &[u8] = code.as_ref();

        if code_ref != machine.block().account_code(address) {
            if debug {
                print!("\n");
                println!("Account code check failed for address 0x{:x}.", address);
            }

            return false;
        }
        if balance != machine.block().balance(address) {
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
            if value != machine.block().account_storage(address, index) {
                if debug {
                    print!("\n");
                    println!("Storage check failed for address 0x{:x} in storage index 0x{:x}",
                             address, index);
                    println!("Expected: 0x{:x}", value);
                    println!("Actual:   0x{:x}", machine.block().account_storage(address, index));
                }
                return false;
            }
        }
    }
    return true;
}
