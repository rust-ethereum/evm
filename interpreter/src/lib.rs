//! Core layer for EVM.

#![deny(warnings, unused_variables, unused_imports)]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod error;
mod interpreter;
mod machine;
mod opcode;

pub mod etable;
pub mod eval;
pub mod runtime;
pub mod trap;
pub mod uint;
pub mod utils;

pub use self::error::{Capture, ExitError, ExitException, ExitFatal, ExitResult, ExitSucceed};
pub use self::interpreter::{
	Control, EtableInterpreter, FeedbackInterpreter, Interpreter, StepInterpreter, Valids,
};
pub use self::machine::{Machine, Memory, Stack};
pub use self::opcode::Opcode;
