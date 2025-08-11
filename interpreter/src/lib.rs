//! Core layer for EVM.

// #![deny(warnings)]
// #![forbid(unsafe_code, unused_variables, unused_imports)]
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
pub mod utils;

pub use self::error::{Capture, ExitError, ExitException, ExitFatal, ExitResult, ExitSucceed};
pub use self::interpreter::{
	Control, EtableInterpreter, FeedbackInterpreter, Interpreter, StepInterpreter, Valids,
};
pub use self::machine::{AsMachine, AsMachineMut, Machine, Memory, Stack};
pub use self::opcode::Opcode;
