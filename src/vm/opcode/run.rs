use utils::bigint::{M256, MI256, U256, U512};
use utils::gas::Gas;
use utils::address::Address;
use super::Opcode;
use vm::{Machine, Memory, Stack, PC, Result, Error};
use transaction::Transaction;
use blockchain::Block;

use std::ops::{Add, Sub, Not, Mul, Div, Shr, Shl, BitAnd, BitOr, BitXor, Rem};
use crypto::sha3::Sha3;
use crypto::digest::Digest;

fn signed_abs(v: M256) -> M256 {
    let negative: M256 = M256::one() << 256;

    if v >= negative {
        !v + 1.into()
    } else {
        v
    }
}

macro_rules! will_pop_push {
    ( $machine:expr, $pop_size:expr, $push_size:expr ) => ({
        if $machine.stack_mut().size() < $pop_size { return Err(Error::StackUnderflow); }
    })
}

macro_rules! op2 {
    ( $machine:expr, $op:ident ) => ({
        will_pop_push!($machine, 2, 1);

        let op1 = $machine.stack_mut().pop().unwrap();
        let op2 = $machine.stack_mut().pop().unwrap();
        $machine.stack_mut().push(op1.$op(op2));
    })
}

macro_rules! op2_ref {
    ( $machine:expr, $op:ident ) => ({
        will_pop_push!($machine, 2, 1);

        let op1 = $machine.stack_mut().pop().unwrap();
        let op2 = $machine.stack_mut().pop().unwrap();
        $machine.stack_mut().push(op1.$op(&op2).into());
    })
}

impl Opcode {
    pub fn run<M: Machine>(&self, machine: &mut M) -> Result<()> {
        let opcode = self.clone();

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

                let mut op1 = machine.stack_mut().pop().unwrap();
                let op2 = machine.stack_mut().pop().unwrap();

                let mut r: [u8; 32] = [0u8; 32];
                let mut sha3 = Sha3::keccak256();

                while op1 != op2 - 1.into() {
                    let val = machine.memory_mut().read(op1);
                    let a: [u8; 32] = val.into();
                    sha3.input(a.as_ref());
                    op1 = op1 + 1.into();
                }
                sha3.result(&mut r);
                machine.stack_mut().push(M256::from(r.as_ref()))
            },

            Opcode::ADDRESS => {
                will_pop_push!(machine, 0, 1);

                let address = machine.transaction().callee();
                machine.stack_mut().push(address.into());
            },

            Opcode::BALANCE => {
                will_pop_push!(machine, 1, 1);

                let address: Option<Address> = machine.stack_mut().pop().unwrap().into();
                let balance = address.map_or(None, |address| {
                    Some(machine.block().balance(address))
                }).map_or(M256::zero(), |balance| balance.into());
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

                let start_index: usize = machine.stack_mut().pop().unwrap().into();
                let load = M256::from(&machine.transaction().data()
                                      .unwrap()[start_index..start_index+32]);
                machine.stack_mut().push(load);
            },

            Opcode::CALLDATASIZE => {
                will_pop_push!(machine, 0, 1);

                let len = machine.transaction().data().map_or(0, |s| s.len());
                machine.stack_mut().push(len.into());
            },

            Opcode::CALLDATACOPY => {
                will_pop_push!(machine, 3, 0);

                let memory_index = machine.stack_mut().pop().unwrap();
                let data_index: usize = machine.stack_mut().pop().unwrap().into();
                let len: usize = machine.stack_mut().pop().unwrap().into();

                for i in 0..len {
                    let val = machine.transaction().data().unwrap()[data_index + i];
                    machine.memory_mut().write_raw(memory_index + i.into(), val);
                }
            },

            Opcode::CODESIZE => {
                will_pop_push!(machine, 0, 1);

                let len = machine.pc().code().len();
                machine.stack_mut().push(len.into());
            },

            Opcode::CODECOPY => {
                will_pop_push!(machine, 1, 1);

                let memory_index = machine.stack_mut().pop().unwrap();
                let code_index: usize = machine.stack_mut().pop().unwrap().into();
                let len: usize = machine.stack_mut().pop().unwrap().into();

                for i in 0..len {
                    let val = machine.pc().code()[code_index + i];
                    machine.memory_mut().write_raw(memory_index + i.into(), val);
                }
            },

            Opcode::GASPRICE => {
                will_pop_push!(machine, 0, 1);

                let price: M256 = machine.transaction().gas_price().into();
                machine.stack_mut().push(price);
            },

            Opcode::EXTCODESIZE => {
                will_pop_push!(machine, 1, 1);

                let account: Option<Address> = machine.stack_mut().pop().unwrap().into();
                let account = account.unwrap();
                let len = machine.block().account_code(account).len();
                machine.stack_mut().push(len.into());
            },

