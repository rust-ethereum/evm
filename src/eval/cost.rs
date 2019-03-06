//! Cost calculation logic

use bigint::{Address, Gas, M256, U256};

#[cfg(not(feature = "std"))]
use core::cmp::max;
#[cfg(feature = "std")]
use std::cmp::max;

use super::State;
use crate::{AccountPatch, Instruction, Memory, Patch};

const G_ZERO: usize = 0;
const G_BASE: usize = 2;
const G_VERYLOW: usize = 3;
const G_LOW: usize = 5;
const G_MID: usize = 8;
const G_HIGH: usize = 10;
const G_JUMPDEST: usize = 1;
const G_SNOOP: usize = 200;
const G_SSET: usize = 20000;
const G_SRESET: usize = 5000;
const R_SRESET: isize = 15000;
const R_SCLEAR: isize = 19800;
const R_SNOOP: isize = 4800;
const R_SUICIDE: isize = 24000;
const G_CREATE: usize = 32000;
const G_CODEDEPOSIT: usize = 200;
const G_CALLVALUE: usize = 9000;
const G_CALLSTIPEND: usize = 2300;
const G_NEWACCOUNT: usize = 25000;
const G_EXP: usize = 10;
const G_MEMORY: usize = 3;
const G_LOG: usize = 375;
const G_LOGDATA: usize = 8;
const G_LOGTOPIC: usize = 375;
const G_SHA3: usize = 30;
const G_SHA3WORD: usize = 6;
const G_COPY: usize = 3;
const G_BLOCKHASH: usize = 20;
const G_EXTCODEHASH: usize = 400;

fn sstore_cost<M: Memory, P: Patch>(state: &State<M, P>) -> Gas {
    let index: U256 = state.stack.peek(0).unwrap().into();
    let value = state.stack.peek(1).unwrap();
    let address = state.context.address;

    if state.patch.has_reduced_sstore_gas_metering() {
        trace!("using EIP1283 reduced SSTORE gas metering scheme");
        // If RequireError is thrown here, that means that original storage was unset, hence defaulting to Zero.
        let orig = state.account_state.storage_read_orig(address, index).unwrap_or(M256::zero());
        let current = state.account_state.storage_read(address, index).unwrap();
        if value == current {
            G_SNOOP.into()
        } else {
            if orig == current {
                if orig == M256::zero() {
                    G_SSET.into()
                } else {
                    G_SRESET.into()
                }
            } else {
                G_SNOOP.into()
            }
        }
    } else if value != M256::zero() && state.account_state.storage_read(address, index).unwrap() == M256::zero() {
        G_SSET.into()
    } else {
        G_SRESET.into()
    }
}

fn call_cost<M: Memory, P: Patch>(machine: &State<M, P>, instruction: &Instruction) -> Gas {
    machine.patch.gas_call() + xfer_cost(machine, instruction) + new_cost(machine, instruction)
}

fn xfer_cost<M: Memory, P: Patch>(machine: &State<M, P>, instruction: &Instruction) -> Gas {
    if instruction == &Instruction::CALL || instruction == &Instruction::CALLCODE {
        let val = machine.stack.peek(2).unwrap();
        if val != M256::zero() {
            G_CALLVALUE.into()
        } else {
            Gas::zero()
        }
    } else {
        Gas::zero()
    }
}

fn new_cost<M: Memory, P: Patch>(machine: &State<M, P>, instruction: &Instruction) -> Gas {
    let address: Address = machine.stack.peek(1).unwrap().into();
    if (instruction == &Instruction::CALL || instruction == &Instruction::STATICCALL)
        && !machine.account_state.exists(address).unwrap()
    {
        Gas::from(G_NEWACCOUNT)
    } else {
        Gas::zero()
    }
}

