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
pub use self::eval::{State, Machine, MachineStatus};
pub use self::commit::{AccountCommitment, Account};

use utils::bigint::M256;
use self::errors::{RequireError, CommitError, MachineError};

pub struct VM<M, S>(Vec<Machine<M, S>>);

#[derive(Debug, Clone)]
pub enum VMStatus {
    Running,
    ExitedOk,
    ExitedErr(MachineError),
}

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

    pub fn status(&self) -> VMStatus {
        match self.0[0].status() {
            MachineStatus::Running | MachineStatus::InvokeCall(_, _) => VMStatus::Running,
            MachineStatus::ExitedOk => VMStatus::ExitedOk,
            MachineStatus::ExitedErr(err) => VMStatus::ExitedErr(err),
        }
    }

    pub fn step(&mut self) -> Result<(), RequireError> {
        match self.0.last().unwrap().status().clone() {
            MachineStatus::Running => {
                self.0.last_mut().unwrap().step()
            },
            MachineStatus::ExitedOk | MachineStatus::ExitedErr(_) => {
                if self.0.len() <= 1 {
                    Ok(())
                } else {
                    let finished = self.0.pop().unwrap();
                    self.0.last_mut().unwrap().apply_sub(finished);
                    Ok(())
                }
            },
            MachineStatus::InvokeCall(context, _) => {
                let sub = self.0.last().unwrap().derive(context);
                self.0.push(sub);
                Ok(())
            },
        }
    }
}
