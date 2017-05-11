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

use std::collections::hash_map;
use utils::bigint::M256;
use utils::gas::Gas;
use utils::address::Address;
use self::errors::{RequireError, CommitError, MachineError};

pub type SeqVM = VM<SeqMemory, HashMapStorage>;

pub struct VM<M, S>(Vec<Machine<M, S>>);

#[derive(Debug, Clone)]
pub enum VMStatus {
    Running,
    ExitedOk,
    ExitedErr(MachineError),
}

impl<M: Memory + Default, S: Storage + Default + Clone> VM<M, S> {
    pub fn new(context: Context, block: BlockHeader, patch: Patch) -> VM<M, S> {
        let mut machines = Vec::new();
        machines.push(Machine::new(context, block, patch));
        VM(machines)
    }

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

    pub fn fire(&mut self) -> Result<(), RequireError> {
        loop {
            match self.status() {
                VMStatus::Running => self.step()?,
                VMStatus::ExitedOk | VMStatus::ExitedErr(_) => return Ok(()),
            }
        }
    }

    pub fn accounts(&self) -> hash_map::Values<Address, Account<S>> {
        self.0[0].state().account_state.accounts()
    }

    pub fn out(&self) -> &[u8] {
        self.0[0].state().out.as_slice()
    }

    pub fn available_gas(&self) -> Gas {
        self.0[0].state().available_gas()
    }
}