fn suicide_cost<M: Memory, P: Patch>(machine: &State<M, P>) -> Gas {
    let address: Address = machine.stack.peek(0).unwrap().into();
    let current_balance = machine.account_state.balance(machine.context.address).unwrap();
    let is_existing = machine.account_state.exists(address).unwrap();

    let is_eip161_relaxed_zero_balance_transfer =
        current_balance != U256::zero() && !machine.patch.account_patch().empty_considered_exists();

    debug!("suicide in favor of {}, exists: {}", address, is_existing);
    debug!("suicide transfer value: {}", current_balance);

    let suicide_gas_topup = if !is_existing && !is_eip161_relaxed_zero_balance_transfer {
        trace!("suicide with new account gas topup");
        machine.patch.gas_suicide_new_account()
    } else {
        trace!("suicide with zero gas topup");
        Gas::zero()
    };
    machine.patch.gas_suicide() + suicide_gas_topup
}

fn memory_expand(current: Gas, from: Gas, len: Gas) -> Gas {
    if len == Gas::zero() {
        return current;
    }

    let rem = (from + len) % Gas::from(32u64);
    let new = if rem == Gas::zero() {
        (from + len) / Gas::from(32u64)
    } else {
        (from + len) / Gas::from(32u64) + Gas::from(1u64)
    };
    max(current, new)
}

/// Calculate code deposit cost for a ContractCreation transaction.
pub fn code_deposit_gas(len: usize) -> Gas {
    Gas::from(G_CODEDEPOSIT) * Gas::from(len)
}

/// Calculate the memory gas from the memory cost.
pub fn memory_gas(a: Gas) -> Gas {
    Gas::from(G_MEMORY) * a + a * a / Gas::from(512u64)
}

/// Calculate the memory cost. This is the same as the active memory
/// length in the Yellow Paper.
pub fn memory_cost<M: Memory, P: Patch>(instruction: Instruction, state: &State<M, P>) -> Gas {
    let stack = &state.stack;

    let current = state.memory_cost;
    match instruction {
        Instruction::SHA3 | Instruction::RETURN | Instruction::REVERT | Instruction::LOG(_) => {
            let from: U256 = stack.peek(0).unwrap().into();
            let len: U256 = stack.peek(1).unwrap().into();
            memory_expand(current, Gas::from(from), Gas::from(len))
        }
        Instruction::CODECOPY | Instruction::CALLDATACOPY | Instruction::RETURNDATACOPY => {
            let from: U256 = stack.peek(0).unwrap().into();
            let len: U256 = stack.peek(2).unwrap().into();
            memory_expand(current, Gas::from(from), Gas::from(len))
        }
        Instruction::EXTCODECOPY => {
            let from: U256 = stack.peek(1).unwrap().into();
            let len: U256 = stack.peek(3).unwrap().into();
            memory_expand(current, Gas::from(from), Gas::from(len))
        }
        Instruction::MLOAD | Instruction::MSTORE => {
            let from: U256 = stack.peek(0).unwrap().into();
            memory_expand(current, Gas::from(from), Gas::from(32u64))
        }
        Instruction::MSTORE8 => {
            let from: U256 = stack.peek(0).unwrap().into();
            memory_expand(current, Gas::from(from), Gas::from(1u64))
        }
        Instruction::CREATE | Instruction::CREATE2 => {
            let from: U256 = stack.peek(1).unwrap().into();
            let len: U256 = stack.peek(2).unwrap().into();
            memory_expand(current, Gas::from(from), Gas::from(len))
        }
        Instruction::CALL | Instruction::CALLCODE => {
            let in_from: U256 = stack.peek(3).unwrap().into();
            let in_len: U256 = stack.peek(4).unwrap().into();
            let out_from: U256 = stack.peek(5).unwrap().into();
            let out_len: U256 = stack.peek(6).unwrap().into();
            memory_expand(
                memory_expand(current, Gas::from(in_from), Gas::from(in_len)),
                Gas::from(out_from),
                Gas::from(out_len),
            )
        }
        _ => current,
    }
}

