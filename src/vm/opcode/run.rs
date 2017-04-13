use utils::u256::U256;
use utils::gas::Gas;
use utils::address::Address;
use super::Opcode;
use vm::{Machine, Memory, Stack, PC};
use account::Storage;
use transaction::Transaction;
use blockchain::{Block, Blockchain};

use std::ops::{Add, Sub, Not, Mul, Div, Shr, Shl, BitAnd, BitOr, BitXor};
use crypto::sha3::Sha3;
use crypto::digest::Digest;

fn signed_abs(v: U256) -> U256 {
    let negative: U256 = U256::one() << 256;

    if v >= negative {
        !v + 1.into()
    } else {
        v
    }
}

macro_rules! op2 {
    ( $machine:expr, $op:ident ) => ({
        let op1 = $machine.stack_mut().pop();
        let op2 = $machine.stack_mut().pop();
        $machine.stack_mut().push(op1.$op(op2));
    })
}

macro_rules! op2_ref {
    ( $machine:expr, $op:ident ) => ({
        let op1 = $machine.stack_mut().pop();
        let op2 = $machine.stack_mut().pop();
        $machine.stack_mut().push(op1.$op(&op2).into());
    })
}

impl Opcode {
    pub fn run<M: Machine>(&self, machine: &mut M) {
        let opcode = self.clone();

        match opcode {
            Opcode::STOP => {
                machine.pc_mut().stop();
            },

            Opcode::ADD => op2!(machine, add),
            Opcode::MUL => op2!(machine, mul),
            Opcode::SUB => op2!(machine, sub),

            Opcode::DIV => {
                let op1 = machine.stack_mut().pop();
                let op2 = machine.stack_mut().pop();
                if op2 == 0.into() {
                    machine.stack_mut().push(0.into());
                } else {
                    machine.stack_mut().push(op1 / op2);
                }
            },

            Opcode::SDIV => {
                // This is signed division. So the U256 would need to
                // be treated as signed two's complement. We currently
                // convert it to both positive, and then deal with the
                // sign afterwards...

                let negative: U256 = U256::one() << 256;
                // This value is also -2^255 in two's complement.
                let max: U256 = U256::max_value();
                // This value is also -1 in two's complement.

                let op1 = machine.stack_mut().pop();
                let op2 = machine.stack_mut().pop();
                if op2 == 0.into() {
                    machine.stack_mut().push(0.into());
                } else if op1 == negative && op2 == max {
                    machine.stack_mut().push(negative);
                } else {
                    let aop1 = signed_abs(op1);
                    let aop2 = signed_abs(op2);
                    let r = op1 / op2;

                    if (op1 < negative && op2 < negative) || (op1 >= negative && op2 >= negative) {
                        machine.stack_mut().push(r);
                    } else {
                        let sr = !r + 1.into();
                        machine.stack_mut().push(sr);
                    }
                }
            },

            Opcode::MOD => {
                let op1 = machine.stack_mut().pop();
                let op2 = machine.stack_mut().pop();

                if op2 == 0.into() {
                    machine.stack_mut().push(0.into());
                } else {
                    machine.stack_mut().push(op1 - (op1 / op2) * op2);
                }
            },

            Opcode::SMOD => {
                let negative: U256 = U256::one() << 256;

                let op1 = machine.stack_mut().pop();
                let op2 = machine.stack_mut().pop();

                if op2 == 0.into() {
                    machine.stack_mut().push(0.into());
                } else {
                    let aop1 = signed_abs(op1);
                    let aop2 = signed_abs(op2);
                    let r = aop1 - (aop1 / aop2) * aop2;
                    if op1 < negative && op2 < negative {
                        machine.stack_mut().push(r);
                    } else if op1 >= negative && op2 < negative {
                        machine.stack_mut().push(!(op1 + r) + 1.into());
                    } else if op1 < negative && op2 >= negative {
                        machine.stack_mut().push(op1 + r);
                    } else if op1 >= negative && op2 >= negative {
                        machine.stack_mut().push(!r + 1.into());
                    }
                }
            },

            Opcode::ADDMOD => {
                let op1 = machine.stack_mut().pop();
                let op2 = machine.stack_mut().pop();
                let op3 = machine.stack_mut().pop();

                if op3 == 0.into() {
                    machine.stack_mut().push(0.into());
                } else {
                    // TODO: Handle the case where op1 + op2 > 2^256
                    let v = op1 + op2;
                    machine.stack_mut().push(v - (v / op3) * op3);
                }
            },

            Opcode::MULMOD => {
                let op1 = machine.stack_mut().pop();
                let op2 = machine.stack_mut().pop();
                let op3 = machine.stack_mut().pop();

                if op3 == 0.into() {
                    machine.stack_mut().push(0.into());
                } else {
                    // TODO: Handle the case where op1 * op2 > 2^256
                    let v = op1 * op2;
                    machine.stack_mut().push(v - (v / op3) * op3);
                }
            },

            Opcode::EXP => {
                let op1 = machine.stack_mut().pop();
                let mut op2 = machine.stack_mut().pop();
                let mut r: U256 = 1.into();

                while op2 != 0.into() {
                    r = r * op1;
                    op2 = op2 - 1.into();
                }
                machine.stack_mut().push(r);
            },

            Opcode::SIGNEXTEND => {
                // TODO: Check this confines with the yello paper

                let mut op1 = machine.stack_mut().pop();
                let mut op2 = machine.stack_mut().pop();

                let mut negative: U256 = 1.into();
                let mut s = 0;
                while op2 != 0.into() {
                    negative = U256::one() << s;
                    s = s + 1;
                    op2 = op2 - 1.into();
                }

                if op1 >= negative {
                    while s <= 256 {
                        op1 = op1 + (U256::one() << s);
                        s = s + 1;
                    }
                    machine.stack_mut().push(op1);
                } else {
                    machine.stack_mut().push(op1);
                }
            },

            Opcode::LT => op2_ref!(machine, lt),
            Opcode::GT => op2_ref!(machine, gt),

            Opcode::SLT => {
                let negative = U256::one() << 256;

                let op1 = machine.stack_mut().pop();
                let op2 = machine.stack_mut().pop();

                if op1 < negative && op2 < negative {
                    if op1 < op2 {
                        machine.stack_mut().push(1.into());
                    } else {
                        machine.stack_mut().push(0.into());
                    }
                } else if op2 >= negative && op2 >= negative {
                    if op1 < op2 {
                        machine.stack_mut().push(0.into());
                    } else {
                        machine.stack_mut().push(1.into());
                    }
                } else if op1 < negative && op2 >= negative {
                    machine.stack_mut().push(0.into());
                } else {
                    machine.stack_mut().push(1.into());
                }
            },

            Opcode::SGT => {
                let negative = U256::one() << 256;

                let op1 = machine.stack_mut().pop();
                let op2 = machine.stack_mut().pop();

                if op1 < negative && op2 < negative {
                    if op1 < op2 {
                        machine.stack_mut().push(0.into());
                    } else {
                        machine.stack_mut().push(1.into());
                    }
                } else if op2 >= negative && op2 >= negative {
                    if op1 < op2 {
                        machine.stack_mut().push(1.into());
                    } else {
                        machine.stack_mut().push(0.into());
                    }
                } else if op1 < negative && op2 >= negative {
                    machine.stack_mut().push(1.into());
                } else {
                    machine.stack_mut().push(0.into());
                }
            },

            Opcode::EQ => op2_ref!(machine, eq),

            Opcode::ISZERO => {
                let op1 = machine.stack_mut().pop();

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
                let op1 = machine.stack_mut().pop();

                machine.stack_mut().push(!op1);
            },

            Opcode::BYTE => {
                let op1 = machine.stack_mut().pop();
                let op2: usize = machine.stack_mut().pop().into(); // 256 / 8
                let mark: U256 = 0xff.into();

                if op2 >= 256 / 8 {
                    machine.stack_mut().push(0.into());
                } else {
                    machine.stack_mut().push((op1 >> (op2 * 8)) & mark);
                }
            },

            Opcode::SHA3 => {
                let mut op1 = machine.stack_mut().pop();
                let op2 = machine.stack_mut().pop();

                let mut r: [u8; 32] = [0u8; 32];
                let mut sha3 = Sha3::keccak256();

                while op1 != op2 - 1.into() {
                    let val = machine.memory_mut().read(op1);
                    sha3.input(val.as_ref());
                    op1 = op1 + 1.into();
                }
                sha3.result(&mut r);
                machine.stack_mut().push(U256::from(r.as_ref()))
            },

            Opcode::ADDRESS => {
                let address = machine.transaction().callee();
                machine.stack_mut().push(address.into());
            },

            Opcode::BALANCE => {
                let address: Option<Address> = machine.stack_mut().pop().into();
                let balance = address.map_or(None, |address| {
                    machine.block().balance(address)
                }).map_or(U256::zero(), |balance| balance);
                machine.stack_mut().push(balance);
            },

            // TODO: implement opcode 0x21 to 0x4f

            Opcode::POP => {
                machine.stack_mut().pop();
            },

            Opcode::MLOAD => {
                let op1 = machine.stack_mut().pop();
                let val = machine.memory_mut().read(op1);
                // u_i update is automatically handled by Memory.
                machine.stack_mut().push(val);
            },

            Opcode::MSTORE => {
                let op1 = machine.stack_mut().pop(); // Index
                let op2 = machine.stack_mut().pop(); // Data
                // u_i update is automatically handled by Memory.
                machine.memory_mut().write(op1, op2);
            },

            Opcode::SLOAD => {
                let op1 = machine.stack_mut().pop();
                let val = machine.storage_mut().read(op1);
                machine.stack_mut().push(val);
            },

            Opcode::SSTORE => {
                let op1 = machine.stack_mut().pop(); // Index
                let op2 = machine.stack_mut().pop(); // Data
                machine.storage_mut().write(op1, op2);
            }

            Opcode::JUMP => {
                let op1_u: u64 = machine.stack_mut().pop().into();
                machine.pc_mut().jump(op1_u as usize);
            },

            Opcode::JUMPI => {
                let op1_u: u64 = machine.stack_mut().pop().into();
                let op2 = machine.stack_mut().pop();

                if op2 != 0.into() {
                    machine.pc_mut().jump(op1_u as usize);
                }
            },

            Opcode::PC => {
                let position = machine.pc().position();
                machine.stack_mut().push((position - 1).into()); // PC increment for opcode is always an u8.
            },

            Opcode::MSIZE => {
                let active_len = machine.memory().active_len();
                machine.stack_mut().push(active_len.into());
            },

            Opcode::GAS => {
                let gas: U256 = machine.transaction().gas_limit().into();
                machine.stack_mut().push(gas);
            },

            Opcode::JUMPDEST => (), // This operation has no effect on machine state during execution.

            Opcode::PUSH(v) => {
                let val = machine.pc_mut().read(v);
                machine.stack_mut().push(val);
            },

            Opcode::DUP(v) => {
                let val = machine.stack().peek(v - 1);
                machine.stack_mut().push(val);
            },

            Opcode::SWAP(v) => {
                let val1 = machine.stack().peek(0);
                let val2 = machine.stack().peek(v);
                machine.stack_mut().set(0, val2);
                machine.stack_mut().set(v, val1);
            },

            // TODO: Implement log entries LOG0, LOG1, LOG2, LOG3, LOG4

            // TODO: Implement system operations 0xf0 to 0xff

            _ => {
                unimplemented!();
            }
        }
    }
}
