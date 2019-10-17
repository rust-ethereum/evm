#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;
extern crate alloc;

mod memory;
mod stack;
mod valids;
mod opcode;

use alloc::rc::Rc;
use crate::memory::Memory;
use crate::stack::Stack;
use crate::valids::Valids;

/// Core execution layer for EVM.
pub struct Core {
    /// Program code.
    code: Rc<Vec<u8>>,
    /// Program counter.
    position: usize,
    /// Code validity maps.
    valids: Valids,
    /// Memory.
    memory: Memory,
    /// Stack.
    stack: Stack,
}
