use utils::bigint::{M256, U256, U512};
use utils::gas::Gas;
use utils::address::Address;
use super::Opcode;
use vm::{Machine, Memory, Stack, PC, Error, Result};
use transaction::Transaction;
use blockchain::Block;
use std::cmp::{min, max};

const G_ZERO: usize = 0;
const G_BASE: usize = 2;
const G_VERYLOW: usize = 3;
const G_LOW: usize = 5;
const G_MID: usize = 8;
const G_HIGH: usize = 10;
const G_EXTCODE: usize = 700;
const G_BALANCE: usize = 400;
const G_SLOAD: usize = 200;
const G_JUMPDEST: usize = 1;
const G_SSET: usize = 20000;
const G_SRESET: usize = 5000;
const R_SCLEAR: usize = 15000;
const R_SUICIDE: usize = 24000;
const G_SUICIDE: usize = 5000;
const G_CREATE: usize = 32000;
const G_CODEDEPOSITE: usize = 200;
const G_CALL: usize = 700;
const G_CALLVALUE: usize = 9000;
const G_CALLSTIPEND: usize = 2300;
const G_NEWACCOUNT: usize = 25000;
const G_EXP: usize = 10;
const G_EXPBYTE: usize = 10;
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

fn memory_cost(a: Gas) -> Gas {
    (Gas::from(G_MEMORY) * a + a * a / Gas::from(512u64)).into()
}

fn sstore_cost<M: Machine>(machine: &M) -> Result<Gas> {
    let index = machine.stack().peek(0)?;
    let value = machine.stack().peek(1)?;
    let address = machine.transaction().callee();

    if value != M256::zero() && machine.block().account_storage(address, index) == M256::zero() {
        Ok(G_SSET.into())
    } else {
        Ok(G_SRESET.into())
    }
}

fn call_cost<M: Machine>(machine: &M) -> Result<Gas> {
    Ok(gascap_cost(machine)? + extra_cost(machine)?)
}

fn callgas_cost<M: Machine>(machine: &M) -> Result<Gas> {
    let val = machine.stack().peek(2)?;
    if val != M256::zero() {
        Ok(gascap_cost(machine)? + G_CALLSTIPEND.into())
    } else {
        Ok(gascap_cost(machine)?)
    }
}

fn gascap_cost<M: Machine>(machine: &M) -> Result<Gas> {
    let base1 = machine.available_gas() - extra_cost(machine)?;
    let base2 = machine.stack().peek(0)?.into();

    if machine.available_gas() >= extra_cost(machine)? {
        Ok(min(base1 - base1 / Gas::from(64u64), base2))
    } else {
        Ok(base2)
    }
}

fn extra_cost<M: Machine>(machine: &M) -> Result<Gas> {
    Ok(Gas::from(G_CALL) + xfer_cost(machine)? + new_cost(machine)?)
}

fn xfer_cost<M: Machine>(machine: &M) -> Result<Gas> {
    let val = machine.stack().peek(2)?;
    if val != M256::zero() {
        Ok(G_CALLVALUE.into())
    } else {
        Ok(Gas::zero())
    }
}

fn new_cost<M: Machine>(machine: &M) -> Result<Gas> {
    let address: Address = machine.stack().peek(1)?.into();
    if address == Address::default() {
        Ok(G_NEWACCOUNT.into())
    } else {
        Ok(Gas::zero())
    }
}

fn suicide_cost<M: Machine>(machine: &M) -> Result<Gas> {
    let address: Address = machine.stack().peek(1)?.into();
    Ok(Gas::from(G_SUICIDE) + if address == Address::default() {
        Gas::from(G_NEWACCOUNT)
    } else {
        Gas::zero()
    })
}

// TODO: Implement C_SSTORE, C_CALL and C_SELFDESTRUCT
// TODO: Use a machine_state struct instead of u_ip, u_i and u_s
// TODO: Use machine_state to implement gas cost for EXP,
// CALLDATACOPY, CODECOPY, EXTCODECOPY, LOG0-4, SHA3

