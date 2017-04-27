use sputnikvm::{read_hex, Gas, M256, Address};
use sputnikvm::vm::{Machine, VectorMachine};
use sputnikvm::blockchain::Block;
use sputnikvm::transaction::{Transaction, VectorTransaction};

use serde_json::{Value, Error};
use std::str::FromStr;

pub fn create_transaction(v: &Value) -> VectorTransaction {
    let current_gas_limit = Gas::from_str(v["env"]["currentGasLimit"].as_str().unwrap()).unwrap();
    let address = Address::from_str(v["exec"]["address"].as_str().unwrap()).unwrap();
    let caller = Address::from_str(v["exec"]["caller"].as_str().unwrap()).unwrap();
    let code = read_hex(v["exec"]["code"].as_str().unwrap()).unwrap();
    let data = read_hex(v["exec"]["data"].as_str().unwrap()).unwrap();
    let gas = Gas::from_str(v["exec"]["gas"].as_str().unwrap()).unwrap();
    let gas_price = Gas::from_str(v["exec"]["gasPrice"].as_str().unwrap()).unwrap();
    let origin = Address::from_str(v["exec"]["origin"].as_str().unwrap()).unwrap();
    let value = M256::from_str(v["exec"]["value"].as_str().unwrap()).unwrap();

    VectorTransaction::message_call(
        caller, address, value, data.as_ref(), gas
    )
}
