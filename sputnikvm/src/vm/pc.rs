use utils::bigint::M256;
use utils::opcode::Opcode;
use std::cmp::min;
use super::errors::PCError;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Push(Opcode, M256),
    NotPush(Opcode),
}

pub struct PC {
    position: usize,
    code: Vec<u8>,
    valids: Vec<bool>,
}

impl Default for PC {
    fn default() -> PC {
        PC {
            position: 0,
            code: Vec::new(),
            valids: Vec::new(),
        }
    }
}

impl PC {
    pub fn new(code: &[u8]) -> Self {
        let code: Vec<u8> = code.into();
        let mut valids: Vec<bool> = Vec::with_capacity(code.len());
        valids.resize(code.len(), false);

        let mut i = 0;
        while i < code.len() {
            let opcode: Opcode = code[i].into();
            match opcode {
                Opcode::JUMPDEST => {
                    valids[i] = true;
                    i = i + 1;
                },
                Opcode::PUSH(v) => {
                    i = i + v + 1;
                },
                _ => {
                    i = i + 1;
                }
            }
        }

        PC {
            position: 0,
            code: code,
            valids: valids,
        }
    }

    fn read_bytes(&self, from_position: usize, byte_count: usize) -> Result<M256, PCError> {
        if from_position > self.position {
            return Err(PCError::Overflow);
        }
        let position = from_position;
        if position.checked_add(byte_count).is_none() {
            return Err(PCError::IndexNotSupported);
        }
        let max = min(position + byte_count, self.code.len());
        Ok(M256::from(&self.code[position..max]))
    }

    pub fn jump(&mut self, position: usize) -> Result<(), PCError> {
        if position >= self.code.len() {
            return Err(PCError::Overflow);
        }

        if !self.valids[position] {
            return Err(PCError::BadJumpDest);
        }

        self.position = position;
        Ok(())
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn peek(&self) -> Result<Instruction, PCError> {
        let position = self.position;
        if position >= self.code.len() {
            return Err(PCError::Overflow);
        }
        let opcode: Opcode = self.code[position].into();
        match opcode {
            Opcode::PUSH(v) => {
                let param = self.read_bytes(position + 1, v)?;
                Ok(Instruction::Push(opcode, param))
            },
            _ => {
                Ok(Instruction::NotPush(opcode))
            }
        }
    }

    pub fn read(&mut self) -> Result<Instruction, PCError> {
        let result = self.peek()?;
        match result {
            Instruction::NotPush(_) => {
                self.position = self.position + 1;
            },
            Instruction::Push(Opcode::PUSH(v), _) => {
                self.position = self.position + v + 1;
            },
            _ => panic!(),
        }
        Ok(result)
    }
}
