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
pub use self::transaction::{Transaction, TransactionVM};

use std::collections::hash_map;
use util::bigint::M256;
use util::gas::Gas;
use util::address::Address;
use self::errors::{RequireError, CommitError, MachineError};

#[derive(Debug, Clone)]
/// VM Status
pub enum VMStatus {
    /// A running VM.
    Running,
    /// VM is stopped without errors.
    ExitedOk,
    /// VM is stopped due to an error. The state of the VM is before
    /// the last failing instruction.
    ExitedErr(MachineError),
}

/// Represents an EVM. This is usually the main interface for clients
/// to interact with.
pub trait VM {
    /// Commit an account information to this VM. This should only
    /// be used when receiving `RequireError`.
    fn commit_account(&mut self, commitment: AccountCommitment) -> Result<(), CommitError>;
    /// Commit a block hash to this VM. This should only be used when
    /// receiving `RequireError`.
    fn commit_blockhash(&mut self, number: M256, hash: M256) -> Result<(), CommitError>;
    /// Returns the current status of the VM.
    fn status(&self) -> VMStatus;
    /// Run one instruction and return. If it succeeds, VM status can
    /// still be `Running`. If the call stack has more than one items,
    /// this will only executes the last items' one single
    /// instruction.
    fn step(&mut self) -> Result<(), RequireError>;
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
    fn accounts(&self) -> hash_map::Values<Address, Account>;
    /// Returns the out value, if any.
    fn out(&self) -> &[u8];
    /// Returns the available gas of this VM.
    fn available_gas(&self) -> Gas;
    /// Returns the refunded gas of this VM.
    fn refunded_gas(&self) -> Gas;
    /// Returns logs to be appended to the current block if the user
    /// decided to accept the running status of this VM.
    fn logs(&self) -> &[Log];
    /// Returns all removed account addresses as for current VM execution.
    fn removed(&self) -> &[Address];
}

/// A sequencial VM. It uses sequencial memory representation and hash
/// map storage for accounts.
pub type SeqContextVM = ContextVM<SeqMemory>;
/// A sequencial transaction VM. This is same as `SeqContextVM` except
/// it runs at transaction level.
pub type SeqTransactionVM = TransactionVM<SeqMemory>;

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

    /// Create a new VM with the given account state and blockhash state.
    pub fn with_states(context: Context, block: BlockHeader, patch: &'static Patch,
                       account_state: AccountState, blockhash_state: BlockhashState) -> Self {
        let mut machines = Vec::new();
        machines.push(Machine::with_states(context, block, patch, 1, account_state, blockhash_state));
        ContextVM {
            machines,
            history: Vec::new()
        }
    }

    /// Create a new VM with the result of the previous VM. This is
    /// usually used by transaction for chainning them.
    pub fn with_previous(context: Context, block: BlockHeader, patch: &'static Patch,
                         vm: &ContextVM<M>) -> Self {
        Self::with_states(context, block, patch,
                          vm.machines[0].state().account_state.clone(),
                          vm.machines[0].state().blockhash_state.clone())
    }

    /// Returns the call create history. Only used in testing.
    pub fn history(&self) -> &[Context] {
        self.history.as_slice()
    }
}

impl<M: Memory + Default> VM for ContextVM<M> {
    fn commit_account(&mut self, commitment: AccountCommitment) -> Result<(), CommitError> {
        for machine in &mut self.machines {
            machine.commit_account(commitment.clone())?;
        }
        Ok(())
    }

    fn commit_blockhash(&mut self, number: M256, hash: M256) -> Result<(), CommitError> {
        for machine in &mut self.machines {
            machine.commit_blockhash(number, hash)?;
        }
        Ok(())
    }

    fn status(&self) -> VMStatus {
        match self.machines[0].status() {
            MachineStatus::Running | MachineStatus::InvokeCreate(_) | MachineStatus::InvokeCall(_, _) => VMStatus::Running,
            MachineStatus::ExitedOk => VMStatus::ExitedOk,
            MachineStatus::ExitedErr(err) => VMStatus::ExitedErr(err.into()),
        }
    }

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
            MachineStatus::InvokeCall(context, _) => {
                self.history.push(context.clone());
                let mut sub = self.machines.last().unwrap().derive(context);
                sub.invoke_call();
                self.machines.push(sub);
                Ok(())
            },
            MachineStatus::InvokeCreate(context) => {
                let mut sub = self.machines.last().unwrap().derive(context.clone());
                sub.invoke_create()?;
                self.history.push(context);
                self.machines.push(sub);
                Ok(())
            },
        }
    }

    fn fire(&mut self) -> Result<(), RequireError> {
        loop {
            match self.status() {
                VMStatus::Running => self.step()?,
                VMStatus::ExitedOk | VMStatus::ExitedErr(_) => return Ok(()),
            }
        }
    }

    fn accounts(&self) -> hash_map::Values<Address, Account> {
        self.machines[0].state().account_state.accounts()
    }

    fn out(&self) -> &[u8] {
        self.machines[0].state().out.as_slice()
    }

    fn available_gas(&self) -> Gas {
        self.machines[0].state().available_gas()
    }

    fn refunded_gas(&self) -> Gas {
        self.machines[0].state().refunded_gas
    }

    fn logs(&self) -> &[Log] {
        self.machines[0].state().logs.as_slice()
    }

    fn removed(&self) -> &[Address] {
        self.machines[0].state().removed.as_slice()
    }
}
