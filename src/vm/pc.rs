use utils::u256::U256;
use std::cmp::{min};
use super::opcode::Opcode;

pub trait PC {
    fn peek_opcode(&self) -> Opcode;
    fn read_opcode(&mut self) -> Opcode;
    fn stop(&mut self);
    fn stopped(&self) -> bool;
    fn read(&mut self, byte_count: usize) -> U256;
    fn position(&self) -> usize;
}

pub struct VectorPC {
    position: usize,
    code: Vec<u8>,
    stopped: bool
}

impl VectorPC {
    pub fn new(code: &[u8]) -> Self {
        PC {
            position: 0,
            code: code.into(),
            stopped: false,
        }
    }
}

impl<A: Account> From<&A> for VectorPC {
    fn from(account: &A) -> VectorPC {
        let empty: [u8; 0] = [];
        let code = account.code;
        if code.is_some() {
            VectorPC::new(code.unwrap())
        } else {
            VectorPC::new(empty)
        }
    }
}

impl PC for VectorPC {
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

    fn read(&mut self, byte_count: usize) -> U256 {
        let position = self.position;
        self.position += byte_count;
        let max = min(position + byte_count, self.code.len());
        U256::from(&self.code[position..max])
    }
}
