use utils::gas::Gas;
use utils::address::Address;
use utils::bigint::{M256, U256};

use std::cmp::max;
use vm::{Memory, Storage, Instruction};
use super::State;

const G_ZERO: usize = 0;
const G_BASE: usize = 2;
const G_VERYLOW: usize = 3;
const G_LOW: usize = 5;
const G_MID: usize = 8;
const G_HIGH: usize = 10;
const G_EXTCODE_DEFAULT: usize = 20;
const G_EXTCODE_EIP150: usize = 700;
const G_BALANCE_DEFAULT: usize = 20;
const G_BALANCE_EIP150: usize = 400;
const G_SLOAD_DEFAULT: usize = 50;
const G_SLOAD_EIP150: usize = 200;
const G_JUMPDEST: usize = 1;
const G_SSET: usize = 20000;
const G_SRESET: usize = 5000;
const R_SCLEAR: usize = 15000;
const R_SUICIDE: usize = 24000;
const G_SUICIDE_DEFAULT: usize = 0;
const G_SUICIDE_EIP150: usize = 5000;
const G_CREATE: usize = 32000;
const G_CODEDEPOSITE: usize = 200;
const G_CALL_DEFAULT: usize = 40;
const G_CALL_EIP150: usize = 700;
const G_CALLVALUE: usize = 9000;
const G_CALLSTIPEND: usize = 2300;
const G_NEWACCOUNT: usize = 25000;
const G_EXP: usize = 10;
const G_EXPBYTE_DEFAULT: usize = 10;
const G_EXPBYTE_EIP160: usize = 50;
const G_MEMORY: usize = 3;
const G_TXCREATE: usize = 32000;
const G_TXDATAZERO: usize = 4;
const G_TXDATANONZERO: usize = 68;
const G_TRANSACTION: usize = 21000;
const G_LOG: usize = 375;
const G_LOGDATA: usize = 8;
const G_LOGTOPIC: usize = 375;
const G_SHA3: usize = 30;
const G_SHA3WORD: usize = 6;
const G_COPY: usize = 3;
const G_BLOCKHASH: usize = 20;

fn sstore_cost<M: Memory + Default,
               S: Storage + Default + Clone>(machine: &State<M, S>) -> Gas {
    let index = machine.stack.peek(0).unwrap();
    let value = machine.stack.peek(1).unwrap();
    let address = machine.context.address;

    if value != M256::zero() && machine.account_state.storage(address).unwrap().read(index) == M256::zero() {
        G_SSET.into()
    } else {
        G_SRESET.into()
    }
}

fn call_cost<M: Memory + Default,
             S: Storage + Default + Clone>(machine: &State<M, S>) -> Gas {
    gascap_cost(machine) + extra_cost(machine)
}

#[allow(unused_variables)]
fn gascap_cost<M: Memory + Default,
               S: Storage + Default>(machine: &State<M, S>) -> Gas {
    // let base2 = machine.stack().peek(0).unwrap().into();

    // if available_gas >= extra_cost(machine).unwrap() {
    //     let base1 = available_gas - extra_cost(machine).unwrap();
    //     Ok(min(base1 - base1 / Gas::from(64u64), base2))
    // } else {
    //     Ok(base2)
    // }
    Gas::zero()
}

fn extra_cost<M: Memory + Default,
              S: Storage + Default + Clone>(machine: &State<M, S>) -> Gas {
    Gas::from(if machine.patch.eip150() { G_CALL_EIP150 } else { G_CALL_DEFAULT }) + xfer_cost(machine) + new_cost(machine)
}

fn xfer_cost<M: Memory + Default,
             S: Storage + Default>(machine: &State<M, S>) -> Gas {
    let val = machine.stack.peek(2).unwrap();
    if val != M256::zero() {
        G_CALLVALUE.into()
    } else {
        Gas::zero()
    }
}

fn new_cost<M: Memory + Default,
            S: Storage + Default + Clone>(machine: &State<M, S>) -> Gas {
    let address: Address = machine.stack.peek(1).unwrap().into();
    if machine.account_state.balance(address).unwrap() == U256::zero() && machine.account_state.nonce(address).unwrap() == M256::zero() && machine.account_state.code(address).unwrap().len() == 0 {
        G_NEWACCOUNT.into()
    } else {
        Gas::zero()
    }
}

