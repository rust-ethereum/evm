//! EVM account storage

use util::bigint::{U256, M256};
use util::address::Address;
use std::collections::hash_map::HashMap;
use super::errors::{CommitError, RequireError};

/// Internal representation of an account storage. It will return a
/// `RequireError` if trying to access non-existing storage.
#[derive(Debug, Clone)]
pub struct Storage {
    partial: bool,
    address: Address,
    storage: HashMap<U256, M256>,
}

impl Into<HashMap<U256, M256>> for Storage {
    fn into(self) -> HashMap<U256, M256> {
        self.storage
    }
}

impl Storage {
    /// Create a new storage.
    pub fn new(address: Address, partial: bool) -> Self {
        Storage {
            partial: partial,
            address: address,
            storage: HashMap::new(),
        }
    }

    /// Create a full storage.
    pub fn full(address: Address) -> Self {
        Self::new(address, false)
    }

    /// Commit a value into the storage.
    pub fn commit(&mut self, index: U256, value: M256) -> Result<(), CommitError> {
        if !self.partial {
            return Err(CommitError::InvalidCommitment);
        }

        if self.storage.contains_key(&index) {
            return Err(CommitError::AlreadyCommitted);
        }

        self.storage.insert(index, value);
        Ok(())
    }

    /// Read a value from the storage.
    pub fn read(&self, index: U256) -> Result<M256, RequireError> {
        match self.storage.get(&index) {
            Some(&v) => Ok(v),
            None => if self.partial {
                Err(RequireError::AccountStorage(self.address, index))
            } else {
                Ok(M256::zero())
            }
        }
    }

    /// Write a value into the storage.
    pub fn write(&mut self, index: U256, value: M256) -> Result<(), RequireError> {
        if !self.storage.contains_key(&index) && self.partial {
            return Err(RequireError::AccountStorage(self.address, index));
        }
        self.storage.insert(index, value);
        Ok(())
    }
}
