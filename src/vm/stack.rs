use utils::bigint::M256;
use super::{Result, Error};

pub struct Stack {
    stack: Vec<M256>,
}

impl Default for Stack {
    fn default() -> VectorStack {
        VectorStack {
            stack: Vec::new(),
        }
    }
}

impl Stack {
    fn push(&mut self, elem: M256) -> Result<()> {
        self.stack.push(elem);
        if self.size() > 1024 {
            self.stack.pop();
            Err(Error::StackOverflow)
        } else {
            Ok(())
        }
    }

    fn pop(&mut self) -> Result<M256> {
        match self.stack.pop() {
            Some(x) => Ok(x),
            None => Err(Error::StackUnderflow),
        }
    }

    fn set(&mut self, no_from_top: usize, val: M256) -> Result<()> {
        if self.stack.len() > no_from_top {
            let len = self.stack.len();
            self.stack[len - no_from_top - 1] = val;
            Ok(())
        } else {
            Err(Error::StackUnderflow)
        }
    }

    fn peek(&self, no_from_top: usize) -> Result<M256> {
        if self.stack.len() > no_from_top {
            Ok(self.stack[self.stack.len() - no_from_top - 1])
        } else {
            Err(Error::StackUnderflow)
        }
    }

    fn len(&self) -> usize {
        self.stack.len()
    }
}
