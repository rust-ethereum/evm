use sputnikvm::{Gas, H256, U256, Address};
use sputnikvm::vm::{Machine, VectorMachine};
use sputnikvm::blockchain::Block;

use serde_json::Value;
use std::collections::HashMap;

pub struct JSONVectorBlock {
    codes: HashMap<Address, Vec<u8>>,
    balances: HashMap<Address, U256>,
    storages: HashMap<Address, Vec<U256>>,

    coinbase: Address,
    timestamp: U256,
    number: U256,
    difficulty: U256,
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
            difficulty: U256::from_str(current_difficulty).unwrap(),
            gas_limit: Gas::from_str(current_gas_limit).unwrap(),
            number: U256::from_str(current_number).unwrap(),
            timestamp: U256::from_str(current_timestamp).unwrap(),
        }
    }

    pub fn set_account_code(&mut self, address: Address, code: &[u8]) {
        self.codes.insert(address, code.into());
    }

    pub fn set_balance(&mut self, address: Address, balance: U256) {
        self.balances.insert(address, balance);
    }
}

impl Block for JSONVectorBlock {
    fn account_code(&self, address: Address) -> Option<&[u8]> {
        self.codes.get(&address).map(|s| s.as_ref())
    }

    fn create_account(&mut self, code: &[u8]) -> Option<Address> {
        unimplemented!()
    }

    fn coinbase(&self) -> Address {
        self.coinbase
    }

    fn balance(&self, address: Address) -> Option<U256> {
        self.balances.get(&address).map(|s| *s)
    }

    fn timestamp(&self) -> U256 {
        self.timestamp
    }

    fn number(&self) -> U256 {
        self.number
    }

    fn difficulty(&self) -> U256 {
        self.difficulty
    }

    fn gas_limit(&self) -> Gas {
        self.gas_limit
    }

    fn account_storage(&self, address: Address, index: U256) -> U256 {
        match self.storages.get(&address) {
            None => U256::zero(),
            Some(ref ve) => {
                let index: usize = index.into();

                match ve.get(index) {
                    Some(&v) => v,
                    None => U256::zero()
                }
            }
        }
    }

    fn set_account_storage(&mut self, address: Address, index: U256, val: U256) {
        if self.storages.get(&address).is_none() {
            self.storages.insert(address, Vec::new());
        }

        let v = self.storages.get_mut(&address).unwrap();

        let index: usize = index.into();

        if v.len() <= index {
            v.resize(index + 1, 0.into());
        }

        v[index] = val;
    }

    fn log(&mut self, address: Address, data: &[u8], topics: &[U256]) {
        unimplemented!()
    }

    fn blockhash(&self, n: U256) -> H256 {
        unimplemented!()
    }
}