/// Calculate the gas cost.
pub fn gas_cost<M: Memory, P: Patch>(instruction: Instruction, state: &State<M, P>) -> Gas {
    match instruction {
        Instruction::CALL => call_cost::<M, P>(state, &Instruction::CALL),
        Instruction::CALLCODE => call_cost::<M, P>(state, &Instruction::CALLCODE),
        Instruction::DELEGATECALL => call_cost::<M, P>(state, &Instruction::DELEGATECALL),
        Instruction::STATICCALL => call_cost::<M, P>(state, &Instruction::STATICCALL),
        Instruction::SUICIDE => suicide_cost::<M, P>(state),
        Instruction::SSTORE => sstore_cost(state),

        Instruction::SHA3 => {
            let len = state.stack.peek(1).unwrap();
            let wordd = Gas::from(len) / Gas::from(32u64);
            let wordr = Gas::from(len) % Gas::from(32u64);
            Gas::from(G_SHA3)
                + Gas::from(G_SHA3WORD)
                    * if wordr == Gas::zero() {
                        wordd
                    } else {
                        wordd + Gas::from(1u64)
                    }
        }

        Instruction::LOG(v) => {
            let len = state.stack.peek(1).unwrap();
            Gas::from(G_LOG) + Gas::from(G_LOGDATA) * Gas::from(len) + Gas::from(G_LOGTOPIC) * Gas::from(v)
        }

        Instruction::EXTCODECOPY => {
            let len = state.stack.peek(3).unwrap();
            let wordd = Gas::from(len) / Gas::from(32u64);
            let wordr = Gas::from(len) % Gas::from(32u64);
            state.patch.gas_extcode()
                + Gas::from(G_COPY)
                    * if wordr == Gas::zero() {
                        wordd
                    } else {
                        wordd + Gas::from(1u64)
                    }
        }

        Instruction::CALLDATACOPY | Instruction::CODECOPY | Instruction::RETURNDATACOPY => {
            let len = state.stack.peek(2).unwrap();
            let wordd = Gas::from(len) / Gas::from(32u64);
            let wordr = Gas::from(len) % Gas::from(32u64);
            Gas::from(G_VERYLOW)
                + Gas::from(G_COPY)
                    * if wordr == Gas::zero() {
                        wordd
                    } else {
                        wordd + Gas::from(1u64)
                    }
        }

        Instruction::EXP => {
            if state.stack.peek(1).unwrap() == M256::zero() {
                Gas::from(G_EXP)
            } else {
                Gas::from(G_EXP)
                    + state.patch.gas_expbyte()
                        * (Gas::from(1u64) + Gas::from(state.stack.peek(1).unwrap().log2floor()) / Gas::from(8u64))
            }
        }

        Instruction::CREATE => G_CREATE.into(),
        Instruction::CREATE2 => {
            let base = G_CREATE;
            let init_code_len = state.stack.peek(2).unwrap().as_u64();
            let sha_addup = G_SHA3WORD * (init_code_len as f32 / 32.0).ceil() as usize;
            (base + sha_addup).into()
        }
        Instruction::JUMPDEST => G_JUMPDEST.into(),
        Instruction::SLOAD => state.patch.gas_sload(),

        // W_zero
        Instruction::STOP | Instruction::RETURN | Instruction::REVERT => G_ZERO.into(),

        // W_base
        Instruction::ADDRESS
        | Instruction::ORIGIN
        | Instruction::CALLER
        | Instruction::CALLVALUE
        | Instruction::CALLDATASIZE
        | Instruction::RETURNDATASIZE
        | Instruction::CODESIZE
        | Instruction::GASPRICE
        | Instruction::COINBASE
        | Instruction::TIMESTAMP
        | Instruction::NUMBER
        | Instruction::DIFFICULTY
        | Instruction::GASLIMIT
        | Instruction::POP
        | Instruction::PC
        | Instruction::MSIZE
        | Instruction::GAS => G_BASE.into(),

        // W_verylow
        Instruction::ADD
        | Instruction::SUB
        | Instruction::NOT
        | Instruction::LT
        | Instruction::GT
        | Instruction::SLT
        | Instruction::SGT
        | Instruction::EQ
        | Instruction::ISZERO
        | Instruction::AND
        | Instruction::OR
        | Instruction::XOR
        | Instruction::BYTE
        | Instruction::CALLDATALOAD
        | Instruction::MLOAD
        | Instruction::MSTORE
        | Instruction::MSTORE8
        | Instruction::PUSH(_)
        | Instruction::DUP(_)
        | Instruction::SWAP(_)
        | Instruction::SHL
        | Instruction::SHR
        | Instruction::SAR => G_VERYLOW.into(),

        // W_low
        Instruction::MUL
        | Instruction::DIV
        | Instruction::SDIV
        | Instruction::MOD
        | Instruction::SMOD
        | Instruction::SIGNEXTEND => G_LOW.into(),

        // W_mid
        Instruction::ADDMOD | Instruction::MULMOD | Instruction::JUMP => G_MID.into(),

        // W_high
        Instruction::JUMPI => G_HIGH.into(),

        // W_extcode
        Instruction::EXTCODESIZE => state.patch.gas_extcode(),
        Instruction::BALANCE => state.patch.gas_balance(),
        Instruction::BLOCKHASH => G_BLOCKHASH.into(),
        Instruction::EXTCODEHASH => G_EXTCODEHASH.into(),
    }
}

