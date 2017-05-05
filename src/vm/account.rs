use std::collections::hash_map;
use utils::gas::Gas;
use utils::address::Address;
use utils::bigint::{M256, U256};

pub enum Account<S> {
    Full {
        address: Address,
        balance: M256,
        storage: S,
        code: Vec<u8>,
        appending_logs: Vec<Log>,
    },
    Code {
        address: Address,
        code: Vec<u8>,
    },
    Remove(Address),
    Topup(Address, M256),
}

impl<S: Storage> Account<S> {
    pub fn address(&self) -> Address {
        match self {
            &Account::Full {
                address: address,
                balance: _,
                storage: _,
                code: _,
                appending_logs: _,
            } => address,
            &Account::Code {
                address: address,
                code: _,
            } => address,
            &Account::Remove(address) => address,
            &Account::Topup(address, _) => address,
        }
    }
}

impl<S: Storage> From<Commitment<S>> for Account<S> {
    fn from(val: Commitment<S>) -> Account<S> {
        match val {
            Commitment::Full {
                balance: balance,
                storage: storage,
                code: code,
            } => Account::Full {
                balance: balance,
                storage: storage,
                code: code,
                appending_logs: Vec::new(),
            },
            Commitment::Code {
                code: code,
            } => Account::Code {
                code: code
            },
        }
    }
}

pub enum Commitment<S> {
    Full {
        address: Address,
        balance: M256,
        storage: S,
        code: Option<Vec<u8>>,
    },
    Code {
        address: Address,
        code: Option<Vec<u8>>,
    },
}

impl<S: Storage> Commitment<S> {
    pub fn address(&self) -> Address {
        match self {
            &Commitment::Full {
                address: address,
                balance: _,
                storage: _,
                code: _,
                appending_logs: _,
            } => address,
            &Commitment::Code {
                address: address,
                code: _,
            } => address,
        }
    }
}

pub trait Storage {
    fn read(&self, index: M256) -> ExecutionResult<M256>;
    fn write(&mut self, index: M256, value: M256) -> ExecutionResult<()>;
}

pub struct HashMapStorage(hash_map::HashMap<M256, M256>);

impl Default for HashMapStorage {
    fn default() -> HashMapStorage {
        HashMapStorage(hash_map::HashMap::new())
    }
}

impl Storage for HashMapStorage {
    fn read(&self, index: M256) -> ExecutionResult<M256> {
        match self.0.get(&index) {
            Some(&v) => Ok(v),
            None => Ok(M256::zero())
        }
    }

    fn write(&mut self, index: M256, val: M256) -> ExecutionResult<()> {
        self.0.insert(index, val);
        Ok(())
    }
}

pub struct Log {
    pub data: Vec<u8>,
    pub topics: Vec<M256>,
}
