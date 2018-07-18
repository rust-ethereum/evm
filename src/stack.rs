//! EVM stack

#[cfg(not(feature = "std"))]
use alloc::Vec;

use bigint::M256;
use super::errors::OnChainError;

/// Represents an EVM stack.
#[derive(Debug)]
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
    pub fn check_pop_push(&self, pop: usize, push: usize) -> Result<(), OnChainError> {
        if self.len() < pop {
            return Err(OnChainError::StackUnderflow);
        }
        if self.len() - pop + push > 1024 {
            return Err(OnChainError::StackOverflow);
        }
        Ok(())
    }

    /// Push a new value to the stack.
    pub fn push(&mut self, elem: M256) -> Result<(), OnChainError> {
        self.stack.push(elem);
        if self.len() > 1024 {
            self.stack.pop();
            Err(OnChainError::StackOverflow)
        } else {
            Ok(())
        }
    }

    /// Pop a value from the stack.
    pub fn pop(&mut self) -> Result<M256, OnChainError> {
        match self.stack.pop() {
            Some(x) => Ok(x),
            None => Err(OnChainError::StackUnderflow),
        }
    }

    /// Set a value at given index for the stack, where the top of the
    /// stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    pub fn set(&mut self, no_from_top: usize, val: M256) -> Result<(), OnChainError> {
        if self.stack.len() > no_from_top {
            let len = self.stack.len();
            self.stack[len - no_from_top - 1] = val;
            Ok(())
        } else {
            Err(OnChainError::StackUnderflow)
        }
    }

    /// Peek a value at given index for the stack, where the top of
    /// the stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    pub fn peek(&self, no_from_top: usize) -> Result<M256, OnChainError> {
        if self.stack.len() > no_from_top {
            Ok(self.stack[self.stack.len() - no_from_top - 1])
        } else {
            Err(OnChainError::StackUnderflow)
        }
    }

    /// Get the current stack length.
    pub fn len(&self) -> usize {
        self.stack.len()
    }
}
