#[macro_use]
extern crate clap;
extern crate serde_json;
extern crate sputnikvm;

use sputnikvm::{read_hex, Gas, U256, Address};
use sputnikvm::vm::{Machine, FakeVectorMachine};
use sputnikvm::blockchain::Block;

use serde_json::{Value, Error};
use std::fs::File;
use std::path::Path;
use std::io::BufReader;

fn test_transaction(v: &Value) {
    let current_coinbase = v["env"]["currentCoinbase"].as_str().unwrap();
    let current_difficulty = v["env"]["currentDifficulty"].as_str().unwrap();
    let current_gas_limit = v["env"]["currentGasLimit"].as_str().unwrap();
    let current_number = v["env"]["currentNumber"].as_str().unwrap();
    let current_timestamp = v["env"]["currentTimestamp"].as_str().unwrap();

    let address = v["exec"]["address"].as_str().unwrap();
    let caller = v["exec"]["caller"].as_str().unwrap();
    let code = v["exec"]["code"].as_str().unwrap();
    let data = v["exec"]["data"].as_str().unwrap();
    let gas = v["exec"]["gas"].as_str().unwrap();
    let gas_price = v["exec"]["gasPrice"].as_str().unwrap();
    let origin = v["exec"]["origin"].as_str().unwrap();
    let value = v["exec"]["value"].as_str().unwrap();

    let out = v["out"].as_str().unwrap();

    let ref pre_addresses = v["pre"];
    let ref post_addresses = v["post"];

    let code = read_hex(code).unwrap();
    let data = read_hex(data).unwrap();
    let gas = Gas::from_str(gas).unwrap();

    let mut machine = FakeVectorMachine::new(code.as_ref(), data.as_ref(), gas);
    machine.fire();

    let out = read_hex(out).unwrap();
    let out_ref: &[u8] = out.as_ref();
    assert!(machine.return_values() == out_ref);

    for (address, data) in post_addresses.as_object().unwrap() {
        let address = Address::from_str(address.as_str()).unwrap();
        let storage = data["storage"].as_object().unwrap();
        for (index, value) in storage {
            let index = U256::from_str(index.as_str()).unwrap();
            let value = U256::from_str(value.as_str().unwrap()).unwrap();
            // TODO: change Address::default() to the actual address
            assert!(value == machine.block().account_storage(Address::default(), index));
        }
    }
}

fn main() {
    let app = clap_app!(jsonlighter =>
        (version: "0.1.0")
        (author: "SputnikVM Contributors")
        (@arg FILE: -f --file +takes_value +required "ethereumproject/tests JSON file to run for this test")
        (@arg TEST: -t --test +takes_value +required "test to run in the given file")
    ).get_matches();

    let path = Path::new(app.value_of("FILE").unwrap());
    let file = File::open(&path).unwrap();
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader).unwrap();
    let test = app.value_of("TEST").unwrap();

    test_transaction(&json[test]);
}
