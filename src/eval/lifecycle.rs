//! Runtime lifecycle related functionality.

#[cfg(not(feature = "std"))]
use alloc::Vec;

#[cfg(not(feature = "std"))] use alloc::rc::Rc;
#[cfg(feature = "std")] use std::rc::Rc;

use bigint::{U256, M256, Gas, Address};
use errors::{RequireError, OnChainError};
use commit::AccountState;
use ::{Memory, Patch};
use super::{Machine, MachineStatus};
use super::util::copy_into_memory_apply;
use super::cost::code_deposit_gas;

/// # Lifecycle of a Machine
///
/// When a new non-invoked transaction is created, `initialize_call`
/// or `initialize_create` should be called. After this, the machine
/// can be stepped as normal. When the machine meets a CALL/CALLCODE
/// or CREATE instruction, a sub-machine will be created. This
/// submachine should first call `invoke_call` or
/// `invoke_create`. After the submachine is finished, it should call
/// `apply_sub`. When the non-invoked transaction is finished, it
/// should first call `code_deposit` if it is a contract creation
/// transaction. After that, it should call `finalize`.

impl<M: Memory + Default, P: Patch> Machine<M, P> {
    /// Initialize a MessageCall transaction.
    pub fn initialize_call(&mut self, preclaimed_value: U256) {
        self.state.account_state.premark_exists(self.state.context.address);

        if !self.state.context.is_system {
            self.state.account_state.decrease_balance(self.state.context.caller, preclaimed_value);
            self.state.account_state.decrease_balance(self.state.context.caller, self.state.context.value);
        }
        self.state.account_state.increase_balance(self.state.context.address, self.state.context.value);
    }

    /// Initialize the runtime as a call from a CALL or CALLCODE opcode.
    pub fn invoke_call(&mut self) {
        self.state.account_state.premark_exists(self.state.context.address);

        if !self.state.context.is_system {
            self.state.account_state.decrease_balance(self.state.context.caller, self.state.context.value);
        }
        self.state.account_state.increase_balance(self.state.context.address, self.state.context.value);
    }

    /// Initialize a ContractCreation transaction.
    pub fn initialize_create(&mut self, preclaimed_value: U256) -> Result<(), RequireError> {
        self.state.account_state.require(self.state.context.address)?;

        self.state.account_state.premark_exists(self.state.context.address);
        if !self.state.context.is_system {
            self.state.account_state.decrease_balance(self.state.context.caller, preclaimed_value);
            self.state.account_state.decrease_balance(self.state.context.caller, self.state.context.value);
        }
        self.state.account_state.create(self.state.context.address, self.state.context.value).unwrap();

        Ok(())
    }

    /// Initialize the runtime as a call from a CREATE opcode.
    pub fn invoke_create(&mut self) -> Result<(), RequireError> {
        self.state.account_state.require(self.state.context.address)?;

        self.state.account_state.premark_exists(self.state.context.address);
        if !self.state.context.is_system {
            self.state.account_state.decrease_balance(self.state.context.caller, self.state.context.value);
        }
        self.state.account_state.create(self.state.context.address, self.state.context.value).unwrap();

        Ok(())
    }

    /// Deposit code for a ContractCreation transaction or a CREATE opcode.
    pub fn code_deposit(&mut self) {
        match self.status() {
            MachineStatus::ExitedOk | MachineStatus::ExitedErr(_) => (),
            _ => panic!(),
        }

        let deposit_cost = code_deposit_gas(self.state.out.len());
        if deposit_cost > self.state.available_gas() {
            if !P::force_code_deposit() {
                self.status = MachineStatus::ExitedErr(OnChainError::EmptyGas);
            } else {
                self.state.account_state.code_deposit(self.state.context.address, Rc::new(Vec::new()));
            }
        } else {
            self.state.used_gas = self.state.used_gas + deposit_cost;
            self.state.account_state.code_deposit(self.state.context.address,
                                                  self.state.out.clone());
        }
    }

