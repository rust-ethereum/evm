use ::{Gas, H256, M256, U256, Address, read_hex};
use ::blockchain::Block;
use ::transaction::{Transaction, VectorTransaction};
use ::vm::{Machine, VectorMachine};

use std::collections::HashMap;
use serde_json::{Value, Error};
use std::str::FromStr;

pub struct JSONVectorBlock {
    codes: HashMap<Address, Vec<u8>>,
    balances: HashMap<Address, U256>,
    storages: HashMap<Address, HashMap<M256, M256>>,

    coinbase: Address,
    timestamp: M256,
    number: M256,
    difficulty: M256,
    gas_limit: Gas
}

impl JSONVectorBlock {
    pub fn new(env: &Value) -> Self {
        let current_coinbase = env["currentCoinbase"].as_str().unwrap();
        let current_difficulty = env["currentDifficulty"].as_str().unwrap();
        let current_gas_limit = env["currentGasLimit"].as_str().unwrap();
        let current_number = env["currentNumber"].as_str().unwrap();
        let current_timestamp = env["currentTimestamp"].as_str().unwrap();

        JSONVectorBlock {
            balances: HashMap::new(),
            storages: HashMap::new(),
            codes: HashMap::new(),

            coinbase: Address::from_str(current_coinbase).unwrap(),
            difficulty: M256::from_str(current_difficulty).unwrap(),
            gas_limit: Gas::from_str(current_gas_limit).unwrap(),
            number: M256::from_str(current_number).unwrap(),
            timestamp: M256::from_str(current_timestamp).unwrap(),
        }
    }
}

static EMPTY: [u8; 0] = [];

impl Block for JSONVectorBlock {
    fn account_code(&self, address: Address) -> &[u8] {
        self.codes.get(&address).map_or(EMPTY.as_ref(), |s| s.as_ref())
    }

    fn set_account_code(&mut self, address: Address, code: &[u8]) {
        self.codes.insert(address, code.into());
    }

    fn set_balance(&mut self, address: Address, balance: U256) {
        self.balances.insert(address, balance);
    }

    fn create_account(&mut self, code: &[u8]) -> Option<Address> {
        unimplemented!()
    }

    fn coinbase(&self) -> Address {
        self.coinbase
    }

    fn balance(&self, address: Address) -> U256 {
        self.balances.get(&address).map_or(U256::zero(), |s| (*s).into())
    }

    fn timestamp(&self) -> M256 {
        self.timestamp
    }

    fn number(&self) -> M256 {
        self.number
    }

    fn difficulty(&self) -> M256 {
        self.difficulty
    }

    fn gas_limit(&self) -> Gas {
        self.gas_limit
    }

    fn account_storage(&self, address: Address, index: M256) -> M256 {
        match self.storages.get(&address) {
            None => M256::zero(),
            Some(ref ve) => {
                match ve.get(&index) {
                    Some(&v) => v,
                    None => M256::zero()
                }
            }
        }
    }

    fn set_account_storage(&mut self, address: Address, index: M256, val: M256) {
        if self.storages.get(&address).is_none() {
            self.storages.insert(address, HashMap::new());
        }

        let v = self.storages.get_mut(&address).unwrap();
        v.insert(index, val);
    }

    fn log(&mut self, address: Address, data: &[u8], topics: &[M256]) {
        unimplemented!()
    }

    fn blockhash(&self, n: M256) -> H256 {
        unimplemented!()
    }
}

pub fn create_block(v: &Value) -> JSONVectorBlock {
    let mut block = JSONVectorBlock::new(&v["env"]);

    let ref pre_addresses = v["pre"];

    for (address, data) in pre_addresses.as_object().unwrap() {
        let address = Address::from_str(address.as_str()).unwrap();
        let balance = U256::from_str(data["balance"].as_str().unwrap()).unwrap();
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

    block
}

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
        caller, address, value, data.as_ref(), current_gas_limit
    )
}
