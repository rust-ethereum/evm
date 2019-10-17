use primitive_types::H256;
use crate::ExitReason;

pub struct Stack {
    data: Vec<H256>,
    limit: usize,
}

impl Stack {
    pub fn pop(&mut self) -> Result<H256, ExitReason> {
        self.data.pop().ok_or(ExitReason::StackUnderflow)
    }

    pub fn push(&mut self, value: H256) -> Result<(), ExitReason> {
        if self.data.len() + 1 > self.limit {
            return Err(ExitReason::StackOverflow)
        }
        self.data.push(value);
        Ok(())
    }
}
