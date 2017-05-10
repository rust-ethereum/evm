mod memory;
mod stack;
mod pc;
mod storage;
mod params;
mod eval;
mod commit;
pub mod errors;

pub use self::memory::{Memory, SeqMemory};
pub use self::stack::Stack;
pub use self::pc::{PC, Instruction};
pub use self::storage::{Storage, HashMapStorage};
pub use self::params::{Context, BlockHeader, Log, Patch};
pub use self::eval::{State, Machine, Status};
pub use self::commit::{AccountCommitment, Account};

use utils::bigint::M256;
use self::errors::{RequireError, CommitError};

pub struct VM<M, S>(Vec<Machine<M, S>>);

impl<M: Memory + Default, S: Storage + Default + Clone> VM<M, S> {
    pub fn commit_account(&mut self, commitment: AccountCommitment<S>) -> Result<(), CommitError> {
        for machine in &mut self.0 {
            machine.commit_account(commitment.clone())?;
        }
        Ok(())
    }

    pub fn commit_blockhash(&mut self, number: M256, hash: M256) -> Result<(), CommitError> {
        for machine in &mut self.0 {
            machine.commit_blockhash(number, hash)?;
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), RequireError> {
        unimplemented!()
    }
}
