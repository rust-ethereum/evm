use utils::bigint::M256;
use std::collections::hash_map::HashMap;
use super::errors::StorageError;

pub trait Storage {
    /// Check whether write on this index would result in an
    /// error. `write` should succeed if this function returns no
    /// error.
    fn check_write(&self, index: M256) -> Result<(), StorageError>;

    fn read(&self, index: M256) -> M256;
    fn write(&mut self, index: M256, value: M256) -> Result<(), StorageError>;
}

#[derive(Debug, Clone)]
pub struct HashMapStorage(HashMap<M256, M256>);

impl From<HashMap<M256, M256>> for HashMapStorage {
    fn from(val: HashMap<M256, M256>) -> HashMapStorage {
        HashMapStorage(val)
    }
}

impl Into<HashMap<M256, M256>> for HashMapStorage {
    fn into(self) -> HashMap<M256, M256> {
        self.0
    }
}

impl Default for HashMapStorage {
    fn default() -> HashMapStorage {
        HashMapStorage(HashMap::new())
    }
}

impl Storage for HashMapStorage {
    fn check_write(&self, _: M256) -> Result<(), StorageError> {
        Ok(())
    }

    fn read(&self, index: M256) -> M256 {
        match self.0.get(&index) {
            Some(&v) => v,
            None => M256::zero()
        }
    }

    fn write(&mut self, index: M256, val: M256) -> Result<(), StorageError> {
        self.0.insert(index, val);
        Ok(())
    }
}