    /// Finalize a transaction. This should not be used when invoked
    /// by an opcode.
    pub fn finalize(&mut self, beneficiary: Address, real_used_gas: Gas, preclaimed_value: U256, fresh_account_state: &AccountState<P::Account>) -> Result<(), RequireError> {
        self.state.account_state.require(self.state.context.address)?;

        match self.status() {
            MachineStatus::ExitedOk => {
                // Requires removed accounts to exist.
                for address in &self.state.removed {
                    self.state.account_state.require(*address)?;
                }
            },
            MachineStatus::ExitedErr(_) => {
                // If exited with error, reset all changes.
                self.state.account_state = fresh_account_state.clone();
                if !self.state.context.is_system {
                    self.state.account_state.decrease_balance(self.state.context.caller, preclaimed_value);
                }
                self.state.logs = Vec::new();
                self.state.removed = Vec::new();
            },
            _ => panic!(),
        }

        let gas_dec = real_used_gas * self.state.context.gas_price;
        if !self.state.context.is_system {
            self.state.account_state.increase_balance(self.state.context.caller, preclaimed_value);
            self.state.account_state.decrease_balance(self.state.context.caller, gas_dec.into());
        }

        // Apply miner rewards
        self.state.account_state.increase_balance(beneficiary, gas_dec.into());

        for address in &self.state.removed {
            self.state.account_state.remove(*address).unwrap();
        }

        match self.status() {
            MachineStatus::ExitedOk => Ok(()),
            MachineStatus::ExitedErr(_) => Ok(()),
            _ => panic!(),
        }
    }

    /// Apply a sub runtime into the current runtime. This sub runtime
    /// should have been created by the current runtime's `derive`
    /// function. Depending whether the current runtime is invoking a
    /// ContractCreation or MessageCall instruction, it will apply
    /// various states back.
    pub fn apply_sub(&mut self, sub: Machine<M, P>) {
        #[cfg(feature = "std")]
        use std::mem::swap;

        #[cfg(not(feature = "std"))]
        use core::mem::swap;

        let mut status = MachineStatus::Running;
        swap(&mut status, &mut self.status);
        match status {
            MachineStatus::InvokeCreate(_) => {
                self.apply_create(sub);
            },
            MachineStatus::InvokeCall(_, (out_start, out_len)) => {
                self.apply_call(sub, out_start, out_len);
            },
            _ => panic!(),
        }
    }

    fn apply_create(&mut self, mut sub: Machine<M, P>) {
        if self.state.available_gas() < sub.state.used_gas {
            panic!();
        }

        sub.code_deposit();

        match sub.status() {
            MachineStatus::ExitedOk => {
                let sub_total_used_gas = sub.state.total_used_gas();

                self.state.account_state = sub.state.account_state;
                self.state.logs = sub.state.logs;
                self.state.removed = sub.state.removed;
                self.state.used_gas = self.state.used_gas + sub_total_used_gas;
                self.state.refunded_gas = self.state.refunded_gas + sub.state.refunded_gas;

            },
            MachineStatus::ExitedErr(_) => {
                self.state.used_gas = self.state.used_gas + sub.state.context.gas_limit;
                self.state.stack.pop().unwrap();
                self.state.stack.push(M256::zero()).unwrap();
            },
            _ => panic!(),
        }
    }

    fn apply_call(&mut self, sub: Machine<M, P>, out_start: U256, out_len: U256) {
        if self.state.available_gas() < sub.state.used_gas {
            panic!();
        }

        match sub.status() {
            MachineStatus::ExitedOk => {
                let sub_total_used_gas = sub.state.total_used_gas();

                self.state.account_state = sub.state.account_state;
                self.state.logs = sub.state.logs;
                self.state.removed = sub.state.removed;
                self.state.used_gas = self.state.used_gas + sub_total_used_gas;
                self.state.refunded_gas = self.state.refunded_gas + sub.state.refunded_gas;
                copy_into_memory_apply(&mut self.state.memory, sub.state.out.as_slice(),
                                       out_start, out_len);
            },
            MachineStatus::ExitedErr(_) => {
                self.state.used_gas = self.state.used_gas + sub.state.context.gas_limit;
                self.state.stack.pop().unwrap();
                self.state.stack.push(M256::zero()).unwrap();
            },
            _ => panic!(),
        }
    }
}
