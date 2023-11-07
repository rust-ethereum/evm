//! Ethereum Virtual Machine implementation in Rust

// #![deny(warnings)]
// #![forbid(unsafe_code, unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod standard;

mod backend;
mod call_stack;
mod gasometer;
mod invoker;

pub use evm_interpreter::*;

pub use crate::backend::{TransactionalBackend, TransactionalMergeStrategy};
pub use crate::call_stack::{execute, CallStack};
pub use crate::gasometer::{run_with_gasometer, Gas, Gasometer, GasometerMergeStrategy};
pub use crate::invoker::Invoker;
