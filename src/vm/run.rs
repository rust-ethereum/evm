use utils::bigint::{M256, MI256, U256, U512};
use utils::gas::Gas;
use utils::address::Address;
use utils::opcode::Opcode;
use vm::{Machine, Memory, Stack, PC, ExecutionResult, ExecutionError, Storage, BlockHeader, Transaction};

use std::ops::{Add, Sub, Not, Mul, Div, Shr, Shl, BitAnd, BitOr, BitXor, Rem};
use std::cmp::min;
use crypto::sha3::Sha3;
use crypto::digest::Digest;

fn call_code<M: Memory, S: Storage>(
    machine: &mut Machine<M, S>, gas: Gas, from: Address, to: Address, value: M256,
    mut memory_in_start: M256, memory_in_len: M256,
    mut memory_out_start: M256, memory_out_len: M256) -> M256 {
    unimplemented!()
}

macro_rules! will_pop_push {
    ( $machine:expr, $pop_size:expr, $push_size:expr ) => ({
        if $machine.stack.len() < $pop_size { return Err(ExecutionError::StackUnderflow); }
        if $machine.stack.len() - $pop_size + $push_size > 1024 { return Err(ExecutionError::StackOverflow); }
    })
}

macro_rules! op2 {
    ( $machine:expr, $op:ident ) => ({
        will_pop_push!($machine, 2, 1);

        begin_rescuable!($machine, &mut Machine<M, S>, __);
        let op1 = $machine.stack.pop().unwrap();
        let op2 = $machine.stack.pop().unwrap();
        on_rescue!(|machine| {
            machine.stack.push(op2).unwrap();
            machine.stack.push(op1).unwrap();
        }, __);

        $machine.stack.push(op1.$op(op2)).unwrap();
        end_rescuable!(__);
    })
}

macro_rules! op2_ref {
    ( $machine:expr, $op:ident ) => ({
        will_pop_push!($machine, 2, 1);

        begin_rescuable!($machine, &mut Machine<M, S>, __);
        let op1 = $machine.stack.pop().unwrap();
        let op2 = $machine.stack.pop().unwrap();
        on_rescue!(|machine| {
            machine.stack.push(op2).unwrap();
            machine.stack.push(op1).unwrap();
        }, __);

        $machine.stack.push(op1.$op(&op2).into()).unwrap();
        end_rescuable!(__);
    })
}

