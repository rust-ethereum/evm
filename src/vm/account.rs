use std::collections::hash_map;
use utils::gas::Gas;
use utils::address::Address;
use utils::bigint::{M256, U256};

use super::{ExecutionResult, ExecutionError};

#[derive(Debug, Clone)]
pub enum Account<S> {
    Full {
        nonce: M256,
        address: Address,
        balance: U256,
        storage: S,
        code: Vec<u8>,
        appending_logs: Vec<Log>,
    },
    Code {
        address: Address,
        code: Vec<u8>,
    },
    Remove(Address),
    Topup(Address, U256),
}

impl<S: Storage> Account<S> {
    pub fn address(&self) -> Address {
        match self {
            &Account::Full {
                address: address,
                ..
            } => address,
            &Account::Code {
                address: address,
                ..
            } => address,
            &Account::Remove(address) => address,
            &Account::Topup(address, _) => address,
        }
    }
}

pub trait Storage {
    fn read(&self, index: M256) -> ExecutionResult<M256>;
    fn write(&mut self, index: M256, value: M256) -> ExecutionResult<()>;
}

#[derive(Debug, Clone)]
pub struct HashMapStorage(hash_map::HashMap<M256, M256>);

impl From<hash_map::HashMap<M256, M256>> for HashMapStorage {
    fn from(val: hash_map::HashMap<M256, M256>) -> HashMapStorage {
        HashMapStorage(val)
    }
}

impl Into<hash_map::HashMap<M256, M256>> for HashMapStorage {
    fn into(self) -> hash_map::HashMap<M256, M256> {
        self.0
    }
}

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

#[derive(Debug, Clone)]
pub struct Log {
    pub data: Vec<u8>,
    pub topics: Vec<M256>,
}
