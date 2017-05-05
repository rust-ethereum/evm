use utils::bigint::M256;
use std::cmp::{min};
use super::{Result, Error};
use super::opcode::Opcode;

pub struct PC {
    position: usize,
    code: Vec<u8>,
    valids: Vec<bool>,
    stopped: bool
}

impl Default for PC {
    fn default() -> PC {
        PC {
            position: 0,
            code: Vec::new(),
            valids: Vec::new(),
            stopped: true,
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

        VectorPC {
            position: 0,
            code: code,
            valids: valids,
            stopped: false,
        }
    }

    fn code(&self) -> &[u8] {
        self.code.as_ref()
    }

    fn jump(&mut self, position: usize) -> Result<()> {
        if position >= self.code.len() {
            return Err(Error::PCOverflow);
        }

        if !self.valids[position] {
            return Err(Error::PCBadJumpDest);
        }

        self.position = position;
        Ok(())
    }

    fn jump_unchecked(&mut self, position: usize) {
        self.position = position;
    }

    fn position(&self) -> usize {
        self.position
    }

    fn peek_opcode(&self) -> Result<Opcode> {
        let position = self.position;
        if position >= self.code.len() {
            return Err(Error::PCOverflow);
        }
        let opcode: Opcode = self.code[position].into();
        Ok(opcode)
    }

    fn read_opcode(&mut self) -> Result<Opcode> {
        let position = self.position;
        if position.checked_add(1).is_none() {
            return Err(Error::PCTooLarge);
        }
        if position >= self.code.len() {
            return Err(Error::PCOverflow);
        }
        let opcode: Opcode = self.code[position].into();
        self.position += 1;
        Ok(opcode)
    }

    fn stop(&mut self) {
        self.stopped = true;
    }

    fn stopped(&self) -> bool {
        self.stopped || self.position >= self.code.len()
    }

    fn read(&mut self, byte_count: usize) -> Result<M256> {
        let position = self.position;
        if position.checked_add(byte_count).is_none() {
            return Err(Error::PCTooLarge);
        }
        self.position += byte_count;
        let max = min(position + byte_count, self.code.len());
        Ok(M256::from(&self.code[position..max]))
    }
}
