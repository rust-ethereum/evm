//! VM Runtime

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(not(feature = "std"))] use alloc::rc::Rc;
#[cfg(not(feature = "std"))] use alloc::boxed::Box;
#[cfg(feature = "std")] use std::rc::Rc;

#[cfg(not(feature = "std"))] use core::ops::AddAssign;
#[cfg(feature = "std")] use std::ops::AddAssign;

use bigint::{M256, U256, Gas, Address};
use super::pc::Instruction;
use super::commit::{AccountState, BlockhashState};
use super::errors::{RequireError, RuntimeError, CommitError, EvalOnChainError,
                    OnChainError, NotSupportedError};
use super::{Stack, Context, HeaderParams, Patch, PC, PCMut, Valids, Memory,
            AccountCommitment, Log, Opcode};

use self::check::{check_opcode, check_static, check_support, extra_check_opcode};
use self::run::run_opcode;
use self::cost::{gas_refund, gas_stipend, gas_cost, memory_cost, memory_gas};

macro_rules! reset_error_hard {
    ($self: expr, $err: expr) => {
        $self.status = MachineStatus::ExitedErr($err);
        $self.state.used_gas = GasUsage::All;
        $self.state.refunded_gas = Gas::zero();
        $self.state.logs = Vec::new();
        $self.state.out = Rc::new(Vec::new());
    }
}

macro_rules! reset_error_revert {
    ($self: expr) => {
        $self.status = MachineStatus::ExitedErr(OnChainError::Revert);
    }
}

macro_rules! reset_error_not_supported {
    ($self: expr, $err: expr) => {
        $self.status = MachineStatus::ExitedNotSupported($err);
        $self.state.used_gas = GasUsage::Some(Gas::zero());
        $self.state.refunded_gas = Gas::zero();
        $self.state.logs = Vec::new();
        $self.state.out = Rc::new(Vec::new());
    }
}

mod cost;
mod run;
mod check;
mod util;
mod lifecycle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GasUsage {
    All,
    Some(Gas),
}

impl AddAssign<Gas> for GasUsage {
    fn add_assign(&mut self, rhs: Gas) {
        match self {
            GasUsage::All => (),
            GasUsage::Some(ref mut gas) => {
                *gas = *gas + rhs;
            }
        }
    }
}

/// A VM state without PC.
pub struct State<M, P: Patch> {
    /// Memory of this runtime.
    pub memory: M,
    /// Stack of this runtime.
    pub stack: Stack,

    /// Context.
    pub context: Context,

    /// The current out value.
    pub out: Rc<Vec<u8>>,
    /// Return data buffer.
    pub ret: Rc<Vec<u8>>,

    /// The current memory cost. Note that this is different from
    /// memory gas.
    pub memory_cost: Gas,
    /// Used gas excluding memory gas.
    pub used_gas: GasUsage,
    /// Refunded gas.
    pub refunded_gas: Gas,

    /// The current account commitment states.
    pub account_state: AccountState<P::Account>,
    /// Logs appended.
    pub logs: Vec<Log>,
    /// All removed accounts using the SUICIDE opcode.
    pub removed: Vec<Address>,

    /// Depth of this runtime.
    pub depth: usize,

    /// Code valid maps.
    pub valids: Valids,
    /// PC position.
    pub position: usize,
}

impl<M, P: Patch> State<M, P> {
    /// Memory gas, part of total used gas.
    pub fn memory_gas(&self) -> Gas {
        memory_gas(self.memory_cost)
    }

    /// Available gas at this moment.
    pub fn available_gas(&self) -> Gas {
        self.context.gas_limit - self.total_used_gas()
    }

    /// Total used gas including the memory gas.
    pub fn total_used_gas(&self) -> Gas {
        match self.used_gas {
            GasUsage::All => self.context.gas_limit,
            GasUsage::Some(gas) => self.memory_gas() + gas,
        }
    }
}

/// A VM runtime. Only available in eval.
pub struct Runtime {
    /// The current blockhash commitment states.
    pub blockhash_state: BlockhashState,
    /// Block header.
    pub block: HeaderParams,

    /// Hooks for context history.
    pub context_history_hooks: Vec<Box<Fn(&Context)>>,
}

impl Runtime {
    /// Create a new VM runtime.
    pub fn new(block: HeaderParams) -> Self {
        Self::with_states(block, BlockhashState::default())
    }

    /// Create the runtime with the given blockhash state.
    pub fn with_states(block: HeaderParams, blockhash_state: BlockhashState) -> Self {
        Runtime {
            block, blockhash_state,

            context_history_hooks: Vec::new(),
        }
    }
}

/// A VM state with PC.
pub struct Machine<M, P: Patch> {
    state: State<M, P>,
    status: MachineStatus,
}

