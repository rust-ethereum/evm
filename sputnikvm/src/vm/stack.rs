//! EVM stack

use utils::bigint::M256;
use super::errors::StackError;

/// Represents an EVM stack.
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
    /// Check a pop-push cycle. If the check succeeded, `push`, `pop`,
    /// `set`, `peek` within the limit should not fail.
    pub fn check_pop_push(&self, pop: usize, push: usize) -> Result<(), StackError> {
        if self.len() < pop {
            return Err(StackError::Underflow);
        }
        if self.len() - pop + push > 1024 {
            return Err(StackError::Overflow);
        }
        Ok(())
    }

    /// Push a new value to the stack.
    pub fn push(&mut self, elem: M256) -> Result<(), StackError> {
        self.stack.push(elem);
        if self.len() > 1024 {
            self.stack.pop();
            Err(StackError::Overflow)
        } else {
            Ok(())
        }
    }

    /// Pop a value from the stack.
    pub fn pop(&mut self) -> Result<M256, StackError> {
        match self.stack.pop() {
            Some(x) => Ok(x),
            None => Err(StackError::Underflow),
        }
    }

    /// Set a value at given index for the stack, where the top of the
    /// stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    pub fn set(&mut self, no_from_top: usize, val: M256) -> Result<(), StackError> {
        if self.stack.len() > no_from_top {
            let len = self.stack.len();
            self.stack[len - no_from_top - 1] = val;
            Ok(())
        } else {
            Err(StackError::Underflow)
        }
    }

    /// Peek a value at given index for the stack, where the top of
    /// the stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    pub fn peek(&self, no_from_top: usize) -> Result<M256, StackError> {
        if self.stack.len() > no_from_top {
            Ok(self.stack[self.stack.len() - no_from_top - 1])
        } else {
            Err(StackError::Underflow)
        }
    }

    /// Get the current stack length.
    pub fn len(&self) -> usize {
        self.stack.len()
    }
}
