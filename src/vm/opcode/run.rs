use super::Opcode;
use utils::u256::U256;
use vm::{Machine, Memory, Stack, PC};

// TODO: deal with gas limit and other Ethereum-specific things.

fn signed_abs(v: U256) -> U256 {
    let negative: U256 = U256::one() << 256;

    if v >= negative {
        !v + 1.into()
    } else {
        v
    }
}

impl Opcode {
    pub fn run<M: Memory, S: Stack>(&self, machine: &mut Machine<M, S>) {
        let opcode = self.clone();

        match opcode {
            Opcode::STOP => {
                machine.pc.stop();
            },

            Opcode::ADD => {
                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();
                machine.stack.push(op1 + op2);
            },

            Opcode::MUL => {
                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();
                machine.stack.push(op1 * op2);
            },

            Opcode::SUB => {
                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();
                machine.stack.push(op1 - op2);
            },

            Opcode::DIV => {
                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();
                if op2 == 0.into() {
                    machine.stack.push(0.into());
                } else {
                    machine.stack.push(op1 / op2);
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

                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();
                if op2 == 0.into() {
                    machine.stack.push(0.into());
                } else if op1 == negative && op2 == max {
                    machine.stack.push(negative);
                } else {
                    let aop1 = signed_abs(op1);
                    let aop2 = signed_abs(op2);
                    let r = op1 / op2;

                    if (op1 < negative && op2 < negative) || (op1 >= negative && op2 >= negative) {
                        machine.stack.push(r);
                    } else {
                        let sr = !r + 1.into();
                        machine.stack.push(sr);
                    }
                }
            },

            Opcode::MOD => {
                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();

                if op2 == 0.into() {
                    machine.stack.push(0.into());
                } else {
                    machine.stack.push(op1 - (op1 / op2) * op2);
                }
            },

            Opcode::SMOD => {
                let negative: U256 = U256::one() << 256;

                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();

                if op2 == 0.into() {
                    machine.stack.push(0.into());
                } else {
                    let aop1 = signed_abs(op1);
                    let aop2 = signed_abs(op2);
                    let r = aop1 - (aop1 / aop2) * aop2;
                    if op1 < negative && op2 < negative {
                        machine.stack.push(r);
                    } else if op1 >= negative && op2 < negative {
                        machine.stack.push(!(op1 + r) + 1.into());
                    } else if op1 < negative && op2 >= negative {
                        machine.stack.push(op1 + r);
                    } else if op1 >= negative && op2 >= negative {
                        machine.stack.push(!r + 1.into());
                    }
                }
            },

            Opcode::ADDMOD => {
                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();
                let op3 = machine.stack.pop();

                if op3 == 0.into() {
                    machine.stack.push(0.into());
                } else {
                    // TODO: Handle the case where op1 + op2 > 2^256
                    let v = op1 + op2;
                    machine.stack.push(v - (v / op3) * op3);
                }
            },

            Opcode::MULMOD => {
                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();
                let op3 = machine.stack.pop();

                if op3 == 0.into() {
                    machine.stack.push(0.into());
                } else {
                    // TODO: Handle the case where op1 * op2 > 2^256
                    let v = op1 * op2;
                    machine.stack.push(v - (v / op3) * op3);
                }
            },

            Opcode::EXP => {
                let op1 = machine.stack.pop();
                let mut op2 = machine.stack.pop();
                let mut r: U256 = 1.into();

                while op2 != 0.into() {
                    r = r * op1;
                    op2 = op2 - 1.into();
                }
                machine.stack.push(r);
            },

            Opcode::SIGNEXTEND => {
                // TODO: Check this confines with the yello paper

                let mut op1 = machine.stack.pop();
                let mut op2 = machine.stack.pop();

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
                    machine.stack.push(op1);
                } else {
                    machine.stack.push(op1);
                }
            },

            Opcode::LT => {
                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();
                if op1 < op2 {
                    machine.stack.push(1.into());
                } else {
                    machine.stack.push(0.into());
                }
            },

            Opcode::GT => {
                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();
                if op1 > op2 {
                    machine.stack.push(1.into());
                } else {
                    machine.stack.push(0.into());
                }
            },

            Opcode::SLT => {
                let negative = U256::one() << 256;

                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();

                if op1 < negative && op2 < negative {
                    if op1 < op2 {
                        machine.stack.push(1.into());
                    } else {
                        machine.stack.push(0.into());
                    }
                } else if op2 >= negative && op2 >= negative {
                    if op1 < op2 {
                        machine.stack.push(0.into());
                    } else {
                        machine.stack.push(1.into());
                    }
                } else if op1 < negative && op2 >= negative {
                    machine.stack.push(0.into());
                } else {
                    machine.stack.push(1.into());
                }
            },

            Opcode::SGT => {
                let negative = U256::one() << 256;

                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();

                if op1 < negative && op2 < negative {
                    if op1 < op2 {
                        machine.stack.push(0.into());
                    } else {
                        machine.stack.push(1.into());
                    }
                } else if op2 >= negative && op2 >= negative {
                    if op1 < op2 {
                        machine.stack.push(1.into());
                    } else {
                        machine.stack.push(0.into());
                    }
                } else if op1 < negative && op2 >= negative {
                    machine.stack.push(1.into());
                } else {
                    machine.stack.push(0.into());
                }
            },

            Opcode::EQ => {
                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();

                if op1 == op2 {
                    machine.stack.push(1.into());
                } else {
                    machine.stack.push(0.into());
                }
            },

            Opcode::ISZERO => {
                let op1 = machine.stack.pop();

                if op1 == 0.into() {
                    machine.stack.push(1.into());
                } else {
                    machine.stack.push(0.into());
                }
            },

            Opcode::AND => {
                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();

                machine.stack.push(op1 & op2);
            },

            Opcode::OR => {
                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();

                machine.stack.push(op1 | op2);
            },

            Opcode::XOR => {
                let op1 = machine.stack.pop();
                let op2 = machine.stack.pop();

                machine.stack.push(op1 ^ op2);
            },

            Opcode::NOT => {
                let op1 = machine.stack.pop();

                machine.stack.push(!op1);
            },

            Opcode::BYTE => {
                let op1 = machine.stack.pop();
                let op2: usize = machine.stack.pop().into(); // 256 / 8
                let mark: U256 = 0xff.into();

                if op2 >= 256 / 8 {
                    machine.stack.push(0.into());
                } else {
                    machine.stack.push((op1 >> (op2 * 8)) & mark);
                }
            },

            // TODO: implement opcode 0x20 to 0x4f

            Opcode::POP => {
                machine.stack.pop();
            },

            Opcode::MLOAD => {
                let op1 = machine.stack.pop();
                // u_i update is automatically handled by Memory.
                machine.stack.push(machine.memory.read(op1));
            },

            Opcode::MSTORE => {
                let op1 = machine.stack.pop(); // Index
                let op2 = machine.stack.pop(); // Data
                // u_i update is automatically handled by Memory.
                machine.memory.write(op1, op2);
            },

            // TODO: implement storage related opcode SLOAD, SSTORE

            Opcode::JUMP => {
                let op1_u: u64 = machine.stack.pop().into();
                machine.pc.position = op1_u as usize;
            },

            Opcode::JUMPI => {
                let op1_u: u64 = machine.stack.pop().into();
                let op2 = machine.stack.pop();

                if op2 != 0.into() {
                    machine.pc.position = op1_u as usize;
                }
            },

            Opcode::PC => {
                machine.stack.push((machine.pc.position - 1).into()); // PC increment for opcode is always an u8.
            },

            Opcode::MSIZE => {
                machine.stack.push(machine.memory.active_len().into());
            },

            Opcode::GAS => {
                let gas: U256 = machine.available_gas().into();
                machine.stack.push(gas);
            },

            Opcode::JUMPDEST => (), // This operation has no effect on machine state during execution.

            Opcode::PUSH(v) => {
                let val = machine.pc.read(v);
                machine.stack.push(val);
            },

            Opcode::DUP(v) => {
                let val = machine.stack.peek(v - 1);
                machine.stack.push(val);
            },

            Opcode::SWAP(v) => {
                let val1 = machine.stack.peek(0);
                let val2 = machine.stack.peek(v);
                machine.stack.set(0, val2);
                machine.stack.set(v, val1);
            },

            // TODO: Implement log entries LOG0, LOG1, LOG2, LOG3, LOG4

            // TODO: Implement system operations 0xf0 to 0xff

            _ => {
                unimplemented!();
            }
        }
    }
}