#[derive(Debug, Clone)]
/// Represents the current runtime status.
// TODO: consider boxing the large fields to reduce the total size of the enum
pub enum MachineStatus {
    /// This runtime is actively running or has just been started.
    Running,
    /// This runtime has exited successfully. Calling `step` on this
    /// runtime again would panic.
    ExitedOk,
    /// This runtime has exited with errors. Calling `step` on this
    /// runtime again would panic.
    ExitedErr(OnChainError),
    /// This runtime has exited because it does not support certain
    /// operations. Unlike `ExitedErr`, this is not on-chain, and if
    /// it happens, client should either drop the transaction or panic
    /// (because it rarely happens).
    ExitedNotSupported(NotSupportedError),
    /// This runtime requires execution of a sub runtime, which is a
    /// ContractCreation instruction.
    InvokeCreate(Context),
    /// This runtime requires execution of a sub runtime, which is a
    /// MessageCall instruction.
    InvokeCall(Context, (U256, U256)),
}

#[derive(Debug, Clone)]
/// Used for `check` for additional checks related to the runtime.
pub enum ControlCheck {
    Jump(M256),
}

#[derive(Debug, Clone)]
/// Used for `step` for additional operations related to the runtime.
// TODO: consider boxing the large fields to reduce the total size of the enum
pub enum Control {
    Stop,
    Revert,
    Jump(M256),
    InvokeCreate(Context),
    InvokeCall(Context, (U256, U256)),
}

impl<M: Memory + Default, P: Patch> Machine<M, P> {
    /// Create a new runtime.
    pub fn new(context: Context, depth: usize) -> Self {
        Self::with_states(context, depth,
                          AccountState::default())
    }

    /// Create a new runtime with the given states.
    pub fn with_states(context: Context,
                       depth: usize, account_state: AccountState<P::Account>) -> Self {
        Machine {
            status: MachineStatus::Running,
            state: State {
                memory: M::default(),
                stack: Stack::default(),

                out: Rc::new(Vec::new()),
                ret: Rc::new(Vec::new()),

                memory_cost: Gas::zero(),
                used_gas: GasUsage::Some(Gas::zero()),
                refunded_gas: Gas::zero(),

                account_state,
                logs: Vec::new(),
                removed: Vec::new(),

                depth,
                position: 0,
                valids: Valids::new(context.code.as_slice()),

                context,
            },
        }
    }

    /// Derive this runtime to create a sub runtime. This will not
    /// modify the current runtime, and it will have a chance to
    /// review whether it wants to accept the result of this sub
    /// runtime afterwards.
    pub fn derive(&self, context: Context) -> Self {
        Machine {
            status: MachineStatus::Running,
            state: State {
                memory: M::default(),
                stack: Stack::default(),

                out: Rc::new(Vec::new()),
                ret: Rc::new(Vec::new()),

                memory_cost: Gas::zero(),
                used_gas: GasUsage::Some(Gas::zero()),
                refunded_gas: Gas::zero(),

                account_state: self.state.account_state.clone(),
                logs: Vec::new(),
                removed: self.state.removed.clone(),

                depth: self.state.depth + 1,

                position: 0,
                valids: Valids::new(context.code.as_slice()),

                context,
            },
        }
    }

    /// Commit a new account into this runtime.
    pub fn commit_account(&mut self, commitment: AccountCommitment) -> Result<(), CommitError> {
        self.state.account_state.commit(commitment)
    }

    /// Step a precompiled runtime. This function returns true if the
    /// runtime is indeed a precompiled address. Otherwise return
    /// false with state unchanged.
    pub fn step_precompiled(&mut self) -> bool {
        for precompiled in P::precompileds() {
            if self.state.context.address == precompiled.0 &&
                (precompiled.1.is_none() || precompiled.1.unwrap() == self.state.context.code.as_slice())
            {
                let data = &self.state.context.data;
                match precompiled.2.gas_and_step(data, self.state.context.gas_limit) {
                    Err(RuntimeError::OnChain(err)) => {
                        reset_error_hard!(self, err);
                    },
                    Err(RuntimeError::NotSupported(err)) => {
                        reset_error_not_supported!(self, err);
                    },
                    Ok((gas, ret)) => {
                        assert!(gas <= self.state.context.gas_limit);
                        self.state.used_gas = GasUsage::Some(gas);
                        self.state.out = ret;
                        self.status = MachineStatus::ExitedOk;
                    }
                }
                return true;
            }
        }
        false
    }

