#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;
extern crate alloc;

mod memory;
mod stack;
mod valids;
mod opcode;
mod trap;
mod eval;
mod utils;

pub use crate::memory::Memory;
pub use crate::stack::Stack;
pub use crate::valids::Valids;
pub use crate::opcode::{Opcode, ExternalOpcode};
pub use crate::trap::{Trap, ExitReason};

use core::ops::Range;
use alloc::rc::Rc;
use primitive_types::U256;
use crate::eval::{eval, Control};

/// Core execution layer for EVM.
pub struct Core {
    /// Program code.
    code: Rc<Vec<u8>>,
    /// Program counter.
    position: Result<usize, ExitReason>,
    /// Return value.
    return_range: Range<U256>,
    /// Code validity maps.
    valids: Valids,
    /// Memory.
    memory: Memory,
    /// Stack.
    stack: Stack,
}

impl Core {
    pub fn inspect(&self) -> Option<(Result<Opcode, ExternalOpcode>, &Stack)> {
        let position = match self.position {
            Ok(position) => position,
            Err(_) => return None,
        };
        self.code.get(position).map(|v| (Opcode::parse(*v), &self.stack))
    }

    pub fn step(&mut self) -> Result<(), Trap> {
        let position = self.position?;

        match self.code.get(position).map(|v| Opcode::parse(*v)) {
            Some(Ok(opcode)) => {
                match eval(self, opcode, position) {
                    Control::Continue(p) => {
                        self.position = Ok(position + p);
                        Ok(())
                    },
                    Control::Exit(e) => {
                        self.position = Err(e);
                        Err(Trap::Exit(e))
                    },
                    Control::Jump(p) => {
                        if self.valids.is_valid(p) {
                            self.position = Ok(p);
                            Ok(())
                        } else {
                            self.position = Err(ExitReason::InvalidJump);
                            Err(Trap::Exit(ExitReason::InvalidJump))
                        }
                    },
                }
            },
            Some(Err(external)) => {
                self.position = Ok(position + 1);
                Err(Trap::External(external))
            },
            None => {
                self.position = Err(ExitReason::Stopped);
                Err(Trap::Exit(ExitReason::Stopped))
            },
        }
    }
}