/// Raise gas stipend for CALL and CALLCODE instruction.
pub fn gas_stipend<M: Memory, P: Patch>(instruction: Instruction, state: &State<M, P>) -> Gas {
    match instruction {
        Instruction::CALL | Instruction::CALLCODE => {
            let value = state.stack.peek(2).unwrap();

            if value != M256::zero() {
                G_CALLSTIPEND.into()
            } else {
                Gas::zero()
            }
        }
        _ => Gas::zero(),
    }
}

/// Calculate the refunded gas.
pub fn gas_refund<M: Memory, P: Patch>(instruction: Instruction, state: &State<M, P>) -> isize {
    match instruction {
        Instruction::SSTORE => {
            let index: U256 = state.stack.peek(0).unwrap().into();
            let value = state.stack.peek(1).unwrap();
            let address = state.context.address;

            if state.patch.has_reduced_sstore_gas_metering() {
                trace!("using EIP1283 reduced SSTORE gas metering scheme");
                // If RequireError is thrown here, that means that original storage was unset, hence defaulting to Zero.
                let orig = state.account_state.storage_read_orig(address, index).unwrap_or(M256::zero());
                let current = state.account_state.storage_read(address, index).unwrap();
                let mut refund = 0;

                if value != current {
                    if orig == current {
                        if orig != M256::zero() {
                            refund += R_SRESET;
                        }
                    } else {
                        if orig != M256::zero() {
                            if current == M256::zero() {
                                refund -= R_SRESET;
                            }
                            if value == M256::zero() {
                                refund += R_SRESET;
                            }
                        }
                        if orig == value {
                            if orig == M256::zero() {
                                refund += R_SCLEAR;
                            } else {
                                refund += R_SNOOP;
                            }
                        }
                    }
                }

                refund
            } else if value == M256::zero() && state.account_state.storage_read(address, index).unwrap() != M256::zero()
            {
                R_SRESET
            } else {
                0
            }
        }
        Instruction::SUICIDE => {
            if state.removed.contains(&state.context.address) {
                0
            } else {
                R_SUICIDE
            }
        }
        _ => 0,
    }
}

pub trait AddRefund {
    fn add_refund(self, refund: isize) -> Self;
}

impl AddRefund for Gas {
    fn add_refund(self, refund: isize) -> Self {
        let (refund, sign) = if refund < 0 {
            (0 - refund, true)
        } else {
            (refund, false)
        };

        if sign {
            self - Gas::from(refund as usize)
        } else {
            self + Gas::from(refund as usize)
        }
    }
}