pub fn run_opcode<M: Memory, S: Storage>(opcode: Opcode, machine: &mut Machine<M, S>, after_gas: Gas)
                                         -> ExecutionResult<()> {
    // Note: Please do not use try! or ? syntax in this opcode
    // running function. Anything that might fail after the stack
    // has poped may result the VM in invalid state. Instead, if
    // an operation might fail, manually restore the stack as well
    // as other VM structs before returning the error.
    match opcode {
        Opcode::STOP => {
            machine.pc.stop();
        },

        Opcode::ADD => op2!(machine, add),
        Opcode::MUL => op2!(machine, mul),
        Opcode::SUB => op2!(machine, sub),
        Opcode::DIV => op2!(machine, div),

        Opcode::SDIV => {
            will_pop_push!(machine, 2, 1);

            let op1: MI256 = machine.stack.pop().unwrap().into();
            let op2: MI256 = machine.stack.pop().unwrap().into();
            let r = op1 / op2;
            machine.stack.push(r.into()).unwrap();
        },

        Opcode::MOD => op2!(machine, rem),

        Opcode::SMOD => {
            will_pop_push!(machine, 2, 1);

            let op1: MI256 = machine.stack.pop().unwrap().into();
            let op2: MI256 = machine.stack.pop().unwrap().into();
            let r = op1 % op2;
            machine.stack.push(r.into()).unwrap();
        },

        Opcode::ADDMOD => {
            will_pop_push!(machine, 2, 1);

            let op1: U256 = machine.stack.pop().unwrap().into();
            let op2: U256 = machine.stack.pop().unwrap().into();
            let op3: U256 = machine.stack.pop().unwrap().into();

            let op1: U512 = op1.into();
            let op2: U512 = op2.into();
            let op3: U512 = op3.into();

            if op3 == U512::zero() {
                machine.stack.push(0.into()).unwrap();
            } else {
                let v = (op1 + op2) % op3;
                let v: U256 = v.into();
                machine.stack.push(v.into()).unwrap();
            }
        },

        Opcode::MULMOD => {
            will_pop_push!(machine, 2, 1);

            let op1: U256 = machine.stack.pop().unwrap().into();
            let op2: U256 = machine.stack.pop().unwrap().into();
            let op3: U256 = machine.stack.pop().unwrap().into();

            let op1: U512 = op1.into();
            let op2: U512 = op2.into();
            let op3: U512 = op3.into();

            if op3 == U512::zero() {
                machine.stack.push(0.into()).unwrap();
            } else {
                let v = (op1 * op2) % op3;
                let v: U256 = v.into();
                machine.stack.push(v.into()).unwrap();
            }
        },

        Opcode::EXP => {
            will_pop_push!(machine, 2, 1);

            let mut op1 = machine.stack.pop().unwrap();
            let mut op2 = machine.stack.pop().unwrap();
            let mut r: M256 = 1.into();

            while op2 != 0.into() {
                if op2 & 1.into() != 0.into() {
                    r = r * op1;
                }
                op2 = op2 >> 1;
                op1 = op1 * op1;
            }

            machine.stack.push(r).unwrap();
        },

        Opcode::SIGNEXTEND => {
            will_pop_push!(machine, 2, 1);

            let mut op1 = machine.stack.pop().unwrap();
            let mut op2 = machine.stack.pop().unwrap();

            let mut ret = M256::zero();

            if op1 > M256::from(32) {
                machine.stack.push(op2).unwrap();
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
                machine.stack.push(ret).unwrap();
            }
        },

        Opcode::LT => op2_ref!(machine, lt),
        Opcode::GT => op2_ref!(machine, gt),

        Opcode::SLT => {
            will_pop_push!(machine, 2, 1);

            let op1: MI256 = machine.stack.pop().unwrap().into();
            let op2: MI256 = machine.stack.pop().unwrap().into();

            machine.stack.push(op1.lt(&op2).into()).unwrap();
        },

        Opcode::SGT => {
            will_pop_push!(machine, 2, 1);

            let op1: MI256 = machine.stack.pop().unwrap().into();
            let op2: MI256 = machine.stack.pop().unwrap().into();

            machine.stack.push(op1.gt(&op2).into()).unwrap();
        },

        Opcode::EQ => op2_ref!(machine, eq),

        Opcode::ISZERO => {
            will_pop_push!(machine, 1, 1);

            let op1 = machine.stack.pop().unwrap();

            if op1 == 0.into() {
                machine.stack.push(1.into()).unwrap();
            } else {
                machine.stack.push(0.into()).unwrap();
            }
        },

        Opcode::AND => op2!(machine, bitand),
        Opcode::OR => op2!(machine, bitor),
        Opcode::XOR => op2!(machine, bitxor),

        Opcode::NOT => {
            will_pop_push!(machine, 1, 1);

            let op1 = machine.stack.pop().unwrap();

            machine.stack.push(!op1).unwrap();
        },

        Opcode::BYTE => {
            will_pop_push!(machine, 2, 1);

            let op1 = machine.stack.pop().unwrap();
            let op2 = machine.stack.pop().unwrap();

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

            machine.stack.push(ret).unwrap();
        },

        Opcode::SHA3 => {
            will_pop_push!(machine, 2, 1);

            begin_rescuable!(machine, &mut Machine<M, S>, __);
            let mut from = machine.stack.pop().unwrap();
            let from0 = from;
            let len = machine.stack.pop().unwrap();
            on_rescue!(|machine| {
                machine.stack.push(len).unwrap();
                machine.stack.push(from0).unwrap();
            }, __);
            let ender = from + len;
            if ender < from {
                trr!(Err(ExecutionError::MemoryTooLarge), __);
            }

            let mut ret = [0u8; 32];
            let mut sha3 = Sha3::keccak256();

            while from < ender {
                let val = trr!(machine.memory.read_raw(from), __);
                let a: [u8; 1] = [ val ];
                sha3.input(a.as_ref());
                from = from + 1.into();
            }
            sha3.result(&mut ret);
            machine.stack.push(M256::from(ret.as_ref())).unwrap();
            end_rescuable!(__);
        },

        Opcode::ADDRESS => {
            will_pop_push!(machine, 0, 1);

            let address = machine.owner();
            machine.stack.push(address.into()).unwrap();
        },

        Opcode::BALANCE => {
            will_pop_push!(machine, 1, 1);

            begin_rescuable!(machine, &mut Machine<M, S>, __);
            let address = machine.stack.pop().unwrap();
            on_rescue!(|machine| {
                machine.stack.push(address).unwrap();
            }, __);
            let address: Address = address.into();
            let balance = trr!(machine.account_balance(address), __).into();
            machine.stack.push(balance).unwrap();
        },

        Opcode::ORIGIN => {
            will_pop_push!(machine, 0, 1);

            let address = machine.transaction().originator();
            machine.stack.push(address.into()).unwrap();
        },

        Opcode::CALLER => {
            will_pop_push!(machine, 0, 1);

            let address = machine.transaction().caller();
            machine.stack.push(address.into()).unwrap();
        },

        Opcode::CALLVALUE => {
            will_pop_push!(machine, 0, 1);

            let value = machine.transaction().value();
            machine.stack.push(value).unwrap();
        },

        Opcode::CALLDATALOAD => {
            will_pop_push!(machine, 1, 1);

            begin_rescuable!(machine, &mut Machine<M, S>, __);
            let start_index = machine.stack.pop().unwrap();
            on_rescue!(|machine| {
                machine.stack.push(start_index).unwrap();
            }, __);

            if start_index > usize::max_value().into() {
                trr!(Err(ExecutionError::DataTooLarge), __);
            }
            let start_index: usize = start_index.into();
            if start_index.checked_add(32).is_none() {
                trr!(Err(ExecutionError::DataTooLarge), __);
            }

            let data: Vec<u8> = match machine.transaction() {
                &Transaction::MessageCall {
                    data: ref data,
                    ..
                } => {
                    data.clone()
                },
                &Transaction::ContractCreation {
                    ..
                } => {
                    Vec::new()
                },
            };
            let mut load: [u8; 32] = [0u8; 32];
            for i in 0..32 {
                if start_index + i < data.len() {
                    load[i] = data[start_index + i];
                }
            }
            machine.stack.push(load.into()).unwrap();
            end_rescuable!(__);
        },

        Opcode::CALLDATASIZE => {
            will_pop_push!(machine, 0, 1);

            let len = match machine.transaction() {
                &Transaction::MessageCall {
                    data: ref data,
                    ..
                } => data.len(),
                &Transaction::ContractCreation {
                    ..
                } => 0,
            };
            machine.stack.push(len.into()).unwrap();
        },

        Opcode::CALLDATACOPY => {
            will_pop_push!(machine, 3, 0);

            begin_rescuable!(machine, &mut Machine<M, S>, __);
            let memory_index = machine.stack.pop().unwrap();
            let data_index = machine.stack.pop().unwrap();
            let len = machine.stack.pop().unwrap();

            on_rescue!(|machine| {
                machine.stack.push(len).unwrap();
                machine.stack.push(data_index).unwrap();
                machine.stack.push(memory_index).unwrap();
            }, __);

            if data_index > usize::max_value().into() {
                trr!(Err(ExecutionError::DataTooLarge), __);
            }
            let data_index: usize = data_index.into();

            if len > usize::max_value().into() {
                trr!(Err(ExecutionError::DataTooLarge), __);
            }
            let len: usize = len.into();

            if data_index.checked_add(len).is_none() {
                trr!(Err(ExecutionError::DataTooLarge), __);
            }

            let data = match machine.transaction() {
                &Transaction::MessageCall {
                    data: ref data,
                    ..
                } => data.clone(),
                &Transaction::ContractCreation {
                    ..
                } => Vec::new(),
            };
            for i in 0..len {
                if data_index + i < data.len() {
                    let val = data[data_index + i];
                    machine.memory.write_raw(memory_index + i.into(), val);
                } else {
                    machine.memory.write_raw(memory_index + i.into(), 0);
                }
            }
            end_rescuable!(__);
        },

        Opcode::CODESIZE => {
            will_pop_push!(machine, 0, 1);

            let len = machine.pc.code().len();
            machine.stack.push(len.into()).unwrap();
        },

        Opcode::CODECOPY => {
            will_pop_push!(machine, 1, 1);

            let memory_index = machine.stack.pop().unwrap();
            let code_index = machine.stack.pop().unwrap();
            let len = machine.stack.pop().unwrap();

            let restore = |machine: &mut Machine<M, S>| {
                machine.stack.push(len).unwrap();
                machine.stack.push(code_index).unwrap();
                machine.stack.push(memory_index).unwrap();
            };

            if code_index > usize::max_value().into() {
                restore(machine);
                return Err(ExecutionError::CodeTooLarge);
            }
            let code_index: usize = code_index.into();

            if len > usize::max_value().into() {
                restore(machine);
                return Err(ExecutionError::CodeTooLarge);
            }
            let len: usize = len.into();

            if code_index.checked_add(len).is_none() {
                restore(machine);
                return Err(ExecutionError::CodeTooLarge);
            }

            for i in 0..len {
                let code: Vec<u8> = machine.pc.code().into();
                if code_index + i < code.len() {
                    let val = code[code_index + i];
                    machine.memory.write_raw(memory_index + i.into(), val);
                }
            }
        },

        Opcode::GASPRICE => {
            will_pop_push!(machine, 0, 1);

            let price: M256 = machine.transaction().gas_price().into();
            machine.stack.push(price).unwrap();
        },

        Opcode::EXTCODESIZE => {
            will_pop_push!(machine, 1, 1);

            begin_rescuable!(machine, &mut Machine<M, S>, __);
            let account = machine.stack.pop().unwrap();
            on_rescue!(|machine| {
                machine.stack.push(account).unwrap();
            }, __);
            let account: Address = account.into();
            let len = trr!(machine.account_code(account).and_then(|code| Ok(code.len())), __);
            machine.stack.push(len.into()).unwrap();
            end_rescuable!(__);
        },

        Opcode::EXTCODECOPY => {
            will_pop_push!(machine, 4, 0);

            begin_rescuable!(machine, &mut Machine<M, S>, __);
            let account = machine.stack.pop().unwrap();
            let memory_index = machine.stack.pop().unwrap();
            let code_index = machine.stack.pop().unwrap();
            let len = machine.stack.pop().unwrap();
            on_rescue!(|machine| {
                machine.stack.push(len).unwrap();
                machine.stack.push(code_index).unwrap();
                machine.stack.push(memory_index).unwrap();
                machine.stack.push(account).unwrap();
            }, __);

            let account: Address = account.into();

            if code_index > usize::max_value().into() {
                trr!(Err(ExecutionError::CodeTooLarge), __);
            }
            let code_index: usize = code_index.into();

            if len > usize::max_value().into() {
                trr!(Err(ExecutionError::CodeTooLarge), __);
            }
            let len: usize = len.into();

            if code_index.checked_add(len).is_none() {
                trr!(Err(ExecutionError::CodeTooLarge), __);
            }

            for i in 0..len {
                let code: Vec<u8> = trr!(machine.account_code(account).and_then(|code| Ok(code.into())), __);
                if code_index + i < code.len() {
                    let val = code[code_index + i];
                    machine.memory.write_raw(memory_index + i.into(), val);
                }
            }
            end_rescuable!(__);
        },

        Opcode::BLOCKHASH => {
            will_pop_push!(machine, 1, 1);

            // TODO: use ExecutionError::RequireBlockhash
            unimplemented!()
        },

        Opcode::COINBASE => {
            will_pop_push!(machine, 0, 1);

            let val = machine.block().coinbase;
            machine.stack.push(val.into()).unwrap();
        },

        Opcode::TIMESTAMP => {
            will_pop_push!(machine, 0, 1);

            let val = machine.block().timestamp;
            machine.stack.push(val.into()).unwrap();
        },

        Opcode::NUMBER => {
            will_pop_push!(machine, 0, 1);

            let val = machine.block().number;
            machine.stack.push(val.into()).unwrap();
        },

        Opcode::DIFFICULTY => {
            will_pop_push!(machine, 0, 1);

            let val = machine.block().difficulty;
            machine.stack.push(val.into()).unwrap();
        },

        Opcode::GASLIMIT => {
            will_pop_push!(machine, 0, 1);

            let val = machine.block().gas_limit;
            machine.stack.push(val.into()).unwrap();
        },

        Opcode::POP => {
            will_pop_push!(machine, 1, 0);

            machine.stack.pop().unwrap();
        },

        Opcode::MLOAD => {
            will_pop_push!(machine, 1, 1);

            begin_rescuable!(machine, &mut Machine<M, S>, __);
            let op1 = machine.stack.pop().unwrap();
            on_rescue!(|machine| {
                machine.stack.push(op1).unwrap();
            }, __);
            let val = trr!(machine.memory.read(op1), __);
            machine.stack.push(val).unwrap();
            end_rescuable!(__);
        },

        Opcode::MSTORE => {
            will_pop_push!(machine, 2, 0);

            let op1 = machine.stack.pop().unwrap(); // Index
            let op2 = machine.stack.pop().unwrap(); // Data
            // u_i update is automatically handled by Memory.
            machine.memory.write(op1, op2);
        },

        Opcode::MSTORE8 => {
            will_pop_push!(machine, 2, 0);

            let op1 = machine.stack.pop().unwrap(); // Index
            let op2 = machine.stack.pop().unwrap(); // Data
            let a: [u8; 32] = op2.into();
            let val = a[31];
            machine.memory.write_raw(op1, val);
        },

        Opcode::SLOAD => {
            will_pop_push!(machine, 1, 1);

            begin_rescuable!(machine, &mut Machine<M, S>, __);
            let op1 = machine.stack.pop().unwrap();
            on_rescue!(|machine| {
                machine.stack.push(op1).unwrap();
            }, __);

            let from = machine.owner();
            let val = trr!(machine.account_storage(from).and_then(|storage| storage.read(op1)), __);
            machine.stack.push(val).unwrap();
        },

        Opcode::SSTORE => {
            will_pop_push!(machine, 2, 0);

            begin_rescuable!(machine, &mut Machine<M, S>, __);
            let op1 = machine.stack.pop().unwrap(); // Index
            let op2 = machine.stack.pop().unwrap(); // Data
            on_rescue!(|machine| {
                machine.stack.push(op2).unwrap();
                machine.stack.push(op1).unwrap();
            }, __);

            let from = machine.owner();
            trr!(machine.account_storage_mut(from).and_then(|storage| storage.write(op1, op2)), __);
            end_rescuable!(__);
        }

        Opcode::JUMP => {
            will_pop_push!(machine, 1, 0);

            begin_rescuable!(machine, &mut Machine<M, S>, __);
            let op1 = machine.stack.pop().unwrap();
            on_rescue!(|machine| {
                machine.stack.push(op1).unwrap();
            }, __);

            if op1 > usize::max_value().into() {
                trr!(Err(ExecutionError::PCTooLarge), __);
            }

            trr!(machine.pc.jump(op1.into()), __);
            end_rescuable!(__);
        },

        Opcode::JUMPI => {
            will_pop_push!(machine, 2, 0);

            begin_rescuable!(machine, &mut Machine<M, S>, __);
            let op1 = machine.stack.pop().unwrap();
            on_rescue!(|machine| {
                machine.stack.push(op1).unwrap();
            }, __);

            if op1 > usize::max_value().into() {
                trr!(Err(ExecutionError::PCTooLarge), __);
            }

            let op2 = machine.stack.pop().unwrap();
            on_rescue!(|machine| {
                machine.stack.push(op2).unwrap();
            }, __);

            if op2 != M256::zero() {
                trr!(machine.pc.jump(op1.into()), __);
            }
            end_rescuable!(__);
        },

        Opcode::PC => {
            will_pop_push!(machine, 0, 1);

            let position = machine.pc.position();
            machine.stack.push((position - 1).into()).unwrap(); // PC increment for opcode is always an u8.
        },

        Opcode::MSIZE => {
            will_pop_push!(machine, 0, 1);

            let active_memory_len = machine.active_memory_len();
            machine.stack.push(M256::from(32u64) * active_memory_len).unwrap();
        },

        Opcode::GAS => {
            will_pop_push!(machine, 0, 1);

            machine.stack.push(after_gas.into()).unwrap();
        },

        Opcode::JUMPDEST => {
            will_pop_push!(machine, 0, 0);
            ()
        }, // This operation has no effect on machine state during execution.

        Opcode::PUSH(v) => {
            will_pop_push!(machine, 0, 1);

            let val = machine.pc.read(v)?; // We don't have any stack to restore, so this ? is okay.
            machine.stack.push(val).unwrap();
        },

        Opcode::DUP(v) => {
            will_pop_push!(machine, v, v+1);

            let val = machine.stack().peek(v - 1).unwrap();
            machine.stack.push(val).unwrap();
        },

        Opcode::SWAP(v) => {
            will_pop_push!(machine, v+1, v+1);

            let val1 = machine.stack().peek(0).unwrap();
            let val2 = machine.stack().peek(v).unwrap();
            machine.stack.set(0, val2).unwrap();
            machine.stack.set(v, val1).unwrap();
        },

        Opcode::LOG(v) => {
            will_pop_push!(machine, v+2, 0);

            begin_rescuable!(machine, &mut Machine<M, S>, __);
            let address = machine.owner();
            let mut data: Vec<u8> = Vec::new();
            let mut start = machine.stack.pop().unwrap();
            let start0 = start;
            let len = machine.stack.pop().unwrap();
            let ender = start + len;
            on_rescue!(|machine| {
                machine.stack.push(len).unwrap();
                machine.stack.push(start0).unwrap();
            }, __);
            if ender < start {
                trr!(Err(ExecutionError::MemoryTooLarge), __);
            }

            while start < ender {
                data.push(trr!(machine.memory().read_raw(start), __));
                start = start + M256::one();
            }
            end_rescuable!(__);

            let mut topics: Vec<M256> = Vec::new();

            for i in 0..v {
                topics.push(machine.stack.pop().unwrap());
            }

            machine.account_log(address, data.as_ref(), topics.as_ref());
        },

        Opcode::CREATE => {
            will_pop_push!(machine, 3, 1);

            unimplemented!()
        },

        Opcode::CALL => {
            will_pop_push!(machine, 7, 1);

            let gas: Gas = machine.stack.pop().unwrap().into();
            let from = machine.owner();
            let to: Address = machine.stack.pop().unwrap().into();
            let value = machine.stack.pop().unwrap().into();
            let memory_in_start = machine.stack.pop().unwrap();
            let memory_in_len = machine.stack.pop().unwrap();
            let memory_out_start = machine.stack.pop().unwrap();
            let memory_out_len = machine.stack.pop().unwrap();

            let ret = call_code(machine, gas, from, to, value,
                                memory_in_start, memory_in_len,
                                memory_out_start, memory_out_len);

            machine.stack.push(ret).unwrap();
        },

        Opcode::CALLCODE => {
            will_pop_push!(machine, 7, 1);

            let gas: Gas = machine.stack.pop().unwrap().into();
            machine.stack.pop().unwrap();
            let from = machine.owner();
            let to = machine.owner();
            let value = machine.stack.pop().unwrap().into();
            let memory_in_start = machine.stack.pop().unwrap();
            let memory_in_len = machine.stack.pop().unwrap();
            let memory_out_start = machine.stack.pop().unwrap();
            let memory_out_len = machine.stack.pop().unwrap();

            let ret = call_code(machine, gas, from, to, value,
                                memory_in_start, memory_in_len,
                                memory_out_start, memory_out_len);

            machine.stack.push(ret).unwrap();
        },

        Opcode::RETURN => {
            will_pop_push!(machine, 2, 0);

            begin_rescuable!(machine, &mut Machine<M, S>, __);
            let mut start = machine.stack.pop().unwrap();
            let start0 = start;
            let len = machine.stack.pop().unwrap();
            let ender = start + len;
            on_rescue!(|machine| {
                machine.stack.push(len).unwrap();
                machine.stack.push(start0).unwrap();
            }, __);
            if ender < start {
                trr!(Err(ExecutionError::MemoryTooLarge), __);
            }
            let mut vec: Vec<u8> = Vec::new();

            while start < ender {
                vec.push(trr!(machine.memory().read_raw(start), __));
                start = start + M256::one();
            }

            machine.return_values = vec;
            machine.pc.stop();
            end_rescuable!(__);
        },

        Opcode::DELEGATECALL => {
            will_pop_push!(machine, 6, 1);

            let gas: Gas = machine.stack.pop().unwrap().into();
            let from = machine.transaction().caller();
            let to: Address = machine.stack.pop().unwrap().into();
            let value = machine.transaction().value();
            let memory_in_start = machine.stack.pop().unwrap();
            let memory_in_len = machine.stack.pop().unwrap();
            let memory_out_start = machine.stack.pop().unwrap();
            let memory_out_len = machine.stack.pop().unwrap();

            let ret = call_code(machine, gas, from, to, value,
                                memory_in_start, memory_in_len,
                                memory_out_start, memory_out_len);

            machine.stack.push(ret).unwrap();
        },

        Opcode::SUICIDE => {
            will_pop_push!(machine, 1, 0);

            machine.stack.pop().unwrap();
            machine.pc.stop();
        },

        Opcode::INVALID => {
            machine.pc.stop();
            return Err(ExecutionError::InvalidOpcode);
        }
    }
    Ok(())
}
