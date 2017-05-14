use utils::bigint::M256;
use utils::gas::Gas;
use super::commit::{AccountState, BlockhashState};
use super::errors::{RequireError, MachineError, CommitError, EvalError, PCError};
use super::{Stack, Context, BlockHeader, Patch, PC, Storage, Memory, AccountCommitment, Log};

use self::check::check_opcode;
use self::run::run_opcode;
use self::cost::{gas_refund, gas_stipend, gas_cost, memory_cost, memory_gas};
use self::utils::copy_into_memory;

mod cost;
mod run;
mod check;
mod utils;

/// A VM state without PC.
pub struct State<M, S> {
    pub memory: M,
    pub stack: Stack,

    pub context: Context,
    pub block: BlockHeader,
    pub patch: Patch,

    pub out: Vec<u8>,

    pub memory_cost: Gas,
    pub used_gas: Gas,
    pub refunded_gas: Gas,

    pub account_state: AccountState<S>,
    pub blockhash_state: BlockhashState,
    pub logs: Vec<Log>,

    pub depth: usize,
}

impl<M, S> State<M, S> {
    pub fn memory_gas(&self) -> Gas {
        memory_gas(self.memory_cost)
    }

    pub fn available_gas(&self) -> Gas {
        self.context.gas_limit - self.memory_gas() - self.used_gas
    }
}

/// A VM state with PC.
pub struct Machine<M, S> {
    state: State<M, S>,
    pc: PC,
    status: MachineStatus,
}

#[derive(Debug, Clone)]
pub enum MachineStatus {
    Running,
    ExitedOk,
    ExitedErr(MachineError),
    InvokeCreate(Context),
    InvokeCall(Context, (M256, M256)),
}

#[derive(Debug, Clone)]
pub enum ControlCheck {
    Jump(M256),
}

#[derive(Debug, Clone)]
pub enum Control {
    Stop,
    Jump(M256),
    InvokeCreate(Context),
    InvokeCall(Context, (M256, M256)),
}

impl<M: Memory + Default, S: Storage + Default + Clone> Machine<M, S> {
    pub fn new(context: Context, block: BlockHeader, patch: Patch, depth: usize) -> Self {
        Machine {
            pc: PC::new(context.code.as_slice()),
            status: MachineStatus::Running,
            state: State {
                memory: M::default(),
                stack: Stack::default(),

                context: context,
                block: block,
                patch: patch,

                out: Vec::new(),

                memory_cost: Gas::zero(),
                used_gas: Gas::zero(),
                refunded_gas: Gas::zero(),

                account_state: AccountState::default(),
                blockhash_state: BlockhashState::default(),
                logs: Vec::new(),

                depth: depth,
            },
        }
    }

    pub fn derive(&self, context: Context) -> Self {
        Machine {
            pc: PC::new(context.code.as_slice()),
            status: MachineStatus::Running,
            state: State {
                memory: M::default(),
                stack: Stack::default(),

                context: context,
                block: self.state.block.clone(),
                patch: self.state.patch.clone(),

                out: Vec::new(),

                memory_cost: Gas::zero(),
                used_gas: Gas::zero(),
                refunded_gas: Gas::zero(),

                account_state: self.state.account_state.clone(),
                blockhash_state: self.state.blockhash_state.clone(),
                logs: self.state.logs.clone(),

                depth: self.state.depth + 1,
            },
        }
    }

    pub fn commit_account(&mut self, commitment: AccountCommitment<S>) -> Result<(), CommitError> {
        self.state.account_state.commit(commitment)
    }

    pub fn commit_blockhash(&mut self, number: M256, hash: M256) -> Result<(), CommitError> {
        self.state.blockhash_state.commit(number, hash)
    }

