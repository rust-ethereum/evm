use sputnikvm::{Gas, M256, U256, Address, read_hex};
use sputnikvm::vm::{Machine, Log, Context,
                    Account, Storage, AccountCommitment,
                    BlockHeader};

use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

pub struct JSONBlock {
    codes: HashMap<Address, Vec<u8>>,
    balances: HashMap<Address, U256>,
    storages: HashMap<Address, HashMap<M256, M256>>,
    nonces: HashMap<Address, M256>,

    coinbase: Address,
    timestamp: M256,
    number: M256,
    difficulty: M256,
    gas_limit: Gas,

    logs: Vec<Log>,
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

    pub fn request_account(&self, address: Address) -> AccountCommitment {
        let balance = self.balance(address);
        let code = self.account_code(address);
        let nonce = self.account_nonce(address);

        AccountCommitment::Full {
            address: address,
            balance: balance,
            code: code.into(),
            nonce: nonce
        }
    }

    pub fn request_account_storage(&self, address: Address, index: M256) -> AccountCommitment {
        let hashmap_default = HashMap::new();
        let storage = self.storages.get(&address).unwrap_or(&hashmap_default);
        let value = match storage.get(&index) {
            Some(val) => *val,
            None => M256::zero(),
        };

        AccountCommitment::Storage {
            address: address,
            index: index,
            value: value,
        }
    }

    pub fn request_account_code(&self, address: Address) -> AccountCommitment {
        let default = Vec::new();
        let code = self.codes.get(&address).unwrap_or(&default);

        AccountCommitment::Code {
            address: address,
            code: code.clone(),
        }
    }

    pub fn apply_account(&mut self, account: Account) {
        match account {
            Account::Full {
                address: address,
                balance: balance,
                changing_storage: changing_storage,
                code: code,
                nonce: nonce,
            } => {
                self.set_balance(address, balance);
                self.set_account_code(address, code.as_slice());
                if !self.storages.contains_key(&address) {
                    self.storages.insert(address, HashMap::new());
                }
                let changing_storage: HashMap<M256, M256> = changing_storage.into();
                for (key, value) in changing_storage {
                    self.storages.get_mut(&address).unwrap().insert(key, value);
                }
                self.set_account_nonce(address, nonce);
            },
            Account::Create {
                address: address,
                balance: balance,
                storage: storage,
                code: code,
                nonce: nonce,
            } => {
                self.set_balance(address, balance);
                self.set_account_code(address, code.as_slice());
                self.storages.insert(address, storage.into());
                self.set_account_nonce(address, nonce);
            },
            Account::IncreaseBalance(address, topup) => {
                let balance = self.balance(address);
                self.set_balance(address, balance + topup);
            },
            Account::DecreaseBalance(address, withdraw) => {
                let balance = self.balance(address);
                self.set_balance(address, balance - withdraw);
            },
        }
    }

    pub fn apply_log(&mut self, log: Log) {
        self.logs.push(log);
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

    pub fn account_nonce(&self, address: Address) -> M256 {
        self.nonces.get(&address).map_or(M256::zero(), |s| (*s).into())
    }

    pub fn set_account_nonce(&mut self, address: Address, nonce: M256) {
        self.nonces.insert(address, nonce);
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
        for log in &self.logs {
            if log.address == address && log.data.as_slice() == data && log.topics.as_slice() == topics {
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
            nonces: HashMap::new(),

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

pub fn create_context(v: &Value) -> Context {
    let address = Address::from_str(v["exec"]["address"].as_str().unwrap()).unwrap();
    let caller = Address::from_str(v["exec"]["caller"].as_str().unwrap()).unwrap();
    let code = read_hex(v["exec"]["code"].as_str().unwrap()).unwrap();
    let data = read_hex(v["exec"]["data"].as_str().unwrap()).unwrap();
    let gas = Gas::from_str(v["exec"]["gas"].as_str().unwrap()).unwrap();
    let gas_price = Gas::from_str(v["exec"]["gasPrice"].as_str().unwrap()).unwrap();
    let origin = Address::from_str(v["exec"]["origin"].as_str().unwrap()).unwrap();
    let value = U256::from_str(v["exec"]["value"].as_str().unwrap()).unwrap();

    Context {
        address: address,
        caller: caller,
        code: code,
        data: data,
        gas_limit: gas,
        gas_price: gas_price,
        origin: origin,
        value: value,
    }
}