impl Opcode {
    fn gas_cost_memory<M: Machine>(&self, machine: &M) -> Result<Gas> {
        let ref stack = machine.stack();
        let ref memory = machine.memory();
        let opcode = self.clone();

        let current = memory_cost(machine.memory().active_len().into());
        let next = match opcode {
            Opcode::SHA3 | Opcode::CODECOPY | Opcode::RETURN => {
                let from: U256 = stack.peek(0)?.into();
                let len: U256 = stack.peek(1)?.into();
                memory_cost(max(Gas::from(from) + Gas::from(len),
                                machine.memory().active_len().into()))
            },
            Opcode::MLOAD | Opcode::MSTORE => {
                let from: U256 = stack.peek(0)?.into();
                memory_cost(max(Gas::from(from) + Gas::from(32u64),
                                machine.memory().active_len().into()))
            },
            Opcode::MSTORE8 => {
                let from: U256 = stack.peek(0)?.into();
                memory_cost(max(Gas::from(from) + Gas::from(1u64),
                                machine.memory().active_len().into()))
            },
            Opcode::CREATE => {
                let from: U256 = stack.peek(1)?.into();
                let len: U256 = stack.peek(2)?.into();
                memory_cost(max(Gas::from(from) + Gas::from(len),
                                machine.memory().active_len().into()))
            },
            _ => {
                memory_cost(machine.memory().active_len().into())
            }
        };
        Ok(next - current)
    }

    pub fn gas_cost<M: Machine>(&self, machine: &M) -> Result<Gas> {
        let ref stack = machine.stack();
        let ref memory = machine.memory();
        let opcode = self.clone();
        let self_cost: Gas = match opcode {
            Opcode::CALL | Opcode::CALLCODE |
            Opcode::DELEGATECALL => call_cost(machine)?,

            Opcode::SUICIDE => suicide_cost(machine)?,

            Opcode::SSTORE => sstore_cost(machine)?,

            Opcode::SHA3 => {
                let len = stack.peek(1)?;
                (Gas::from(G_SHA3) + Gas::from(G_SHA3WORD) * (Gas::from(len) / Gas::from(32u64))).into()
            },

            Opcode::LOG(v) => {
                let len = stack.peek(1)?;
                (Gas::from(G_LOG) + Gas::from(G_LOGDATA) * Gas::from(len) + Gas::from(G_LOGTOPIC) * Gas::from(v)).into()
            },

            Opcode::EXTCODECOPY => {
                let len = stack.peek(2)?;
                (Gas::from(G_EXTCODE) + Gas::from(G_COPY) * (Gas::from(len) / Gas::from(32u64))).into()
            },

            Opcode::CALLDATACOPY | Opcode::CODECOPY => {
                let len = stack.peek(2)?;
                (Gas::from(G_VERYLOW) + Gas::from(G_COPY) * (Gas::from(len) / Gas::from(32u64))).into()
            },

            Opcode::EXP => {
                if stack.peek(1)? == M256::zero() {
                    Gas::from(G_EXP)
                } else {
                    (Gas::from(G_EXP) + Gas::from(G_EXPBYTE) * (Gas::from(1u64) + Gas::from(stack.peek(1)?.log2floor()))).into()
                }
            }

            Opcode::CREATE => G_CREATE.into(),
            Opcode::JUMPDEST => G_JUMPDEST.into(),
            Opcode::SLOAD => G_SLOAD.into(),

            // W_zero
            Opcode::STOP | Opcode::RETURN
                => G_ZERO.into(),

            // W_base
            Opcode::ADDRESS | Opcode::ORIGIN | Opcode::CALLER |
            Opcode::CALLVALUE | Opcode::CALLDATASIZE |
            Opcode::CODESIZE | Opcode::GASPRICE | Opcode::COINBASE |
            Opcode::TIMESTAMP | Opcode::NUMBER | Opcode::DIFFICULTY |
            Opcode::GASLIMIT | Opcode::POP | Opcode::PC |
            Opcode::MSIZE | Opcode::GAS
                => G_BASE.into(),

            // W_verylow
            Opcode::ADD | Opcode::SUB | Opcode::NOT | Opcode::LT |
            Opcode::GT | Opcode::SLT | Opcode::SGT | Opcode::EQ |
            Opcode::ISZERO | Opcode::AND | Opcode::OR | Opcode::XOR |
            Opcode::BYTE | Opcode::CALLDATALOAD | Opcode::MLOAD |
            Opcode::MSTORE | Opcode::MSTORE8 | Opcode::PUSH(_) |
            Opcode::DUP(_) | Opcode::SWAP(_)
                => G_VERYLOW.into(),

            // W_low
            Opcode::MUL | Opcode::DIV | Opcode::SDIV | Opcode::MOD |
            Opcode::SMOD | Opcode::SIGNEXTEND
                => G_LOW.into(),

            // W_mid
            Opcode::ADDMOD | Opcode::MULMOD | Opcode::JUMP
                => G_MID.into(),

            // W_high
            Opcode::JUMPI => G_HIGH.into(),

            // W_extcode
            Opcode::EXTCODESIZE => G_EXTCODE.into(),

            Opcode::BALANCE => G_BALANCE.into(),
            Opcode::BLOCKHASH => G_BLOCKHASH.into(),
            Opcode::INVALID => Gas::zero(),
        };
        Ok(self_cost + self.gas_cost_memory(machine)?)
    }
}
