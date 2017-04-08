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
        let pc = &mut machine.pc;
        let memory = &mut machine.memory;
        let stack = &mut machine.stack;
        let opcode = self.clone();

        match opcode {
            Opcode::STOP => {
                pc.stop();
            },

            Opcode::ADD => {
                let op1 = stack.pop();
                let op2 = stack.pop();
                stack.push(op1 + op2);
            },

            Opcode::MUL => {
                let op1 = stack.pop();
                let op2 = stack.pop();
                stack.push(op1 * op2);
            },

            Opcode::SUB => {
                let op1 = stack.pop();
                let op2 = stack.pop();
                stack.push(op1 - op2);
            },

            Opcode::DIV => {
                let op1 = stack.pop();
                let op2 = stack.pop();
                if op2 == 0.into() {
                    stack.push(0.into());
                } else {
                    stack.push(op1 / op2);
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

                let op1 = stack.pop();
                let op2 = stack.pop();
                if op2 == 0.into() {
                    stack.push(0.into());
                } else if op1 == negative && op2 == max {
                    stack.push(negative);
                } else {
                    let aop1 = signed_abs(op1);
                    let aop2 = signed_abs(op2);
                    let r = op1 / op2;

                    if (op1 < negative && op2 < negative) || (op1 >= negative && op2 >= negative) {
                        stack.push(r);
                    } else {
                        let sr = !r + 1.into();
                        stack.push(sr);
                    }
                }
            },

            Opcode::GT => {
                let op1 = stack.pop();
                let op2 = stack.pop();
                if op1 > op2 {
                    stack.push(op1);
                } else {
                    stack.push(op2);
                }
            }

            Opcode::LT => {
                let op1 = stack.pop();
                let op2 = stack.pop();
                if op1 < op2 {
                    stack.push(op1);
                } else {
                    stack.push(op2);
                }
            }

            // TODO: implement omitted opcodes.

            Opcode::PUSH(v) => {
                let val = pc.read(v);
                stack.push(val);
            },

            _ => {
                unimplemented!();
            }
        }
    }
}
