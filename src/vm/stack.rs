use utils::bigint::M256;
use super::{Result, Error};

pub struct Stack {
    stack: Vec<M256>,
}

impl Default for Stack {
    fn default() -> Stack {
        Stack {
            stack: Vec::new(),
        }
    }
}

impl Stack {
    pub fn push(&mut self, elem: M256) -> Result<()> {
        self.stack.push(elem);
        if self.size() > 1024 {
            self.stack.pop();
            Err(Error::StackOverflow)
        } else {
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Result<M256> {
        match self.stack.pop() {
            Some(x) => Ok(x),
            None => Err(Error::StackUnderflow),
        }
    }

    pub fn set(&mut self, no_from_top: usize, val: M256) -> Result<()> {
        if self.stack.len() > no_from_top {
            let len = self.stack.len();
            self.stack[len - no_from_top - 1] = val;
            Ok(())
        } else {
            Err(Error::StackUnderflow)
        }
    }

    pub fn peek(&self, no_from_top: usize) -> Result<M256> {
        if self.stack.len() > no_from_top {
            Ok(self.stack[self.stack.len() - no_from_top - 1])
        } else {
            Err(Error::StackUnderflow)
        }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
}
