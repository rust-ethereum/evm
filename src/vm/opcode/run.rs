use super::Opcode;
use vm::{Machine, Memory, Stack, PC};

// TODO: deal with gas limit and other Ethereum-specific things.

fn push<S: Stack>(opcode: Opcode, pc: &mut PC, stack: &mut S) {
    let code_u8: u8 = opcode.into();
    let count = code_u8 - 0x5f;
    let val = pc.read(count as usize);
    stack.push(val);
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

            // TODO: implement omitted opcodes.

            Opcode::PUSH1 | Opcode::PUSH2 | Opcode::PUSH3 |
            Opcode::PUSH4 | Opcode::PUSH5 | Opcode::PUSH6 |
            Opcode::PUSH7 | Opcode::PUSH8 | Opcode::PUSH9 |
            Opcode::PUSH10 | Opcode::PUSH11 | Opcode::PUSH12 |
            Opcode::PUSH13 | Opcode::PUSH14 | Opcode::PUSH15 |
            Opcode::PUSH16 | Opcode::PUSH17 | Opcode::PUSH18 |
            Opcode::PUSH19 | Opcode::PUSH20 | Opcode::PUSH21 |
            Opcode::PUSH22 | Opcode::PUSH23 | Opcode::PUSH24 |
            Opcode::PUSH25 | Opcode::PUSH26 | Opcode::PUSH27 |
            Opcode::PUSH28 | Opcode::PUSH29 | Opcode::PUSH30 |
            Opcode::PUSH31 | Opcode::PUSH32
                => push(opcode, pc, stack),

            _ => {
                unimplemented!();
            }
        }
    }
}
