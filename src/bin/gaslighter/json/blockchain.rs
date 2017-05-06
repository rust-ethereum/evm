use sputnikvm::{Gas, M256, U256, Address, read_hex};
use sputnikvm::vm::{Machine, Log, Transaction,
                    Account, HashMapStorage, Commitment,
                    BlockHeader};

use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

pub struct JSONBlock {
    codes: HashMap<Address, Vec<u8>>,
    balances: HashMap<Address, U256>,
    storages: HashMap<Address, HashMap<M256, M256>>,

    coinbase: Address,
    timestamp: M256,
    number: M256,
    difficulty: M256,
    gas_limit: Gas,

    logs: Vec<(Address, Log)>,
}

static EMPTY: [u8; 0] = [];

impl JSONBlock {
    pub fn block_header(&self) -> BlockHeader {
        BlockHeader {
            coinbase: self.coinbase,
            timestamp: self.timestamp,
            number: self.number,
            difficulty: self.difficulty,
            gas_limit: self.gas_limit,
        }
    }

    pub fn request_account(&self, address: Address) -> Commitment<HashMapStorage> {
        let balance = self.balances.get(&address).and_then(|i| Some(i.clone())).unwrap_or(U256::zero());
        let vec_default = Vec::new();
        let code = self.codes.get(&address).unwrap_or(&vec_default);
        let hashmap_default = HashMap::new();
        let storage = self.storages.get(&address).unwrap_or(&hashmap_default);

        Commitment::Full {
            address: address,
            balance: balance,
            storage: storage.clone().into(),
            code: code.clone(),
        }
    }

    pub fn request_account_code(&self, address: Address) -> Commitment<HashMapStorage> {
        let default = Vec::new();
        let code = self.codes.get(&address).unwrap_or(&default);

        Commitment::Code {
            address: address,
            code: code.clone(),
        }
    }

    pub fn apply_account(&mut self, account: Account<HashMapStorage>) {
        match account {
            Account::Full {
                address: address,
                balance: balance,
                storage: storage,
                code: code,
                appending_logs: logs,
            } => {
                self.set_balance(address, balance);
                self.set_account_code(address, code.as_slice());
                self.storages.insert(address, storage.into());
                for log in logs {
                    self.logs.push((address, log));
                }
            },
            Account::Code {
                ..
            } => (),
            Account::Topup(address, topup) => {
                let balance = self.balance(address);
                self.set_balance(address, balance + topup);
            },
            Account::Remove(address) => {
                self.codes.remove(&address);
                self.storages.remove(&address);
                self.balances.remove(&address);
            },
        }
    }

    pub fn coinbase(&self) -> Address {
        self.coinbase
    }

    pub fn timestamp(&self) -> M256 {
        self.timestamp
    }

    pub fn number(&self) -> M256 {
        self.number
    }

    pub fn difficulty(&self) -> M256 {
        self.difficulty
    }

    pub fn gas_limit(&self) -> Gas {
        self.gas_limit
    }

    pub fn account_code(&self, address: Address) -> &[u8] {
        self.codes.get(&address).map_or(EMPTY.as_ref(), |s| s.as_ref())
    }

    pub fn set_account_code(&mut self, address: Address, code: &[u8]) {
        self.codes.insert(address, code.into());
    }

    pub fn balance(&self, address: Address) -> U256 {
        self.balances.get(&address).map_or(U256::zero(), |s| (*s).into())
    }

    pub fn set_balance(&mut self, address: Address, balance: U256) {
        self.balances.insert(address, balance);
    }

    pub fn account_storage(&self, address: Address, index: M256) -> M256 {
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

    pub fn set_account_storage(&mut self, address: Address, index: M256, val: M256) {
        if self.storages.get(&address).is_none() {
            self.storages.insert(address, HashMap::new());
        }

        let v = self.storages.get_mut(&address).unwrap();
        v.insert(index, val);
    }

    pub fn find_log(&self, address: Address, data: &[u8], topics: &[M256]) -> bool {
        for &(ref addr, ref log) in &self.logs {
            let addr = *addr;
            if addr == address && log.data.as_slice() == data && log.topics.as_slice() == topics {
                return true;
            }
        }
        return false;
    }
}

pub fn create_block(v: &Value) -> JSONBlock {
    let mut block = {
        let env = &v["env"];

        let current_coinbase = env["currentCoinbase"].as_str().unwrap();
        let current_difficulty = env["currentDifficulty"].as_str().unwrap();
        let current_gas_limit = env["currentGasLimit"].as_str().unwrap();
        let current_number = env["currentNumber"].as_str().unwrap();
        let current_timestamp = env["currentTimestamp"].as_str().unwrap();

        JSONBlock {
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
    };

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

pub fn create_transaction(v: &Value) -> Transaction {
    let address = Address::from_str(v["exec"]["address"].as_str().unwrap()).unwrap();
    let caller = Address::from_str(v["exec"]["caller"].as_str().unwrap()).unwrap();
    let code = read_hex(v["exec"]["code"].as_str().unwrap()).unwrap();
    let data = read_hex(v["exec"]["data"].as_str().unwrap()).unwrap();
    let gas = Gas::from_str(v["exec"]["gas"].as_str().unwrap()).unwrap();
    let gas_price = Gas::from_str(v["exec"]["gasPrice"].as_str().unwrap()).unwrap();
    let origin = Address::from_str(v["exec"]["origin"].as_str().unwrap()).unwrap();
    let value = M256::from_str(v["exec"]["value"].as_str().unwrap()).unwrap();

    Transaction::MessageCall {
        gas_price: gas_price,
        gas_limit: gas,
        to: address,
        originator: origin,
        caller: caller,
        data: data.into(),
        value: value,
    }
}