fn suicide_cost<M: Memory + Default,
                S: Storage + Default>(machine: &State<M, S>) -> Gas {
    let address: Address = machine.stack.peek(0).unwrap().into();
    Gas::from(if machine.patch.eip150() { G_SUICIDE_EIP150 } else { G_SUICIDE_DEFAULT }) + if address == Address::default() {
        Gas::from(G_NEWACCOUNT)
    } else {
        Gas::zero()
    }
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

pub fn memory_gas(a: Gas) -> Gas {
    (Gas::from(G_MEMORY) * a + a * a / Gas::from(512u64)).into()
}

pub fn memory_cost<M: Memory + Default, S: Storage + Default + Clone>(instruction: Instruction, state: &State<M, S>) -> Gas {
    let ref stack = state.stack;

    let current = state.memory_cost;
    let next = match instruction {
        Instruction::SHA3 | Instruction::RETURN | Instruction::LOG(_) => {
            let from: U256 = stack.peek(0).unwrap().into();
            let len: U256 = stack.peek(1).unwrap().into();
            memory_expand(current, Gas::from(from), Gas::from(len))
        },
        Instruction::CODECOPY | Instruction::CALLDATACOPY => {
            let from: U256 = stack.peek(0).unwrap().into();
            let len: U256 = stack.peek(2).unwrap().into();
            memory_expand(current, Gas::from(from), Gas::from(len))
        },
        Instruction::EXTCODECOPY => {
            let from: U256 = stack.peek(1).unwrap().into();
            let len: U256 = stack.peek(3).unwrap().into();
            memory_expand(current, Gas::from(from), Gas::from(len))
        },
        Instruction::MLOAD | Instruction::MSTORE => {
            let from: U256 = stack.peek(0).unwrap().into();
            memory_expand(current, Gas::from(from), Gas::from(32u64))
        },
        Instruction::MSTORE8 => {
            let from: U256 = stack.peek(0).unwrap().into();
            memory_expand(current, Gas::from(from), Gas::from(1u64))
        },
        Instruction::CREATE => {
            let from: U256 = stack.peek(1).unwrap().into();
            let len: U256 = stack.peek(2).unwrap().into();
            memory_expand(current, Gas::from(from), Gas::from(len))
        },
        Instruction::CALL => {
            let in_from: U256 = stack.peek(3).unwrap().into();
            let in_len: U256 = stack.peek(4).unwrap().into();
            let out_from: U256 = stack.peek(5).unwrap().into();
            let out_len: U256 = stack.peek(6).unwrap().into();
            memory_expand(memory_expand(current, Gas::from(in_from), Gas::from(in_len)),
                          Gas::from(out_from), Gas::from(out_len))
        },
        _ => {
            current
        }
    };
    next
}

pub fn gas_cost<M: Memory + Default, S: Storage + Default + Clone>(instruction: Instruction, state: &State<M, S>) -> Gas {
    match instruction {
        Instruction::CALL | Instruction::CALLCODE |
        Instruction::DELEGATECALL => call_cost(state),
        Instruction::SUICIDE => suicide_cost(state),
        Instruction::SSTORE => sstore_cost(state),

        Instruction::SHA3 => {
            let len = state.stack.peek(1).unwrap();
            let wordd = Gas::from(len) / Gas::from(32u64);
            let wordr = Gas::from(len) % Gas::from(32u64);
            (Gas::from(G_SHA3) + Gas::from(G_SHA3WORD) * if wordr == Gas::zero() { wordd } else { wordd + Gas::from(1u64) }).into()
        },

        Instruction::LOG(v) => {
            let len = state.stack.peek(1).unwrap();
            (Gas::from(G_LOG) + Gas::from(G_LOGDATA) * Gas::from(len) + Gas::from(G_LOGTOPIC) * Gas::from(v)).into()
        },

        Instruction::EXTCODECOPY => {
            let len = state.stack.peek(3).unwrap();
            let wordd = Gas::from(len) / Gas::from(32u64);
            let wordr = Gas::from(len) % Gas::from(32u64);
            (Gas::from(if state.patch.eip150() { G_EXTCODE_EIP150 } else { G_EXTCODE_DEFAULT }) + Gas::from(G_COPY) * if wordr == Gas::zero() { wordd } else { wordd + Gas::from(1u64) }).into()
        },

        Instruction::CALLDATACOPY | Instruction::CODECOPY => {
            let len = state.stack.peek(2).unwrap();
            let wordd = Gas::from(len) / Gas::from(32u64);
            let wordr = Gas::from(len) % Gas::from(32u64);
            (Gas::from(G_VERYLOW) + Gas::from(G_COPY) * if wordr == Gas::zero() { wordd } else { wordd + Gas::from(1u64) }).into()
        },

        Instruction::EXP => {
            if state.stack.peek(1).unwrap() == M256::zero() {
                Gas::from(G_EXP)
            } else {
                Gas::from(G_EXP) + Gas::from(if state.patch.eip160() { G_EXPBYTE_EIP160 } else { G_EXPBYTE_DEFAULT }) * (Gas::from(1u64) + Gas::from(state.stack.peek(1).unwrap().log2floor()) / Gas::from(8u64))
            }
        }

        Instruction::CREATE => G_CREATE.into(),
        Instruction::JUMPDEST => G_JUMPDEST.into(),
        Instruction::SLOAD => (if state.patch.eip150() { G_SLOAD_EIP150 } else { G_SLOAD_DEFAULT }).into(),

        // W_zero
        Instruction::STOP | Instruction::RETURN
            => G_ZERO.into(),

        // W_base
        Instruction::ADDRESS | Instruction::ORIGIN | Instruction::CALLER |
        Instruction::CALLVALUE | Instruction::CALLDATASIZE |
        Instruction::CODESIZE | Instruction::GASPRICE | Instruction::COINBASE |
        Instruction::TIMESTAMP | Instruction::NUMBER | Instruction::DIFFICULTY |
        Instruction::GASLIMIT | Instruction::POP | Instruction::PC |
        Instruction::MSIZE | Instruction::GAS
            => G_BASE.into(),

        // W_verylow
        Instruction::ADD | Instruction::SUB | Instruction::NOT | Instruction::LT |
        Instruction::GT | Instruction::SLT | Instruction::SGT | Instruction::EQ |
        Instruction::ISZERO | Instruction::AND | Instruction::OR | Instruction::XOR |
        Instruction::BYTE | Instruction::CALLDATALOAD | Instruction::MLOAD |
        Instruction::MSTORE | Instruction::MSTORE8 | Instruction::PUSH(_) |
        Instruction::DUP(_) | Instruction::SWAP(_)
            => G_VERYLOW.into(),

        // W_low
        Instruction::MUL | Instruction::DIV | Instruction::SDIV | Instruction::MOD |
        Instruction::SMOD | Instruction::SIGNEXTEND
            => G_LOW.into(),

        // W_mid
        Instruction::ADDMOD | Instruction::MULMOD | Instruction::JUMP
            => G_MID.into(),

        // W_high
        Instruction::JUMPI => G_HIGH.into(),

        // W_extcode
        Instruction::EXTCODESIZE => (if state.patch.eip150() { G_EXTCODE_EIP150 } else { G_EXTCODE_DEFAULT }).into(),

        Instruction::BALANCE => (if state.patch.eip150() { G_BALANCE_EIP150 } else { G_BALANCE_DEFAULT }).into(),
        Instruction::BLOCKHASH => G_BLOCKHASH.into(),
    }
}

pub fn gas_stipend<M: Memory + Default, S: Storage + Default + Clone>(instruction: Instruction, state: &State<M, S>) -> Gas {
    match instruction {
        Instruction::CALL => {
            let value = state.stack.peek(2).unwrap();

            if value != M256::zero() {
                G_CALLSTIPEND.into()
            } else {
                Gas::zero()
            }
        },
        _ => Gas::zero(),
    }
}

pub fn gas_refund<M: Memory + Default, S: Storage + Default + Clone>(instruction: Instruction, state: &State<M, S>) -> Gas {
    match instruction {
        Instruction::SSTORE => {
            let index = state.stack.peek(0).unwrap();
            let value = state.stack.peek(1).unwrap();
            let address = state.context.address;

            if value == M256::zero() && state.account_state.storage(address).unwrap().read(index) != M256::zero() {
                Gas::from(R_SCLEAR)
            } else {
                Gas::zero()
            }
        },
        Instruction::SUICIDE => {
            // TODO: check whether I_a belongs to A_s
            Gas::zero()
        },
        _ => Gas::zero()
    }
}
