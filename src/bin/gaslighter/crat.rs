use serde_json;
use sputnikvm;

use blockchain::*;

use blockchain::JSONVectorBlock;

use sputnikvm::{read_hex, Gas, M256, Address};
use sputnikvm::vm::{Machine, VectorMachine};
use sputnikvm::blockchain::Block;
use sputnikvm::transaction::{Transaction, VectorTransaction};

use serde_json::{Value, Error};
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Write, stdout};
use std::str::FromStr;

pub fn test_transaction(name: &str, v: &Value) {
    print!("Testing {} ...", name);
    stdout().flush();

    let mut block = JSONVectorBlock::new(&v["env"]);

    let current_gas_limit = Gas::from_str(v["env"]["currentGasLimit"].as_str().unwrap()).unwrap();
    let address = Address::from_str(v["exec"]["address"].as_str().unwrap()).unwrap();
    let caller = Address::from_str(v["exec"]["caller"].as_str().unwrap()).unwrap();
    let code = read_hex(v["exec"]["code"].as_str().unwrap()).unwrap();
    let data = read_hex(v["exec"]["data"].as_str().unwrap()).unwrap();
    let gas = Gas::from_str(v["exec"]["gas"].as_str().unwrap()).unwrap();
    let gas_price = Gas::from_str(v["exec"]["gasPrice"].as_str().unwrap()).unwrap();
    let origin = Address::from_str(v["exec"]["origin"].as_str().unwrap()).unwrap();
    let value = M256::from_str(v["exec"]["value"].as_str().unwrap()).unwrap();

    let transaction = VectorTransaction::message_call(
        caller, address, value, data.as_ref(), current_gas_limit
    );

    let ref pre_addresses = v["pre"];

    for (address, data) in pre_addresses.as_object().unwrap() {
        let address = Address::from_str(address.as_str()).unwrap();
        let balance = M256::from_str(data["balance"].as_str().unwrap()).unwrap();
        let code = read_hex(data["code"].as_str().unwrap()).unwrap();

        block.set_account_code(address, code.as_ref());
        block.set_balance(address, balance);

        let storage = data["storage"].as_object().unwrap();
        for (index, value) in storage {
            let index = M256::from_str(index.as_str()).unwrap();
            let value = M256::from_str(value.as_str().unwrap()).unwrap();
            block.set_account_storage(address, index, value);
        }
    }

    let mut machine: VectorMachine<JSONVectorBlock, Box<JSONVectorBlock>> =
                                   VectorMachine::new(code.as_ref(), data.as_ref(), gas,
                                                      transaction, Box::new(block));

    let out = v["out"].as_str();

    if out.is_some() {
        machine.fire();
        let out = read_hex(out.unwrap()).unwrap();
        let out_ref: &[u8] = out.as_ref();
        assert!(machine.return_values() == out_ref);
    } else {
        println!(" OK (meant to fail, not running)");
        return;
    }

    let ref post_addresses = v["post"];

    for (address, data) in post_addresses.as_object().unwrap() {
        let address = Address::from_str(address.as_str()).unwrap();
        let balance = M256::from_str(data["balance"].as_str().unwrap()).unwrap();
        let code = read_hex(data["code"].as_str().unwrap()).unwrap();

        assert!(Some(code.as_ref()) == machine.block().account_code(address));
        assert!(Some(balance.into()) == machine.block().balance(address));

        let storage = data["storage"].as_object().unwrap();
        for (index, value) in storage {
            let index = M256::from_str(index.as_str()).unwrap();
            let value = M256::from_str(value.as_str().unwrap()).unwrap();
            assert!(value == machine.block().account_storage(address, index));
        }
    }

    println!(" OK");
}

// fn main() {
//     let app = clap_app!(jsonlighter =>
//         (version: "0.1.0")
//         (author: "SputnikVM Contributors")
//         (@arg FILE: -f --file +takes_value +required "ethereumproject/tests JSON file to run for this test")
//         (@arg TEST: -t --test +takes_value "test to run in the given file")
//     ).get_matches();
//
//     let path = Path::new(app.value_of("FILE").unwrap());
//     let file = File::open(&path).unwrap();
//     let reader = BufReader::new(file);
//     let json: Value = serde_json::from_reader(reader).unwrap();
//
//     match app.value_of("TEST") {
//         Some(test) => {
//             test_transaction(test, &json[test]);
//         },
//         None => {
//             for (test, data) in json.as_object().unwrap() {
//                 test_transaction(test, &data);
//             }
//         },
//     }
// }
