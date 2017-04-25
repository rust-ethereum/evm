use utils::bigint::M256;
use std::cmp::{min};
use super::{Result, Error};
use super::opcode::Opcode;

pub trait PC {
    fn peek_opcode(&self) -> Result<Opcode>;
    fn read_opcode(&mut self) -> Result<Opcode>;
    fn stop(&mut self);
    fn stopped(&self) -> bool;
    fn read(&mut self, byte_count: usize) -> Result<M256>;
    fn position(&self) -> usize;
    fn jump(&mut self, position: usize);
    fn code(&self) -> &[u8];
}

pub struct VectorPC {
    position: usize,
    code: Vec<u8>,
    stopped: bool
}

impl VectorPC {
    pub fn new(code: &[u8]) -> Self {
        VectorPC {
            position: 0,
            code: code.into(),
            stopped: false,
        }
    }
}

impl PC for VectorPC {
    fn code(&self) -> &[u8] {
        self.code.as_ref()
    }

    fn jump(&mut self, position: usize) {
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
        if position.checked_add(byte_count).is_none() || position.checked_add(byte_count).unwrap() >= self.code.len() {
            return Err(Error::PCOverflow);
        }
        self.position += byte_count;
        let max = min(position + byte_count, self.code.len());
        Ok(M256::from(&self.code[position..max]))
    }
}
