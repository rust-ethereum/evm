use utils::bigint::{M256, MI256, U256, U512};
use utils::gas::Gas;
use utils::address::Address;
use super::Opcode;
use vm::{Machine, Memory, Stack, PC, Result, Error};
use vm::machine::MachineState;
use transaction::Transaction;
use blockchain::Block;

use std::ops::{Add, Sub, Not, Mul, Div, Shr, Shl, BitAnd, BitOr, BitXor, Rem};
use std::cmp::min;
use crypto::sha3::Sha3;
use crypto::digest::Digest;

fn call_code<M: MachineState>(
    machine: &mut M, gas: Gas, from: Address, to: Address, value: M256,
    mut memory_in_start: M256, memory_in_len: M256,
    mut memory_out_start: M256, memory_out_len: M256) -> M256 {

    let mut mem_in: Vec<u8> = Vec::new();

    let memory_in_end = memory_in_start + memory_in_len;
    while memory_in_start < memory_in_end {
        mem_in.push(machine.memory().read_raw(memory_in_start).unwrap());
        memory_in_start = memory_in_start + M256::one();
    }

    let code: Vec<u8> =
        if to == from {
            machine.pc().code().into()
        } else {
            machine.block().account_code(to).into()
        };
    let mem_out: Vec<u8> = machine.fork(
        gas, from, to, value, mem_in.as_ref(), code.as_ref(),
        |state| {
            let mut submachine = Machine::from_state(state);
            submachine.fire();

            let out: Vec<u8> = submachine.return_values().into();
            (out, submachine.into_state())
        });

    let memory_out_end = memory_out_start + memory_out_len;
    let mut i = 0;
    while memory_out_start < memory_out_end {
        machine.memory_mut().write_raw(memory_out_start,
                                       mem_out[i]);
        memory_out_start = memory_out_start + M256::one();
        i += 1;
    }

    M256::zero()
}

macro_rules! will_pop_push {
    ( $machine:expr, $pop_size:expr, $push_size:expr ) => ({
        if $machine.stack_mut().size() < $pop_size { return Err(Error::StackUnderflow); }
    })
}

macro_rules! op2 {
    ( $machine:expr, $op:ident ) => ({
        will_pop_push!($machine, 2, 1);

        begin_rescuable!($machine, &mut M, __);
        let op1 = $machine.stack_mut().pop().unwrap();
        let op2 = $machine.stack_mut().pop().unwrap();
        on_rescue!(|machine| {
            machine.stack_mut().push(op2);
            machine.stack_mut().push(op1);
        }, __);

        $machine.stack_mut().push(op1.$op(op2));
        end_rescuable!(__);
    })
}

macro_rules! op2_ref {
    ( $machine:expr, $op:ident ) => ({
        will_pop_push!($machine, 2, 1);

        begin_rescuable!($machine, &mut M, __);
        let op1 = $machine.stack_mut().pop().unwrap();
        let op2 = $machine.stack_mut().pop().unwrap();
        on_rescue!(|machine| {
            machine.stack_mut().push(op2);
            machine.stack_mut().push(op1);
        }, __);

        $machine.stack_mut().push(op1.$op(&op2).into());
        end_rescuable!(__);
    })
}

