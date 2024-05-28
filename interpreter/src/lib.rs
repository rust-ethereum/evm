//! Core layer for EVM.

// #![deny(warnings)]
// #![forbid(unsafe_code, unused_variables, unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod error;
pub mod etable;
pub mod eval;
mod interpreter;
pub mod machine;
pub mod opcode;
pub mod runtime;
pub mod utils;

pub use self::interpreter::{EtableInterpreter, Interpreter, RunInterpreter, StepInterpreter};
