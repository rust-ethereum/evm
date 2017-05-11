use std::collections::hash_map::HashMap;
use utils::bigint::M256;

use vm::errors::{RequireError, CommitError};

#[derive(Debug, Clone)]
pub struct BlockhashState(HashMap<M256, M256>);

impl Default for BlockhashState {
    fn default() -> BlockhashState {
        BlockhashState(HashMap::new())
    }
}

impl BlockhashState {
    pub fn require(&self, number: M256) -> Result<(), RequireError> {
        match self.0.get(&number) {
            Some(_) => Ok(()),
            None => Err(RequireError::Blockhash(number)),
        }
    }

    pub fn commit(&mut self, number: M256, hash: M256) -> Result<(), CommitError> {
        if self.0.contains_key(&number) {
            return Err(CommitError::AlreadyCommitted);
        }

        self.0.insert(number, hash);
        Ok(())
    }
}