impl Opcode {
    pub fn run<M: MachineState>(&self, machine: &mut M) -> Result<()> {
        let opcode = self.clone();

        // Note: Please do not use try! or ? syntax in this opcode
        // running function. Anything that might fail after the stack
        // has poped may result the VM in invalid state. Instead, if
        // an operation might fail, manually restore the stack as well
        // as other VM structs before returning the error.
        match opcode {
            Opcode::STOP => {
                machine.pc_mut().stop();
            },

            Opcode::ADD => op2!(machine, add),
            Opcode::MUL => op2!(machine, mul),
            Opcode::SUB => op2!(machine, sub),
            Opcode::DIV => op2!(machine, div),

            Opcode::SDIV => {
                will_pop_push!(machine, 2, 1);

                let op1: MI256 = machine.stack_mut().pop().unwrap().into();
                let op2: MI256 = machine.stack_mut().pop().unwrap().into();
                let r = op1 / op2;
                machine.stack_mut().push(r.into());
            },

            Opcode::MOD => op2!(machine, rem),

            Opcode::SMOD => {
                will_pop_push!(machine, 2, 1);

                let op1: MI256 = machine.stack_mut().pop().unwrap().into();
                let op2: MI256 = machine.stack_mut().pop().unwrap().into();
                let r = op1 % op2;
                machine.stack_mut().push(r.into());
            },

            Opcode::ADDMOD => {
                will_pop_push!(machine, 2, 1);

                let op1: U256 = machine.stack_mut().pop().unwrap().into();
                let op2: U256 = machine.stack_mut().pop().unwrap().into();
                let op3: U256 = machine.stack_mut().pop().unwrap().into();

                let op1: U512 = op1.into();
                let op2: U512 = op2.into();
                let op3: U512 = op3.into();

                if op3 == U512::zero() {
                    machine.stack_mut().push(0.into());
                } else {
                    let v = (op1 + op2) % op3;
                    let v: U256 = v.into();
                    machine.stack_mut().push(v.into());
                }
            },

            Opcode::MULMOD => {
                will_pop_push!(machine, 2, 1);

                let op1: U256 = machine.stack_mut().pop().unwrap().into();
                let op2: U256 = machine.stack_mut().pop().unwrap().into();
                let op3: U256 = machine.stack_mut().pop().unwrap().into();

                let op1: U512 = op1.into();
                let op2: U512 = op2.into();
                let op3: U512 = op3.into();

                if op3 == U512::zero() {
                    machine.stack_mut().push(0.into());
                } else {
                    let v = (op1 * op2) % op3;
                    let v: U256 = v.into();
                    machine.stack_mut().push(v.into());
                }
            },

            Opcode::EXP => {
                will_pop_push!(machine, 2, 1);

                let mut op1 = machine.stack_mut().pop().unwrap();
                let mut op2 = machine.stack_mut().pop().unwrap();
                let mut r: M256 = 1.into();

                while op2 != 0.into() {
                    if op2 & 1.into() != 0.into() {
                        r = r * op1;
                    }
                    op2 = op2 >> 1;
                    op1 = op1 * op1;
                }

                machine.stack_mut().push(r);
            },

            Opcode::SIGNEXTEND => {
                will_pop_push!(machine, 2, 1);

                let mut op1 = machine.stack_mut().pop().unwrap();
                let mut op2 = machine.stack_mut().pop().unwrap();

                let mut ret = M256::zero();

                if op1 > M256::from(32) {
                    machine.stack_mut().push(op2);
                } else {
                    let len: usize = op1.into();
                    let t: usize = 8 * (len + 1) - 1;
                    let t_bit_mask = M256::one() << t;
                    let t_value = (op2 & t_bit_mask) >> t;
                    for i in 0..256 {
                        let bit_mask = M256::one() << i;
                        let i_value = (op2 & bit_mask) >> i;
                        if i <= t {
                            ret = ret + (i_value << i);
                        } else {
                            ret = ret + (t_value << i);
                        }
                    }
                    machine.stack_mut().push(ret);
                }
            },

            Opcode::LT => op2_ref!(machine, lt),
            Opcode::GT => op2_ref!(machine, gt),

            Opcode::SLT => {
                will_pop_push!(machine, 2, 1);

                let op1: MI256 = machine.stack_mut().pop().unwrap().into();
                let op2: MI256 = machine.stack_mut().pop().unwrap().into();

                machine.stack_mut().push(op1.lt(&op2).into());
            },

            Opcode::SGT => {
                will_pop_push!(machine, 2, 1);

                let op1: MI256 = machine.stack_mut().pop().unwrap().into();
                let op2: MI256 = machine.stack_mut().pop().unwrap().into();

                machine.stack_mut().push(op1.gt(&op2).into());
            },

            Opcode::EQ => op2_ref!(machine, eq),

            Opcode::ISZERO => {
                will_pop_push!(machine, 1, 1);

                let op1 = machine.stack_mut().pop().unwrap();

                if op1 == 0.into() {
                    machine.stack_mut().push(1.into());
                } else {
                    machine.stack_mut().push(0.into());
                }
            },

            Opcode::AND => op2!(machine, bitand),
            Opcode::OR => op2!(machine, bitor),
            Opcode::XOR => op2!(machine, bitxor),

            Opcode::NOT => {
                will_pop_push!(machine, 1, 1);

                let op1 = machine.stack_mut().pop().unwrap();

                machine.stack_mut().push(!op1);
            },

            Opcode::BYTE => {
                will_pop_push!(machine, 2, 1);

                let op1 = machine.stack_mut().pop().unwrap();
                let op2 = machine.stack_mut().pop().unwrap();

                let mut ret = M256::zero();

                for i in 0..256 {
                    if i < 8 && op1 < 32.into() {
                        let o: usize = op1.into();
                        let t = 255 - (7 - i + 8 * o);
                        let bit_mask = M256::one() << t;
                        let value = (op2 & bit_mask) >> t;
                        ret = ret + (value << i);
                    }
                }

                machine.stack_mut().push(ret);
            },

            Opcode::SHA3 => {
                will_pop_push!(machine, 2, 1);

                begin_rescuable!(machine, &mut M, __);
                let mut from = machine.stack_mut().pop().unwrap();
                let from0 = from;
                let len = machine.stack_mut().pop().unwrap();
                on_rescue!(|machine| {
                    machine.stack_mut().push(len);
                    machine.stack_mut().push(from0);
                }, __);
                let ender = from + len;
                if ender < from {
                    trr!(Err(Error::MemoryTooLarge), __);
                }

                let mut ret = [0u8; 32];
                let mut sha3 = Sha3::keccak256();

                while from < ender {
                    let val = trr!(machine.memory_mut().read_raw(from), __);
                    let a: [u8; 1] = [ val ];
                    sha3.input(a.as_ref());
                    from = from + 1.into();
                }
                sha3.result(&mut ret);
                machine.stack_mut().push(M256::from(ret.as_ref()));
                end_rescuable!(__);
            },

            Opcode::ADDRESS => {
                will_pop_push!(machine, 0, 1);

                let address = machine.transaction().callee();
                machine.stack_mut().push(address.into());
            },

            Opcode::BALANCE => {
                will_pop_push!(machine, 1, 1);

                let address: Address = machine.stack_mut().pop().unwrap().into();
                let balance = machine.block().balance(address).into();
                machine.stack_mut().push(balance);
            },

            Opcode::ORIGIN => {
                will_pop_push!(machine, 0, 1);

                let address = machine.transaction().originator();
                machine.stack_mut().push(address.into());
            },

            Opcode::CALLER => {
                will_pop_push!(machine, 0, 1);

                let address = machine.transaction().sender();
                machine.stack_mut().push(address.into());
            },

            Opcode::CALLVALUE => {
                will_pop_push!(machine, 0, 1);

                let value = machine.transaction().value();
                machine.stack_mut().push(value);
            },

            Opcode::CALLDATALOAD => {
                will_pop_push!(machine, 1, 1);

                begin_rescuable!(machine, &mut M, __);
                let start_index = machine.stack_mut().pop().unwrap();
                on_rescue!(|machine| {
                    machine.stack_mut().push(start_index);
                }, __);

                if start_index > usize::max_value().into() {
                    trr!(Err(Error::DataTooLarge), __);
                }
                let start_index: usize = start_index.into();
                if start_index.checked_add(32).is_none() {
                    trr!(Err(Error::DataTooLarge), __);
                }

                let data: Vec<u8> = machine.transaction().data().unwrap().into();
                let mut load: [u8; 32] = [0u8; 32];
                for i in 0..32 {
                    if start_index + i < data.len() {
                        load[i] = data[start_index + i];
                    }
                }
                machine.stack_mut().push(load.into());
                end_rescuable!(__);
            },

            Opcode::CALLDATASIZE => {
                will_pop_push!(machine, 0, 1);

                let len = machine.transaction().data().map_or(0, |s| s.len());
                machine.stack_mut().push(len.into());
            },

            Opcode::CALLDATACOPY => {
                will_pop_push!(machine, 3, 0);

                begin_rescuable!(machine, &mut M, __);
                let memory_index = machine.stack_mut().pop().unwrap();
                let data_index = machine.stack_mut().pop().unwrap();
                let len = machine.stack_mut().pop().unwrap();

                on_rescue!(|machine| {
                    machine.stack_mut().push(len);
                    machine.stack_mut().push(data_index);
                    machine.stack_mut().push(memory_index);
                }, __);

                if data_index > usize::max_value().into() {
                    trr!(Err(Error::DataTooLarge), __);
                }
                let data_index: usize = data_index.into();

                if len > usize::max_value().into() {
                    trr!(Err(Error::DataTooLarge), __);
                }
                let len: usize = len.into();

                if data_index.checked_add(len).is_none() {
                    trr!(Err(Error::DataTooLarge), __);
                }

                for i in 0..len {
                    if machine.transaction().data().is_some() {
                        let data: Vec<u8> = machine.transaction().data().unwrap().into();
                        if data_index + i < data.len() {
                            let val = data[data_index + i];
                            machine.memory_mut().write_raw(memory_index + i.into(), val);
                        }
                    }
                }
                end_rescuable!(__);
            },

            Opcode::CODESIZE => {
                will_pop_push!(machine, 0, 1);

                let len = machine.pc().code().len();
                machine.stack_mut().push(len.into());
            },

            Opcode::CODECOPY => {
                will_pop_push!(machine, 1, 1);

                let memory_index = machine.stack_mut().pop().unwrap();
                let code_index = machine.stack_mut().pop().unwrap();
                let len = machine.stack_mut().pop().unwrap();

                let restore = |machine: &mut M| {
                    machine.stack_mut().push(len);
                    machine.stack_mut().push(code_index);
                    machine.stack_mut().push(memory_index);
                };

                if code_index > usize::max_value().into() {
                    restore(machine);
                    return Err(Error::CodeTooLarge);
                }
                let code_index: usize = code_index.into();

                if len > usize::max_value().into() {
                    restore(machine);
                    return Err(Error::CodeTooLarge);
                }
                let len: usize = len.into();

                if code_index.checked_add(len).is_none() {
                    restore(machine);
                    return Err(Error::CodeTooLarge);
                }

                for i in 0..len {
                    let code: Vec<u8> = machine.pc().code().into();
                    if code_index + i < code.len() {
                        let val = code[code_index + i];
                        machine.memory_mut().write_raw(memory_index + i.into(), val);
                    }
                }
            },

            Opcode::GASPRICE => {
                will_pop_push!(machine, 0, 1);

                let price: M256 = machine.transaction().gas_price().into();
                machine.stack_mut().push(price);
            },

            Opcode::EXTCODESIZE => {
                will_pop_push!(machine, 1, 1);

                let account: Address = machine.stack_mut().pop().unwrap().into();
                let len = machine.block().account_code(account).len();
                machine.stack_mut().push(len.into());
            },

            Opcode::EXTCODECOPY => {
                will_pop_push!(machine, 4, 0);

                let account = machine.stack_mut().pop().unwrap();
                let memory_index = machine.stack_mut().pop().unwrap();
                let code_index = machine.stack_mut().pop().unwrap();
                let len = machine.stack_mut().pop().unwrap();

                let restore = |machine: &mut M| {
                    machine.stack_mut().push(len);
                    machine.stack_mut().push(code_index);
                    machine.stack_mut().push(memory_index);
                    machine.stack_mut().push(account);
                };

                let account: Address = account.into();

                if code_index > usize::max_value().into() {
                    restore(machine);
                    return Err(Error::CodeTooLarge);
                }
                let code_index: usize = code_index.into();

                if len > usize::max_value().into() {
                    restore(machine);
                    return Err(Error::CodeTooLarge);
                }
                let len: usize = len.into();

                if code_index.checked_add(len).is_none() {
                    restore(machine);
                    return Err(Error::CodeTooLarge);
                }

                for i in 0..len {
                    let code: Vec<u8> = machine.block().account_code(account).into();
                    if code_index + i < code.len() {
                        let val = code[code_index + i];
                        machine.memory_mut().write_raw(memory_index + i.into(), val);
                    }
                }
            },

            Opcode::BLOCKHASH => {
                will_pop_push!(machine, 1, 1);

                let target = machine.stack_mut().pop().unwrap();
                let val = machine.block().blockhash(target);
                machine.stack_mut().push(val.into());
            },

            Opcode::COINBASE => {
                will_pop_push!(machine, 0, 1);

                let val = machine.block().coinbase();
                machine.stack_mut().push(val.into());
            },

            Opcode::TIMESTAMP => {
                will_pop_push!(machine, 0, 1);

                let val = machine.block().timestamp();
                machine.stack_mut().push(val.into());
            },

            Opcode::NUMBER => {
                will_pop_push!(machine, 0, 1);

                let val = machine.block().number();
                machine.stack_mut().push(val.into());
            },

            Opcode::DIFFICULTY => {
                will_pop_push!(machine, 0, 1);

                let val = machine.block().difficulty();
                machine.stack_mut().push(val.into());
            },

            Opcode::GASLIMIT => {
                will_pop_push!(machine, 0, 1);

                let val = machine.block().gas_limit();
                machine.stack_mut().push(val.into());
            },

            Opcode::POP => {
                will_pop_push!(machine, 1, 0);

                machine.stack_mut().pop().unwrap();
            },

            Opcode::MLOAD => {
                will_pop_push!(machine, 1, 1);

                begin_rescuable!(machine, &mut M, __);
                let op1 = machine.stack_mut().pop().unwrap();
                on_rescue!(|machine| {
                    machine.stack_mut().push(op1);
                }, __);
                let val = trr!(machine.memory_mut().read(op1), __);
                machine.stack_mut().push(val);
                end_rescuable!(__);
            },

            Opcode::MSTORE => {
                will_pop_push!(machine, 2, 0);

                let op1 = machine.stack_mut().pop().unwrap(); // Index
                let op2 = machine.stack_mut().pop().unwrap(); // Data
                // u_i update is automatically handled by Memory.
                machine.memory_mut().write(op1, op2);
            },

            Opcode::MSTORE8 => {
                will_pop_push!(machine, 2, 0);

                let op1 = machine.stack_mut().pop().unwrap(); // Index
                let op2 = machine.stack_mut().pop().unwrap(); // Data
                let a: [u8; 32] = op2.into();
                let val = a[31];
                machine.memory_mut().write_raw(op1, val);
            },

            Opcode::SLOAD => {
                will_pop_push!(machine, 1, 1);

                let op1 = machine.stack_mut().pop().unwrap();
                let from = machine.transaction().callee();
                let val = machine.block().account_storage(from, op1);
                machine.stack_mut().push(val);
            },

            Opcode::SSTORE => {
                will_pop_push!(machine, 2, 0);

                let op1 = machine.stack_mut().pop().unwrap(); // Index
                let op2 = machine.stack_mut().pop().unwrap(); // Data
                let from = machine.transaction().callee();
                machine.block_mut().set_account_storage(from, op1, op2);
            }

            Opcode::JUMP => {
                will_pop_push!(machine, 1, 0);

                let op1 = machine.stack_mut().pop().unwrap();

                if op1 > usize::max_value().into() {
                    machine.stack_mut().push(op1);
                    return Err(Error::PCTooLarge);
                }

                machine.pc_mut().jump(op1.into());
            },

            Opcode::JUMPI => {
                will_pop_push!(machine, 2, 0);

                let op1 = machine.stack_mut().pop().unwrap();

                if op1 > usize::max_value().into() {
                    machine.stack_mut().push(op1);
                    return Err(Error::PCTooLarge);
                }

                let op2 = machine.stack_mut().pop().unwrap();

                if op2 != 0.into() {
                    machine.pc_mut().jump(op1.into());
                }
            },

            Opcode::PC => {
                will_pop_push!(machine, 0, 1);

                let position = machine.pc().position();
                machine.stack_mut().push((position - 1).into()); // PC increment for opcode is always an u8.
            },

            Opcode::MSIZE => {
                will_pop_push!(machine, 0, 1);

                let active_memory_len = machine.active_memory_len();
                machine.stack_mut().push(M256::from(32u64) * active_memory_len);
            },

            Opcode::GAS => {
                will_pop_push!(machine, 0, 1);

                let gas: M256 = machine.transaction().gas_limit().into();
                machine.stack_mut().push(gas);
            },

            Opcode::JUMPDEST => {
                will_pop_push!(machine, 0, 0);
                ()
            }, // This operation has no effect on machine state during execution.

            Opcode::PUSH(v) => {
                will_pop_push!(machine, 0, 1);

                let val = machine.pc_mut().read(v)?; // We don't have any stack to restore, so this ? is okay.
                machine.stack_mut().push(val);
            },

            Opcode::DUP(v) => {
                will_pop_push!(machine, v, v+1);

                let val = machine.stack().peek(v - 1).unwrap();
                machine.stack_mut().push(val);
            },

            Opcode::SWAP(v) => {
                will_pop_push!(machine, v+1, v+1);

                let val1 = machine.stack().peek(0).unwrap();
                let val2 = machine.stack().peek(v).unwrap();
                machine.stack_mut().set(0, val2).unwrap();
                machine.stack_mut().set(v, val1).unwrap();
            },

            Opcode::LOG(v) => {
                will_pop_push!(machine, v+2, 0);

                begin_rescuable!(machine, &mut M, __);
                let address = machine.transaction().callee();
                let mut data: Vec<u8> = Vec::new();
                let mut start = machine.stack_mut().pop().unwrap();
                let start0 = start;
                let len = machine.stack_mut().pop().unwrap();
                let ender = start + len;
                on_rescue!(|machine| {
                    machine.stack_mut().push(len);
                    machine.stack_mut().push(start0);
                }, __);
                if ender < start {
                    trr!(Err(Error::MemoryTooLarge), __);
                }

                while start < ender {
                    data.push(trr!(machine.memory().read_raw(start), __));
                    start = start + M256::one();
                }
                end_rescuable!(__);

                let mut topics: Vec<M256> = Vec::new();

                for i in 0..v {
                    topics.push(machine.stack_mut().pop().unwrap());
                }

                machine.block_mut().log(address, data.as_ref(), topics.as_ref());
            },

            Opcode::CREATE => {
                will_pop_push!(machine, 3, 1);

                // TODO: Register the transaction for its value.
                let value = machine.stack_mut().pop().unwrap();
                let start: usize = machine.stack_mut().pop().unwrap().into();
                let len: usize = machine.stack_mut().pop().unwrap().into();
                let code: Vec<u8> = machine.pc().code()[start..(start + len)].into();
                let address = machine.block_mut().create_account(code.as_ref());
                machine.stack_mut().push(address.unwrap().into());
            },

            Opcode::CALL => {
                will_pop_push!(machine, 7, 1);

                let gas: Gas = machine.stack_mut().pop().unwrap().into();
                let from = machine.transaction().callee();
                let to: Address = machine.stack_mut().pop().unwrap().into();
                let value = machine.stack_mut().pop().unwrap().into();
                let memory_in_start = machine.stack_mut().pop().unwrap();
                let memory_in_len = machine.stack_mut().pop().unwrap();
                let memory_out_start = machine.stack_mut().pop().unwrap();
                let memory_out_len = machine.stack_mut().pop().unwrap();

                let ret = call_code(machine, gas, from, to, value,
                                    memory_in_start, memory_in_len,
                                    memory_out_start, memory_out_len);

                machine.stack_mut().push(ret);
            },

            Opcode::CALLCODE => {
                will_pop_push!(machine, 7, 1);

                let gas: Gas = machine.stack_mut().pop().unwrap().into();
                machine.stack_mut().pop().unwrap();
                let from = machine.transaction().callee();
                let to = machine.transaction().callee();
                let value = machine.stack_mut().pop().unwrap().into();
                let memory_in_start = machine.stack_mut().pop().unwrap();
                let memory_in_len = machine.stack_mut().pop().unwrap();
                let memory_out_start = machine.stack_mut().pop().unwrap();
                let memory_out_len = machine.stack_mut().pop().unwrap();

                let ret = call_code(machine, gas, from, to, value,
                                    memory_in_start, memory_in_len,
                                    memory_out_start, memory_out_len);

                machine.stack_mut().push(ret);
            },

            Opcode::RETURN => {
                will_pop_push!(machine, 2, 0);

                begin_rescuable!(machine, &mut M, __);
                let mut start = machine.stack_mut().pop().unwrap();
                let start0 = start;
                let len = machine.stack_mut().pop().unwrap();
                let ender = start + len;
                on_rescue!(|machine| {
                    machine.stack_mut().push(len);
                    machine.stack_mut().push(start0);
                }, __);
                if ender < start {
                    trr!(Err(Error::MemoryTooLarge), __);
                }
                let mut vec: Vec<u8> = Vec::new();

                while start < ender {
                    vec.push(trr!(machine.memory().read_raw(start), __));
                    start = start + M256::one();
                }

                machine.set_return_values(vec.as_ref());
                machine.pc_mut().stop();
                end_rescuable!(__);
            },

            Opcode::DELEGATECALL => {
                will_pop_push!(machine, 6, 1);

                let gas: Gas = machine.stack_mut().pop().unwrap().into();
                let from = machine.transaction().sender();
                let to: Address = machine.stack_mut().pop().unwrap().into();
                let value = machine.transaction().value();
                let memory_in_start = machine.stack_mut().pop().unwrap();
                let memory_in_len = machine.stack_mut().pop().unwrap();
                let memory_out_start = machine.stack_mut().pop().unwrap();
                let memory_out_len = machine.stack_mut().pop().unwrap();

                let ret = call_code(machine, gas, from, to, value,
                                    memory_in_start, memory_in_len,
                                    memory_out_start, memory_out_len);

                machine.stack_mut().push(ret);
            },

            Opcode::SUICIDE => {
                will_pop_push!(machine, 1, 0);

                machine.stack_mut().pop().unwrap();
                machine.pc_mut().stop();

                let callee = machine.transaction().callee();
                let code: Vec<u8> = machine.pc().code().into();
                machine.block_mut().set_account_code(callee, code.as_ref());
            },

            Opcode::INVALID => {
                machine.pc_mut().stop();
                return Err(Error::InvalidOpcode);
            }
        }
        Ok(())
    }
}
