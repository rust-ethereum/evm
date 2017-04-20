use utils::bigint::M256;
use std::cmp::{min};
use super::opcode::Opcode;

pub trait PC {
    fn peek_opcode(&self) -> Opcode;
    fn read_opcode(&mut self) -> Opcode;
    fn stop(&mut self);
    fn stopped(&self) -> bool;
    fn read(&mut self, byte_count: usize) -> M256;
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

    fn peek_opcode(&self) -> Opcode {
        let position = self.position;
        let opcode: Opcode = self.code[position].into();
        opcode
    }

    fn read_opcode(&mut self) -> Opcode {
        let position = self.position;
        let opcode: Opcode = self.code[position].into();
        self.position += 1;
        opcode
    }

    fn stop(&mut self) {
        self.stopped = true;
    }

    fn stopped(&self) -> bool {
        self.stopped || self.position >= self.code.len()
    }

    fn read(&mut self, byte_count: usize) -> M256 {
        let position = self.position;
        self.position += byte_count;
        let max = min(position + byte_count, self.code.len());
        M256::from(&self.code[position..max])
    }
}