    /// Peek the next instruction.
    pub fn peek(&self) -> Option<Instruction> {
        let pc = PC::<P>::new(&self.state.context.code,
                              &self.state.valids, &self.state.position);
        match pc.peek() {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }

    /// Peek the next opcode.
    pub fn peek_opcode(&self) -> Option<Opcode> {
        let pc = PC::<P>::new(&self.state.context.code,
                              &self.state.valids, &self.state.position);
        match pc.peek_opcode() {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }

    /// Step an instruction in the PC. The eval result is refected by
    /// the runtime status, and it will only return an error if
    /// there're accounts or blockhashes to be committed to this
    /// runtime for it to run. In that case, the state of the current
    /// runtime will not be affected.
    pub fn step(&mut self, runtime: &Runtime) -> Result<(), RequireError> {
        debug!("VM step started");
        debug!("Code: {:x?}", &self.state.context.code[self.state.position..]);
        debug!("Stack: {:#x?}", self.state.stack);

        struct Precheck {
            position: usize,
            memory_cost: Gas,
            gas_cost: Gas,
            gas_stipend: Gas,
            gas_refund: Gas,
            after_gas: Gas,
        }

        match &self.status {
            MachineStatus::Running => (),
            _ => panic!(),
        }

        if self.step_precompiled() {
            trace!("precompiled step succeeded");
            return Ok(());
        }

        let Precheck {
            position, memory_cost,
            gas_cost, gas_stipend, gas_refund, after_gas
        } = {
            let pc = PC::<P>::new(&self.state.context.code,
                                  &self.state.valids, &self.state.position);

            if pc.is_end() {
                debug!("reached code EOF");
                self.status = MachineStatus::ExitedOk;
                return Ok(());
            }

            let instruction = match pc.peek() {
                Ok(val) => val,
                Err(err) => {
                    reset_error_hard!(self, err);
                    return Ok(())
                },
            };

            match check_opcode(instruction, &self.state, runtime).and_then(|v| {
                match v {
                    None => Ok(()),
                    Some(ControlCheck::Jump(dest)) => {
                        if dest <= M256::from(usize::max_value()) && pc.is_valid(dest.as_usize()) {
                            Ok(())
                        } else {
                            Err(OnChainError::BadJumpDest.into())
                        }
                    }
                }
            }) {
                Ok(()) => (),
                Err(EvalOnChainError::OnChain(error)) => {
                    reset_error_hard!(self, error);
                    return Ok(());
                },
                Err(EvalOnChainError::Require(error)) => {
                    return Err(error);
                },
            }

            if self.state.context.is_static {
                match check_static(instruction, &self.state, runtime) {
                    Ok(()) => (),
                    Err(EvalOnChainError::OnChain(error)) => {
                        reset_error_hard!(self, error);
                        return Ok(());
                    },
                    Err(EvalOnChainError::Require(error)) => {
                        return Err(error);
                    },
                }
            }

            let used_gas = match self.state.used_gas {
                GasUsage::Some(gas) => gas,
                GasUsage::All => {
                    reset_error_hard!(self, OnChainError::EmptyGas);
                    return Ok(());
                },
            };

            let position = pc.position();
            let memory_cost = memory_cost(instruction, &self.state);
            let memory_gas = memory_gas(memory_cost);
            let gas_cost = gas_cost::<M, P>(instruction, &self.state);
            let gas_stipend = gas_stipend(instruction, &self.state);
            let gas_refund = gas_refund(instruction, &self.state);

            let all_gas_cost = memory_gas + used_gas + gas_cost;
            if self.state.context.gas_limit < all_gas_cost {
                reset_error_hard!(self, OnChainError::EmptyGas);
                return Ok(());
            }

            match check_support(instruction, &self.state) {
                Ok(()) => (),
                Err(err) => {
                    reset_error_not_supported!(self, err);
                    return Ok(());
                },
            };

            let after_gas = self.state.context.gas_limit - all_gas_cost;

            match extra_check_opcode::<M, P>(instruction, &self.state, gas_stipend, after_gas) {
                Ok(()) => (),
                Err(err) => {
                    reset_error_hard!(self, err);
                    return Ok(());
                },
            }

            Precheck {
                position, memory_cost,
                gas_cost, gas_stipend, gas_refund, after_gas
            }
        };

        trace!("position:    {}", position);
        trace!("memory_cost: {:x?}", memory_cost);
        trace!("gas_cost:    {:x?}", gas_cost);
        trace!("gas_stipend: {:x?}", gas_stipend);
        trace!("gas_refund:  {:x?}", gas_refund);
        trace!("after_gas:   {:x?}", after_gas);

        let instruction = PCMut::<P>::new(&self.state.context.code,
                                          &self.state.valids, &mut self.state.position)
            .read().unwrap();

        let result = run_opcode::<M, P>((instruction, position),
                                        &mut self.state, runtime, gas_stipend, after_gas);

        self.state.used_gas += gas_cost - gas_stipend;
        self.state.memory_cost = memory_cost;
        self.state.refunded_gas = self.state.refunded_gas + gas_refund;

        debug!("{:?} => {:?}", instruction, result);
        debug!("gas used: {:x?}", self.state.total_used_gas());
        debug!("gas left: {:x?}", self.state.available_gas());

        match result {
            None => Ok(()),
            Some(Control::Jump(dest)) => {
                PCMut::<P>::new(&self.state.context.code,
                                &self.state.valids, &mut self.state.position)
                    .jump(dest.as_usize()).unwrap();
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
            Some(Control::Revert) => {
                reset_error_revert!(self);
                Ok(())
            },
        }
    }

    /// Get the runtime state.
    pub fn state(&self) -> &State<M, P> {
        &self.state
    }

    /// Get the runtime PC.
    pub fn pc(&self) -> PC<P> {
        PC::new(&self.state.context.code, &self.state.valids, &self.state.position)
    }

    /// Get the current runtime status.
    pub fn status(&self) -> MachineStatus {
        self.status.clone()
    }
}
