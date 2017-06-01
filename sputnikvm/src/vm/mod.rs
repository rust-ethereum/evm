//! VM implementation, traits and structs
//!
//! ### Lifecycle
//!
//! A VM can be started given a `Context` and a `BlockHeader`. The
//! user can then `fire` or `step` to run it. Those functions would
//! only fail if it needs some information (accounts in the current
//! block, or block hashes of previous blocks). If this happens, one
//! can use the function `commit_account` and `commit_blockhash` to
//! commit those information to the VM, and `fire` or `step` again
//! until it succeeds. The current VM status can always be obtained
//! using the `status` function.

mod memory;
mod stack;
mod pc;
mod storage;
mod params;
mod eval;
mod commit;
mod patch;
mod transaction;
pub mod errors;

pub use self::memory::{Memory, SeqMemory};
pub use self::stack::Stack;
pub use self::pc::{PC, Instruction};
pub use self::storage::Storage;
pub use self::params::*;
pub use self::patch::*;
pub use self::eval::{State, Machine, MachineStatus};
pub use self::commit::{AccountCommitment, Account, AccountState, BlockhashState};

use std::collections::hash_map;
use utils::bigint::M256;
use utils::gas::Gas;
use utils::address::Address;
use self::errors::{RequireError, CommitError, MachineError};

#[derive(Debug, Clone)]
/// VM Status
pub enum VMStatus {
    /// A running VM.
    Running,
    /// VM is stopped with out errors.
    ExitedOk,
    /// VM is stopped due to an error. The state of the VM is before
    /// the last failing instruction.
    ExitedErr(MachineError),
}

pub trait VM {
    fn commit_account(&mut self, commitment: AccountCommitment) -> Result<(), CommitError>;
    fn commit_blockhash(&mut self, number: M256, hash: M256) -> Result<(), CommitError>;
    fn status(&self) -> VMStatus;
    fn step(&mut self) -> Result<(), RequireError>;
    fn fire(&mut self) -> Result<(), RequireError> {
        loop {
            match self.status() {
                VMStatus::Running => self.step()?,
                VMStatus::ExitedOk | VMStatus::ExitedErr(_) => return Ok(()),
            }
        }
    }
    fn accounts(&self) -> hash_map::Values<Address, Account>;
    fn out(&self) -> &[u8];
    fn available_gas(&self) -> Gas;
    fn used_gas(&self) -> Gas;
    fn refunded_gas(&self) -> Gas;
    fn logs(&self) -> &[Log];
}

/// A sequencial VM. It uses sequencial memory representation and hash
/// map storage for accounts.
pub type SeqContextVM = ContextVM<SeqMemory>;

/// A VM that executes using a context and block information.
pub struct ContextVM<M> {
    machines: Vec<Machine<M>>,
    history: Vec<Context>
}

impl<M: Memory + Default> ContextVM<M> {
    /// Create a new VM using the given context, block header and patch.
    pub fn new(context: Context, block: BlockHeader, patch: &'static Patch) -> Self {
        let mut machines = Vec::new();
        machines.push(Machine::new(context, block, patch, 1));
        ContextVM {
            machines,
            history: Vec::new()
        }
    }

    /// Returns the call create history. Only used in testing.
    pub fn history(&self) -> &[Context] {
        self.history.as_slice()
    }
}

impl<M: Memory + Default> VM for ContextVM<M> {
    /// Commit an account information to this VM. This should only be
    /// used when receiving `RequireError`.
    fn commit_account(&mut self, commitment: AccountCommitment) -> Result<(), CommitError> {
        for machine in &mut self.machines {
            machine.commit_account(commitment.clone())?;
        }
        Ok(())
    }

    /// Commit a block hash to this VM. This should only be used when
    /// receiving `RequireError`.
    fn commit_blockhash(&mut self, number: M256, hash: M256) -> Result<(), CommitError> {
        for machine in &mut self.machines {
            machine.commit_blockhash(number, hash)?;
        }
        Ok(())
    }

    /// Returns the current status of the VM.
    fn status(&self) -> VMStatus {
        match self.machines[0].status() {
            MachineStatus::Running | MachineStatus::InvokeCreate(_) | MachineStatus::InvokeCall(_, _) => VMStatus::Running,
            MachineStatus::ExitedOk => VMStatus::ExitedOk,
            MachineStatus::ExitedErr(err) => VMStatus::ExitedErr(err.into()),
        }
    }

    /// Run one instruction and return. If it succeeds, VM status can
    /// still be `Running`. If the call stack has more than one items,
    /// this will only executes the last items' one single
    /// instruction.
    fn step(&mut self) -> Result<(), RequireError> {
        match self.machines.last().unwrap().status().clone() {
            MachineStatus::Running => {
                self.machines.last_mut().unwrap().step()
            },
            MachineStatus::ExitedOk | MachineStatus::ExitedErr(_) => {
                if self.machines.len() <= 1 {
                    Ok(())
                } else {
                    let finished = self.machines.pop().unwrap();
                    self.machines.last_mut().unwrap().apply_sub(finished);
                    Ok(())
                }
            },
            MachineStatus::InvokeCall(context, _) | MachineStatus::InvokeCreate(context) => {
                self.history.push(context.clone());
                let sub = self.machines.last().unwrap().derive(context);
                self.machines.push(sub);
                Ok(())
            },
        }
    }

    /// Run instructions until it reaches a `RequireError` or
    /// exits. If this function succeeds, the VM status can only be
    /// either `ExitedOk` or `ExitedErr`.
    fn fire(&mut self) -> Result<(), RequireError> {
        loop {
            match self.status() {
                VMStatus::Running => self.step()?,
                VMStatus::ExitedOk | VMStatus::ExitedErr(_) => return Ok(()),
            }
        }
    }

    /// Returns the changed or committed accounts information up to
    /// current execution status.
    fn accounts(&self) -> hash_map::Values<Address, Account> {
        self.machines[0].state().account_state.accounts()
    }

    /// Returns the out value, if any.
    fn out(&self) -> &[u8] {
        self.machines[0].state().out.as_slice()
    }

    /// Returns the available gas of this VM.
    fn available_gas(&self) -> Gas {
        self.machines[0].state().available_gas()
    }

    /// Returns the used gas of this VM.
    fn used_gas(&self) -> Gas {
        self.machines[0].state().used_gas
    }

    /// Returns the refunded gas of this VM.
    fn refunded_gas(&self) -> Gas {
        self.machines[0].state().refunded_gas
    }

    /// Returns logs to be appended to the current block if the user
    /// decided to accept the running status of this VM.
    fn logs(&self) -> &[Log] {
        self.machines[0].state().logs.as_slice()
    }
}
