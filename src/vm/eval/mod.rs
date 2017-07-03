//! VM Runtime
use util::bigint::M256;
use util::gas::Gas;
use util::address::Address;
use super::commit::{AccountState, BlockhashState};
use super::errors::{RequireError, MachineError, CommitError, EvalError, PCError};
use super::{Stack, Context, BlockHeader, Patch, PC, Memory, AccountCommitment, Log};

use self::check::{check_opcode, extra_check_opcode};
use self::run::run_opcode;
use self::cost::{gas_refund, gas_stipend, gas_cost, memory_cost, memory_gas};

mod cost;
mod run;
mod check;
mod util;
mod lifecycle;
mod precompiled;

/// A VM state without PC.
pub struct State<M> {
    /// Memory of this runtime.
    pub memory: M,
    /// Stack of this runtime.
    pub stack: Stack,

    /// Context.
    pub context: Context,
    /// Block header.
    pub block: BlockHeader,
    /// Patch that is used by this runtime.
    pub patch: &'static Patch,

    /// The current out value.
    pub out: Vec<u8>,

    /// The current memory cost. Note that this is different from
    /// memory gas.
    pub memory_cost: Gas,
    /// Used gas excluding memory gas.
    pub used_gas: Gas,
    /// Refunded gas.
    pub refunded_gas: Gas,

    /// The current account commitment states.
    pub account_state: AccountState,
    /// The current blockhash commitment states.
    pub blockhash_state: BlockhashState,
    /// Logs appended.
    pub logs: Vec<Log>,
    /// Removed accounts using the SUICIDE opcode.
    pub removed: Vec<Address>,

    /// Depth of this runtime.
    pub depth: usize,
}

impl<M> State<M> {
    /// Memory gas, part of total used gas.
    pub fn memory_gas(&self) -> Gas {
        memory_gas(self.memory_cost)
    }

    /// Available gas at this moment.
    pub fn available_gas(&self) -> Gas {
        self.context.gas_limit - self.memory_gas() - self.used_gas
    }

    /// Total used gas including the memory gas.
    pub fn total_used_gas(&self) -> Gas {
        self.memory_gas() + self.used_gas
    }
}

/// A VM state with PC.
pub struct Machine<M> {
    state: State<M>,
    pc: PC,
    status: MachineStatus,
}

#[derive(Debug, Clone)]
/// Represents the current runtime status.
pub enum MachineStatus {
    /// This runtime is actively running or has just been started.
    Running,
    /// This runtime has exited successfully. Calling `step` on this
    /// runtime again would panic.
    ExitedOk,
    /// This runtime has exited with errors. Calling `step` on this
    /// runtime again would panic.
    ExitedErr(MachineError),
    /// This runtime requires execution of a sub runtime, which is a
    /// ContractCreation instruction.
    InvokeCreate(Context),
    /// This runtime requires execution of a sub runtime, which is a
    /// MessageCall instruction.
    InvokeCall(Context, (M256, M256)),
}

#[derive(Debug, Clone)]
/// Used for `check` for additional checks related to the runtime.
pub enum ControlCheck {
    Jump(M256),
}

#[derive(Debug, Clone)]
/// Used for `step` for additional operations related to the runtime.
pub enum Control {
    Stop,
    Jump(M256),
    InvokeCreate(Context),
    InvokeCall(Context, (M256, M256)),
}

impl<M: Memory + Default> Machine<M> {
    /// Create a new runtime.
    pub fn new(context: Context, block: BlockHeader, patch: &'static Patch, depth: usize) -> Self {
        Self::with_states(context, block, patch, depth,
                          AccountState::default(), BlockhashState::default())
    }

    /// Create a new runtime with the given states.
    pub fn with_states(context: Context, block: BlockHeader, patch: &'static Patch,
                       depth: usize, account_state: AccountState,
                       blockhash_state: BlockhashState) -> Self {
        Machine {
            pc: PC::new(context.code.as_slice()),
            status: MachineStatus::Running,
            state: State {
                memory: M::default(),
                stack: Stack::default(),

                context,
                block,
                patch,

                out: Vec::new(),

                memory_cost: Gas::zero(),
                used_gas: Gas::zero(),
                refunded_gas: Gas::zero(),

                account_state,
                blockhash_state,
                logs: Vec::new(),
                removed: Vec::new(),

                depth,
            },
        }
    }

    /// Derive this runtime to create a sub runtime. This will not
    /// modify the current runtime, and it will have a chance to
    /// review whether it wants to accept the result of this sub
    /// runtime afterwards.
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
                removed: self.state.removed.clone(),

                depth: self.state.depth + 1,
            },
        }
    }


    /// Commit a new account into this runtime.
    pub fn commit_account(&mut self, commitment: AccountCommitment) -> Result<(), CommitError> {
        self.state.account_state.commit(commitment)
    }

    /// Commit a new blockhash into this runtime.
    pub fn commit_blockhash(&mut self, number: M256, hash: M256) -> Result<(), CommitError> {
        self.state.blockhash_state.commit(number, hash)
    }

    /// Check the next instruction about whether it will return
    /// errors.
    pub fn check(&self) -> Result<(), EvalError> {
        let instruction = self.pc.peek()?;
        check_opcode(instruction, &self.state).and_then(|v| {
            match v {
                None => Ok(()),
                Some(ControlCheck::Jump(dest)) => {
                    if dest <= M256::from(usize::max_value()) && self.pc.is_valid(dest.as_usize()) {
                        Ok(())
                    } else {
                        Err(EvalError::Machine(MachineError::PC(PCError::BadJumpDest)))
                    }
                }
            }
        })
    }

    /// Step an instruction in the PC. The eval result is refected by
    /// the runtime status, and it will only return an error if
    /// there're accounts or blockhashes to be committed to this
    /// runtime for it to run. In that case, the state of the current
    /// runtime will not be affected.
    pub fn step(&mut self) -> Result<(), RequireError> {
        match &self.status {
            &MachineStatus::Running => (),
            _ => panic!(),
        }

        if self.step_precompiled() {
            return Ok(());
        }

        if self.state.depth >= self.state.patch.callstack_limit {
            self.status = MachineStatus::ExitedErr(MachineError::CallstackOverflow);
            return Ok(());
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

        let all_gas_cost = memory_gas + self.state.used_gas + gas_cost - gas_stipend;
        if self.state.context.gas_limit < all_gas_cost {
            self.status = MachineStatus::ExitedErr(MachineError::EmptyGas);
            return Ok(());
        }

        let after_gas = self.state.context.gas_limit - all_gas_cost;

        match extra_check_opcode(instruction, &self.state, gas_stipend, after_gas) {
            Ok(()) => (),
            Err(EvalError::Machine(error)) => {
                self.status = MachineStatus::ExitedErr(error);
                return Ok(());
            },
            Err(EvalError::Require(error)) => {
                return Err(error);
            },
        }

        let instruction = self.pc.read().unwrap();
        let result = run_opcode((instruction, position),
                                &mut self.state, gas_stipend, after_gas);

        self.state.used_gas = self.state.used_gas + gas_cost - gas_stipend;
        self.state.memory_cost = memory_cost;
        self.state.refunded_gas = self.state.refunded_gas + gas_refund;

        match result {
            None => Ok(()),
            Some(Control::Jump(dest)) => {
                self.pc.jump(dest.as_usize()).unwrap();
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

    /// Get the runtime state.
    pub fn state(&self) -> &State<M> {
        &self.state
    }

    /// Get the current runtime status.
    pub fn status(&self) -> MachineStatus {
        self.status.clone()
    }
}
