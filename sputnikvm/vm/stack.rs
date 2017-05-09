use utils::bigint::M256;
use super::{ExecutionResult, ExecutionError};

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
    pub fn push(&mut self, elem: M256) -> ExecutionResult<()> {
        self.stack.push(elem);
        if self.len() > 1024 {
            self.stack.pop();
            Err(ExecutionError::StackOverflow)
        } else {
            Ok(())
        }
    }

    pub fn pop(&mut self) -> ExecutionResult<M256> {
        match self.stack.pop() {
            Some(x) => Ok(x),
            None => Err(ExecutionError::StackUnderflow),
        }
    }

    pub fn set(&mut self, no_from_top: usize, val: M256) -> ExecutionResult<()> {
        if self.stack.len() > no_from_top {
            let len = self.stack.len();
            self.stack[len - no_from_top - 1] = val;
            Ok(())
        } else {
            Err(ExecutionError::StackUnderflow)
        }
    }

    pub fn peek(&self, no_from_top: usize) -> ExecutionResult<M256> {
        if self.stack.len() > no_from_top {
            Ok(self.stack[self.stack.len() - no_from_top - 1])
        } else {
            Err(ExecutionError::StackUnderflow)
        }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
}
