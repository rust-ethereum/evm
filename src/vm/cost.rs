use utils::bigint::{M256, U256, U512};
use utils::gas::Gas;
use utils::address::Address;
use utils::opcode::Opcode;
use vm::{Machine, Memory, Stack, PC, ExecutionError, ExecutionResult, Storage};
use std::cmp::{min, max};

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
const G_CALL_DEFAULT: usize = 20;
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

fn memory_cost(a: Gas) -> Gas {
    (Gas::from(G_MEMORY) * a + a * a / Gas::from(512u64)).into()
}

fn sstore_cost<M: Memory + Default,
               S: Storage + Default>(machine: &Machine<M, S>) -> ExecutionResult<Gas> {
    let index = machine.stack().peek(0)?;
    let value = machine.stack().peek(1)?;
    let address = machine.owner()?;

    if value != M256::zero() && machine.account_storage(address)?.read(index)? == M256::zero() {
        Ok(G_SSET.into())
    } else {
        Ok(G_SRESET.into())
    }
}

fn call_cost<M: Memory + Default,
             S: Storage + Default>(machine: &Machine<M, S>, available_gas: Gas) -> ExecutionResult<Gas> {
    Ok(gascap_cost(machine, available_gas)? + extra_cost(machine)?)
}

fn callgas_cost<M: Memory + Default,
                S: Storage + Default>(machine: &Machine<M, S>, available_gas: Gas) -> ExecutionResult<Gas> {
    let val = machine.stack().peek(2)?;
    if val != M256::zero() {
        Ok(gascap_cost(machine, available_gas)? + G_CALLSTIPEND.into())
    } else {
        Ok(gascap_cost(machine, available_gas)?)
    }
}

fn gascap_cost<M: Memory + Default,
               S: Storage + Default>(machine: &Machine<M, S>, available_gas: Gas) -> ExecutionResult<Gas> {
    let base2 = machine.stack().peek(0)?.into();

    if available_gas >= extra_cost(machine)? {
        let base1 = available_gas - extra_cost(machine)?;
        Ok(min(base1 - base1 / Gas::from(64u64), base2))
    } else {
        Ok(base2)
    }
}

fn extra_cost<M: Memory + Default,
              S: Storage + Default>(machine: &Machine<M, S>) -> ExecutionResult<Gas> {
    Ok(Gas::from(if machine.eip150() { G_CALL_EIP150 } else { G_CALL_DEFAULT }) + xfer_cost(machine)? + new_cost(machine)?)
}

fn xfer_cost<M: Memory + Default,
             S: Storage + Default>(machine: &Machine<M, S>) -> ExecutionResult<Gas> {
    let val = machine.stack().peek(2)?;
    if val != M256::zero() {
        Ok(G_CALLVALUE.into())
    } else {
        Ok(Gas::zero())
    }
}

fn new_cost<M: Memory + Default,
            S: Storage + Default>(machine: &Machine<M, S>) -> ExecutionResult<Gas> {
    let address: Address = machine.stack().peek(1)?.into();
    if address == Address::default() {
        Ok(G_NEWACCOUNT.into())
    } else {
        Ok(Gas::zero())
    }
}

fn suicide_cost<M: Memory + Default,
                S: Storage + Default>(machine: &Machine<M, S>) -> ExecutionResult<Gas> {
    let address: Address = machine.stack().peek(0)?.into();
    Ok(Gas::from(if machine.eip150() { G_SUICIDE_EIP150 } else { G_SUICIDE_DEFAULT }) + if address == Address::default() {
        Gas::from(G_NEWACCOUNT)
    } else {
        Gas::zero()
    })
}

#[derive(Clone, Copy, Debug)]
pub struct CostAggregrator(Gas);

impl CostAggregrator {
    pub fn active_memory_len(&self) -> M256 {
        self.0.into()
    }
}

