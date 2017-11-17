//! SputnikVM implementation, traits and structs
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

#![deny(unused_import_braces, unused_imports,
        unused_comparisons, unused_must_use,
        unused_variables, non_shorthand_field_patterns,
        unreachable_code)]

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]

#[cfg(not(feature = "std"))]
extern crate alloc;

extern crate rlp;
extern crate bigint;
extern crate block_core;
extern crate sha3;
extern crate ripemd160;
extern crate sha2;
extern crate digest;

#[cfg(feature = "c-secp256k1")]
extern crate secp256k1;

#[cfg(feature = "rust-secp256k1")]
extern crate secp256k1;

#[cfg(feature = "std")]
extern crate block;

mod util;
mod memory;
mod stack;
mod pc;
mod params;
mod eval;
mod commit;
mod patch;
mod transaction;
pub mod errors;

pub use self::memory::{Memory, SeqMemory};
pub use self::stack::Stack;
pub use self::pc::{PC, PCMut, Instruction, Valids};
pub use self::params::*;
pub use self::patch::*;
pub use self::eval::{State, Machine, Runtime, MachineStatus};
pub use self::commit::{AccountCommitment, AccountChange, AccountState, BlockhashState, Storage};
pub use self::transaction::{ValidTransaction, TransactionVM};
pub use self::errors::{OnChainError, NotSupportedError, RequireError, CommitError, PreExecutionError};
pub use self::util::opcode::Opcode;

#[cfg(not(feature = "std"))]
use alloc::Vec;

#[cfg(feature = "std")] use std::collections::{HashSet as Set, hash_map as map};
#[cfg(not(feature = "std"))] use alloc::{BTreeSet as Set, btree_map as map};
use bigint::{U256, H256, Gas, Address};

#[derive(Debug, Clone)]
/// VM Status
pub enum VMStatus {
    /// A running VM.
    Running,
    /// VM is stopped without errors.
    ExitedOk,
    /// VM is stopped due to an error. The state of the VM is before
    /// the last failing instruction.
    ExitedErr(OnChainError),
    ExitedNotSupported(NotSupportedError),
}

/// Represents an EVM. This is usually the main interface for clients
/// to interact with.
pub trait VM {
    /// Commit an account information to this VM. This should only
    /// be used when receiving `RequireError`.
    fn commit_account(&mut self, commitment: AccountCommitment) -> Result<(), CommitError>;
    /// Commit a block hash to this VM. This should only be used when
    /// receiving `RequireError`.
    fn commit_blockhash(&mut self, number: U256, hash: H256) -> Result<(), CommitError>;
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
                VMStatus::ExitedOk | VMStatus::ExitedErr(_) |
                VMStatus::ExitedNotSupported(_) => return Ok(()),
            }
        }
    }
    /// Returns the changed or committed accounts information up to
    /// current execution status.
    fn accounts(&self) -> map::Values<Address, AccountChange>;
    /// Returns all fetched or modified addresses.
    fn used_addresses(&self) -> Set<Address>;
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
pub type SeqContextVM<P> = ContextVM<SeqMemory<P>, P>;
/// A sequencial transaction VM. This is same as `SeqContextVM` except
/// it runs at transaction level.
pub type SeqTransactionVM<P> = TransactionVM<SeqMemory<P>, P>;

/// A VM that executes using a context and block information.
pub struct ContextVM<M, P: Patch> {
    runtime: Runtime,
    machines: Vec<Machine<M, P>>,
    history: Vec<Context>
}

impl<M: Memory + Default, P: Patch> ContextVM<M, P> {
    /// Create a new VM using the given context, block header and patch.
    pub fn new(context: Context, block: HeaderParams) -> Self {
        let mut machines = Vec::new();
        machines.push(Machine::new(context, 1));
        ContextVM {
            machines,
            runtime: Runtime::new(block),
            history: Vec::new()
        }
    }

    /// Create a new VM with the given account state and blockhash state.
    pub fn with_states(context: Context, block: HeaderParams,
                       account_state: AccountState<P::Account>, blockhash_state: BlockhashState) -> Self {
        let mut machines = Vec::new();
        machines.push(Machine::with_states(context, 1, account_state));
        ContextVM {
            machines,
            runtime: Runtime::with_states(block, blockhash_state),
            history: Vec::new()
        }
    }

    /// Create a new VM with the result of the previous VM. This is
    /// usually used by transaction for chainning them.
    pub fn with_previous(context: Context, block: HeaderParams, vm: &ContextVM<M, P>) -> Self {
        Self::with_states(context, block,
                          vm.machines[0].state().account_state.clone(),
                          vm.runtime.blockhash_state.clone())
    }

    /// Returns the call create history. Only used in testing.
    pub fn history(&self) -> &[Context] {
        self.history.as_slice()
    }

    /// Returns the current state of the VM.
    pub fn current_state(&self) -> &State<M, P> {
        self.current_machine().state()
    }

    /// Returns the current runtime machine.
    pub fn current_machine(&self) -> &Machine<M, P> {
        self.machines.last().unwrap()
    }
}

impl<M: Memory + Default, P: Patch> VM for ContextVM<M, P> {
    fn commit_account(&mut self, commitment: AccountCommitment) -> Result<(), CommitError> {
        for machine in &mut self.machines {
            machine.commit_account(commitment.clone())?;
        }
        Ok(())
    }

    fn commit_blockhash(&mut self, number: U256, hash: H256) -> Result<(), CommitError> {
        self.runtime.blockhash_state.commit(number, hash)
    }

    fn status(&self) -> VMStatus {
        match self.machines.last().unwrap().status().clone() {
            MachineStatus::ExitedNotSupported(err) => return VMStatus::ExitedNotSupported(err),
            _ => (),
        }

        match self.machines[0].status() {
            MachineStatus::Running | MachineStatus::InvokeCreate(_) | MachineStatus::InvokeCall(_, _) => VMStatus::Running,
            MachineStatus::ExitedOk => VMStatus::ExitedOk,
            MachineStatus::ExitedErr(err) => VMStatus::ExitedErr(err.into()),
            MachineStatus::ExitedNotSupported(err) => VMStatus::ExitedNotSupported(err),
        }
    }

    fn step(&mut self) -> Result<(), RequireError> {
        match self.machines.last().unwrap().status().clone() {
            MachineStatus::Running => {
                self.machines.last_mut().unwrap().step(&self.runtime)
            },
            MachineStatus::ExitedOk | MachineStatus::ExitedErr(_) => {
                if self.machines.len() == 0 {
                    panic!()
                } else if self.machines.len() == 1 {
                    Ok(())
                } else {
                    let finished = self.machines.pop().unwrap();
                    self.machines.last_mut().unwrap().apply_sub(finished);
                    Ok(())
                }
            },
            MachineStatus::ExitedNotSupported(_) => {
                Ok(())
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
                VMStatus::ExitedOk | VMStatus::ExitedErr(_) |
                VMStatus::ExitedNotSupported(_) => return Ok(()),
            }
        }
    }

    fn accounts(&self) -> map::Values<Address, AccountChange> {
        self.machines[0].state().account_state.accounts()
    }

    fn used_addresses(&self) -> Set<Address> {
        self.machines[0].state().account_state.used_addresses()
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
