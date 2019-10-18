use primitive_types::H256;
use crate::ExitError;

#[derive(Clone, Debug)]
pub struct Stack {
    data: Vec<H256>,
    limit: usize,
}

impl Stack {
    pub fn new(limit: usize) -> Self {
        Self {
            data: Vec::new(),
            limit,
        }
    }

    pub fn pop(&mut self) -> Result<H256, ExitError> {
        self.data.pop().ok_or(ExitError::StackUnderflow)
    }

    pub fn push(&mut self, value: H256) -> Result<(), ExitError> {
        if self.data.len() + 1 > self.limit {
            return Err(ExitError::StackOverflow)
        }
        self.data.push(value);
        Ok(())
    }

    /// Peek a value at given index for the stack, where the top of
    /// the stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    pub fn peek(&self, no_from_top: usize) -> Result<H256, ExitError> {
        if self.data.len() > no_from_top {
            Ok(self.data[self.data.len() - no_from_top - 1])
        } else {
            Err(ExitError::StackUnderflow)
        }
    }

    /// Set a value at given index for the stack, where the top of the
    /// stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    pub fn set(&mut self, no_from_top: usize, val: H256) -> Result<(), ExitError> {
        if self.data.len() > no_from_top {
            let len = self.data.len();
            self.data[len - no_from_top - 1] = val;
            Ok(())
        } else {
            Err(ExitError::StackUnderflow)
        }
    }
}
