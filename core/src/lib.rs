#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;
extern crate alloc;

mod memory;
mod stack;
mod valids;
mod opcode;
mod trap;

use core::ops::Range;
use alloc::rc::Rc;
use crate::memory::Memory;
use crate::stack::Stack;
use crate::valids::Valids;
use crate::opcode::{Opcode, ExternalOpcode};
use crate::trap::{Trap, ExitReason};

/// Core execution layer for EVM.
pub struct Core {
    /// Program code.
    code: Rc<Vec<u8>>,
    /// Program counter.
    position: Result<usize, ExitReason>,
    /// Return value range.
    return_range: Range<usize>,
    /// Code validity maps.
    valids: Valids,
    /// Memory.
    memory: Memory,
    /// Stack.
    stack: Stack,
}