    #[allow(unused_variables)]
    pub fn apply_sub(&mut self, sub: Machine<M, S>) {
        use std::mem::swap;
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

    fn apply_create(&mut self, sub: Machine<M, S>) {
        match sub.status() {
            MachineStatus::ExitedOk => {
                if self.state.available_gas() < sub.state.used_gas {
                    self.state.stack.pop().unwrap();
                    self.state.stack.push(M256::zero()).unwrap();
                }

                self.state.account_state = sub.state.account_state;
                self.state.blockhash_state = sub.state.blockhash_state;
                self.state.logs = sub.state.logs;
                self.state.used_gas = self.state.used_gas + sub.state.used_gas;
                self.state.refunded_gas = self.state.refunded_gas + sub.state.refunded_gas;
                self.state.account_state.decrease_balance(self.state.context.address,
                                                          sub.state.context.value);
                self.state.account_state.create(sub.state.context.address,
                                                sub.state.context.value,
                                                sub.state.out.as_slice());
            },
            MachineStatus::ExitedErr(_) => {
                self.state.stack.pop().unwrap();
                self.state.stack.push(M256::zero()).unwrap();
            },
            _ => panic!(),
        }
    }

    fn apply_call(&mut self, sub: Machine<M, S>, out_start: M256, out_len: M256) {
        match sub.status() {
            MachineStatus::ExitedOk => {
                if self.state.available_gas() < sub.state.used_gas {
                    self.state.stack.pop().unwrap();
                    self.state.stack.push(M256::from(1u64)).unwrap();
                }

                self.state.account_state = sub.state.account_state;
                self.state.blockhash_state = sub.state.blockhash_state;
                self.state.logs = sub.state.logs;
                self.state.used_gas = self.state.used_gas + sub.state.used_gas;
                self.state.refunded_gas = self.state.refunded_gas + sub.state.refunded_gas;
                self.state.account_state.decrease_balance(self.state.context.address,
                                                          sub.state.context.value);
                self.state.account_state.increase_balance(sub.state.context.address,
                                                          sub.state.context.value);
                copy_into_memory(&mut self.state.memory, sub.state.out.as_slice(),
                                 out_start, M256::zero(), out_len);
            },
            MachineStatus::ExitedErr(_) => {
                self.state.stack.pop().unwrap();
                self.state.stack.push(M256::from(1u64)).unwrap();
            },
            _ => panic!(),
        }
    }

    pub fn check(&self) -> Result<(), EvalError> {
        let instruction = self.pc.peek()?;
        check_opcode(instruction, &self.state).and_then(|v| {
            match v {
                None => Ok(()),
                Some(ControlCheck::Jump(dest)) => {
                    if dest <= M256::from(usize::max_value()) && self.pc.is_valid(dest.into()) {
                        Ok(())
                    } else {
                        Err(EvalError::Machine(MachineError::PC(PCError::BadJumpDest)))
                    }
                }
            }
        })
    }

    pub fn step(&mut self) -> Result<(), RequireError> {
        match &self.status {
            &MachineStatus::Running => (),
            _ => panic!(),
        }

        if self.pc.is_end() {
            self.status = MachineStatus::ExitedOk;
            return Ok(());
        }

        match self.check() {
            Ok(()) => (),
            Err(EvalError::Machine(error)) => {
                self.status = MachineStatus::ExitedErr(error);
                return Ok(());
            },
            Err(EvalError::Require(error)) => {
                return Err(error);
            },
        };

        let instruction = self.pc.peek().unwrap();
        let position = self.pc.position();
        let memory_cost = memory_cost(instruction, &self.state);
        let memory_gas = memory_gas(memory_cost);
        let gas_cost = gas_cost(instruction, &self.state);
        let gas_stipend = gas_stipend(instruction, &self.state);
        let gas_refund = gas_refund(instruction, &self.state);

        if self.state.context.gas_limit < memory_gas + self.state.used_gas + gas_cost - gas_stipend {
            self.status = MachineStatus::ExitedErr(MachineError::EmptyGas);
            return Ok(());
        }

        let instruction = self.pc.read().unwrap();
        let after_gas = self.state.context.gas_limit - memory_gas - self.state.used_gas - gas_cost + gas_stipend;
        let result = run_opcode((instruction, position),
                                &mut self.state, gas_stipend, after_gas);

        self.state.used_gas = self.state.used_gas + gas_cost - gas_stipend;
        self.state.memory_cost = memory_cost;
        self.state.refunded_gas = self.state.refunded_gas + gas_refund;

        match result {
            None => Ok(()),
            Some(Control::Jump(dest)) => {
                self.pc.jump(dest.into()).unwrap();
                Ok(())
            },
            Some(Control::InvokeCall(context, (from, len))) => {
                self.status = MachineStatus::InvokeCall(context, (from, len));
                Ok(())
            },
            Some(Control::InvokeCreate(context)) => {
                self.status = MachineStatus::InvokeCreate(context);
                Ok(())
            },
            Some(Control::Stop) => {
                self.status = MachineStatus::ExitedOk;
                Ok(())
            },
        }
    }

    pub fn state(&self) -> &State<M, S> {
        &self.state
    }

    pub fn status(&self) -> MachineStatus {
        self.status.clone()
    }
}
