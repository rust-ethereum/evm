//! Ethereum Virtual Machine implementation in Rust

// #![deny(warnings)]
// #![forbid(unsafe_code, unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod backend;
mod executor;

pub use evm_gasometer as gasometer;
pub use evm_interpreter::*;

pub use crate::backend::{TransactionalBackend, TransactionalMergeStrategy};
pub use crate::executor::{CallStack, Invoker};
pub use crate::gasometer::Config;
