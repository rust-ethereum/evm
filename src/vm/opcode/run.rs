use super::Opcode;
use vm::{Machine, Memory, Stack, PC};

// TODO: deal with gas limit and other Ethereum-specific things.

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