            Opcode::EXTCODECOPY => {
                will_pop_push!(machine, 4, 0);

                let account: Option<Address> = machine.stack_mut().pop().unwrap().into();
                let account = account.unwrap();
                let memory_index = machine.stack_mut().pop().unwrap();
                let code_index: usize = machine.stack_mut().pop().unwrap().into();
                let len: usize = machine.stack_mut().pop().unwrap().into();

                for i in 0..len {
                    let val = machine.block().account_code(account)[code_index + i];
                    machine.memory_mut().write_raw(memory_index + i.into(), val);
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

                let op1 = machine.stack_mut().pop().unwrap();
                let val = machine.memory_mut().read(op1);
                // u_i update is automatically handled by Memory.
                machine.stack_mut().push(val);
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

                let op1_u: u64 = machine.stack_mut().pop().unwrap().into();
                machine.pc_mut().jump(op1_u as usize);
            },

            Opcode::JUMPI => {
                will_pop_push!(machine, 2, 0);

                let op1_u: u64 = machine.stack_mut().pop().unwrap().into();
                let op2 = machine.stack_mut().pop().unwrap();

                if op2 != 0.into() {
                    machine.pc_mut().jump(op1_u as usize);
                }
            },

            Opcode::PC => {
                will_pop_push!(machine, 0, 1);

                let position = machine.pc().position();
                machine.stack_mut().push((position - 1).into()); // PC increment for opcode is always an u8.
            },

            Opcode::MSIZE => {
                will_pop_push!(machine, 0, 1);

                let active_len = machine.memory().active_len();
                machine.stack_mut().push(active_len.into());
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

                let val = machine.pc_mut().read(v);
                machine.stack_mut().push(val);
            },

            Opcode::DUP(v) => {
                will_pop_push!(machine, v, v+1);

                let val = machine.stack().peek(v - 1)?;
                machine.stack_mut().push(val);
            },

            Opcode::SWAP(v) => {
                will_pop_push!(machine, v+1, v+1);

                let val1 = machine.stack().peek(0)?;
                let val2 = machine.stack().peek(v)?;
                machine.stack_mut().set(0, val2).unwrap();
                machine.stack_mut().set(v, val1).unwrap();
            },

            Opcode::LOG(v) => {
                will_pop_push!(machine, v+2, 0);

                let address = machine.transaction().callee();
                let mut data: Vec<u8> = Vec::new();
                let start = machine.stack_mut().pop().unwrap();
                let len: usize = machine.stack_mut().pop().unwrap().into();

                for i in 0..len {
                    data.push(machine.memory_mut().read_raw(start + i.into()));
                }

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
                let to: Option<Address> = machine.stack_mut().pop().unwrap().into();
                let to = to.unwrap();
                let value = machine.stack_mut().pop().unwrap().into();
                let memory_in_start = machine.stack_mut().pop().unwrap();
                let memory_in_len = machine.stack_mut().pop().unwrap();
                let memory_out_start = machine.stack_mut().pop().unwrap();
                let memory_out_len = machine.stack_mut().pop().unwrap();

                machine.fork(gas, from, to, value, memory_in_start, memory_in_len,
                             memory_out_start, memory_out_len, |machine| {
                                 machine.fire();
                             });

                machine.stack_mut().push(M256::zero());
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

                machine.fork(gas, from, to, value, memory_in_start, memory_in_len,
                             memory_out_start, memory_out_len, |machine| {
                                 machine.fire();
                             });

                machine.stack_mut().push(M256::zero());
            },

            Opcode::RETURN => {
                will_pop_push!(machine, 2, 0);

                let start = machine.stack_mut().pop().unwrap();
                let len: usize = machine.stack_mut().pop().unwrap().into();
                let mut vec: Vec<u8> = Vec::new();

                for i in 0..len {
                    vec.push(machine.memory_mut().read_raw(start + i.into()));
                }

                machine.set_return_values(vec.as_ref());
                machine.pc_mut().stop();
            },

            Opcode::DELEGATECALL => {
                will_pop_push!(machine, 6, 1);

                let gas: Gas = machine.stack_mut().pop().unwrap().into();
                let from = machine.transaction().sender();
                let to: Option<Address> = machine.stack_mut().pop().unwrap().into();
                let to = to.unwrap();
                let value = machine.transaction().value();
                let memory_in_start = machine.stack_mut().pop().unwrap();
                let memory_in_len = machine.stack_mut().pop().unwrap();
                let memory_out_start = machine.stack_mut().pop().unwrap();
                let memory_out_len = machine.stack_mut().pop().unwrap();

                machine.fork(gas, from, to, value, memory_in_start, memory_in_len,
                             memory_out_start, memory_out_len, |machine| {
                                 machine.fire();
                             });

                machine.stack_mut().push(M256::zero());
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