impl Default for CostAggregrator {
    fn default() -> CostAggregrator {
        CostAggregrator(Gas::zero())
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

fn memory_gas_cost<M: Memory + Default,
                   S: Storage + Default>(opcode: Opcode, machine: &Machine<M, S>,
                                          aggregrator: CostAggregrator)
                               -> ExecutionResult<(Gas, CostAggregrator)> {
    let ref stack = machine.stack();
    let ref memory = machine.memory();

    let current = aggregrator.0;
    let next = match opcode {
        Opcode::SHA3 | Opcode::RETURN | Opcode::LOG(_) => {
            let from: U256 = stack.peek(0)?.into();
            let len: U256 = stack.peek(1)?.into();
            memory_expand(current, Gas::from(from), Gas::from(len))
        },
        Opcode::CODECOPY | Opcode::CALLDATACOPY => {
            let from: U256 = stack.peek(0)?.into();
            let len: U256 = stack.peek(2)?.into();
            memory_expand(current, Gas::from(from), Gas::from(len))
        },
        Opcode::EXTCODECOPY => {
            let from: U256 = stack.peek(1)?.into();
            let len: U256 = stack.peek(3)?.into();
            memory_expand(current, Gas::from(from), Gas::from(len))
        },
        Opcode::MLOAD | Opcode::MSTORE => {
            let from: U256 = stack.peek(0)?.into();
            memory_expand(current, Gas::from(from), Gas::from(32u64))
        },
        Opcode::MSTORE8 => {
            let from: U256 = stack.peek(0)?.into();
            memory_expand(current, Gas::from(from), Gas::from(1u64))
        },
        Opcode::CREATE => {
            let from: U256 = stack.peek(1)?.into();
            let len: U256 = stack.peek(2)?.into();
            memory_expand(current, Gas::from(from), Gas::from(len))
        },
        _ => {
            current
        }
    };
    Ok((memory_cost(next) - memory_cost(current), CostAggregrator(next)))
}

pub fn gas_cost<M: Memory + Default,
                S: Storage + Default>(opcode: Opcode, machine: &Machine<M, S>,
                                       aggregrator: CostAggregrator)
                            -> ExecutionResult<(Gas, CostAggregrator)> {
    let self_cost: Gas = match opcode {
        Opcode::CALL | Opcode::CALLCODE |
        Opcode::DELEGATECALL => call_cost(machine, machine.available_gas())?,
        Opcode::SUICIDE => suicide_cost(machine)?,
        Opcode::SSTORE => sstore_cost(machine)?,

        Opcode::SHA3 => {
            let len = machine.stack.peek(1)?;
            let wordd = Gas::from(len) / Gas::from(32u64);
            let wordr = Gas::from(len) % Gas::from(32u64);
            (Gas::from(G_SHA3) + Gas::from(G_SHA3WORD) * if wordr == Gas::zero() { wordd } else { wordd + Gas::from(1u64) }).into()
        },

        Opcode::LOG(v) => {
            let len = machine.stack.peek(1)?;
            (Gas::from(G_LOG) + Gas::from(G_LOGDATA) * Gas::from(len) + Gas::from(G_LOGTOPIC) * Gas::from(v)).into()
        },

        Opcode::EXTCODECOPY => {
            let len = machine.stack.peek(2)?;
            let wordd = Gas::from(len) / Gas::from(32u64);
            let wordr = Gas::from(len) % Gas::from(32u64);
            (Gas::from(if machine.eip150() { G_EXTCODE_EIP150 } else { G_EXTCODE_DEFAULT }) + Gas::from(G_COPY) * if wordr == Gas::zero() { wordd } else { wordd + Gas::from(1u64) }).into()
        },

        Opcode::CALLDATACOPY | Opcode::CODECOPY => {
            let len = machine.stack.peek(2)?;
            let wordd = Gas::from(len) / Gas::from(32u64);
            let wordr = Gas::from(len) % Gas::from(32u64);
            (Gas::from(G_VERYLOW) + Gas::from(G_COPY) * if wordr == Gas::zero() { wordd } else { wordd + Gas::from(1u64) }).into()
        },

        Opcode::EXP => {
            if machine.stack.peek(1)? == M256::zero() {
                Gas::from(G_EXP)
            } else {
                Gas::from(G_EXP) + Gas::from(if machine.eip160() { G_EXPBYTE_EIP160 } else { G_EXPBYTE_DEFAULT }) * (Gas::from(1u64) + Gas::from(machine.stack.peek(1)?.log2floor()) / Gas::from(8u64))
            }
        }

        Opcode::CREATE => G_CREATE.into(),
        Opcode::JUMPDEST => G_JUMPDEST.into(),
        Opcode::SLOAD => (if machine.eip150() { G_SLOAD_EIP150 } else { G_SLOAD_DEFAULT }).into(),

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
        Opcode::EXTCODESIZE => (if machine.eip150() { G_EXTCODE_EIP150 } else { G_EXTCODE_DEFAULT }).into(),

        Opcode::BALANCE => (if machine.eip150() { G_BALANCE_EIP150 } else { G_BALANCE_DEFAULT }).into(),
        Opcode::BLOCKHASH => G_BLOCKHASH.into(),
        Opcode::INVALID => Gas::zero(),
    };
    let (memory_gas, agg) = memory_gas_cost(opcode, machine, aggregrator)?;
    Ok((self_cost + memory_gas, agg))
}

pub fn gas_refund<M: Memory + Default,
                  S: Storage + Default>(opcode: Opcode, machine: &Machine<M, S>) -> ExecutionResult<Gas> {
    match opcode {
        Opcode::SSTORE => {
            let index = machine.stack().peek(0)?;
            let value = machine.stack().peek(1)?;
            let address = machine.owner()?;

            if value == M256::zero() && machine.account_storage(address)?.read(index)? != M256::zero() {
                Ok(Gas::from(R_SCLEAR))
            } else {
                Ok(Gas::zero())
            }
        },
        Opcode::SUICIDE => {
            // TODO: check whether I_a belongs to A_s
            Ok(Gas::zero())
        },
        _ => Ok(Gas::zero())
    }
}
