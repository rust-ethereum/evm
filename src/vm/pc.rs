use utils::u256::U256;
use std::cmp::{min};
use super::opcode::Opcode;

pub struct PC<'a> {
    pub position: usize,
    code: &'a [u8],
    stopped: bool
}

impl<'a> PC<'a> {
    pub fn new(code: &'a [u8]) -> Self {
        PC {
            position: 0,
            code: code,
            stopped: false,
        }
    }

    pub fn peek_opcode(&self) -> Opcode {
        let position = self.position;
        let opcode: Opcode = self.code[position].into();
        opcode
    }

    pub fn read_opcode(&mut self) -> Opcode {
        let position = self.position;
        let opcode: Opcode = self.code[position].into();
        self.position += 1;
        opcode
    }

    pub fn stop(&mut self) {
        self.stopped = true;
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped || self.position >= self.code.len()
    }

    pub fn read(&mut self, byte_count: usize) -> U256 {
        let position = self.position;
        self.position += byte_count;
        let max = min(position + byte_count, self.code.len());
        U256::from(&self.code[position..max])
    }

    fn len(&self) -> usize {
        self.code.len()
    }
}
