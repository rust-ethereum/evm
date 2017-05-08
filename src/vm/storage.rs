use utils::bigint::M256;

use super::{ExecutionResult, ExecutionError};
use std::collections::hash_map;

pub trait Storage {
    fn read(&self, index: M256) -> ExecutionResult<M256>;
    fn write(&mut self, index: M256, value: M256) -> ExecutionResult<()>;
    fn derive(&self) -> Self;
    fn merge(&mut self, sub: Self);
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

    fn derive(&self) -> Self {
        self.clone()
    }

    fn merge(&mut self, sub: Self) {
        for (key, val) in sub.0 {
            self.0.insert(key, val);
        }
    }
}
