use sputnikvm::{Gas, H256, M256, U256, Address, read_hex};
use sputnikvm::vm::{Machine, VectorMachine};
use sputnikvm::blockchain::Block;

use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

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

pub struct JSONVectorBlock {
    codes: HashMap<Address, Vec<u8>>,
    balances: HashMap<Address, U256>,
    storages: HashMap<Address, HashMap<M256, M256>>,

    coinbase: Address,
    timestamp: M256,
    number: M256,
    difficulty: M256,
    gas_limit: Gas,

    logs: Vec<JSONVectorLog>,
}

struct JSONVectorLog {
    address: Address,
    data: Vec<u8>,
    topics: Vec<M256>,
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

            logs: Vec::new(),
        }
    }

    pub fn find_log(&self, address: Address, data: &[u8], topics: &[M256]) -> bool {
        for log in &self.logs {
            if log.address == address && log.data.as_slice() == data && log.topics.as_slice() == topics {
                return true;
            }
        }
        return false;
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
        self.logs.push(JSONVectorLog {
            address: address,
            data: data.into(),
            topics: topics.into(),
        });
    }

    fn blockhash(&self, n: M256) -> H256 {
        unimplemented!()
    }
}
