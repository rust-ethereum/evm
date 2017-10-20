//! Blockhash commitment management

#[cfg(feature = "std")] use std::collections::HashMap as Map;
#[cfg(not(feature = "std"))] use alloc::BTreeMap as Map;
use bigint::{U256, H256};

use errors::{RequireError, CommitError};

#[derive(Debug, Clone)]
/// A struct that manages the current blockhash state for one EVM.
pub struct BlockhashState(Map<U256, H256>);

impl Default for BlockhashState {
    fn default() -> BlockhashState {
        BlockhashState(Map::new())
    }
}

impl BlockhashState {
    /// Require a blockhash to be existed. If not, requires a
    /// `RequireError`.
    pub fn require(&self, number: U256) -> Result<(), RequireError> {
        match self.0.get(&number) {
            Some(_) => Ok(()),
            None => Err(RequireError::Blockhash(number)),
        }
    }

    /// Commit a new blockhash. Blockhashes are immutable so the
    /// client should be able to use this for other concurrently
    /// running EVMs.
    pub fn commit(&mut self, number: U256, hash: H256) -> Result<(), CommitError> {
        if self.0.contains_key(&number) {
            return Err(CommitError::AlreadyCommitted);
        }

        self.0.insert(number, hash);
        Ok(())
    }

    /// Get a blockhash by its number.
    pub fn get(&self, number: U256) -> Result<H256, RequireError> {
        match self.0.get(&number) {
            Some(value) => Ok(*value),
            None => Err(RequireError::Blockhash(number)),
        }
    }
}
