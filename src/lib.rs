//! Ethereum Virtual Machine implementation in Rust

// #![deny(warnings)]
// #![forbid(unsafe_code, unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod backend;
pub mod standard;

mod call_stack;
mod gasometer;
mod invoker;

pub use evm_interpreter::*;

pub use crate::backend::TransactionalBackend;
pub use crate::call_stack::{transact, HeapTransact};
pub use crate::gasometer::{Gas, GasedMachine, Gasometer};
pub use crate::invoker::{Invoker, InvokerControl};

#[derive(Clone, Debug, Copy)]
pub enum MergeStrategy {
	Commit,
	Revert,
	Discard,
}
