//! Ethereum Virtual Machine implementation in Rust

// #![deny(warnings)]
// #![forbid(unsafe_code, unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod backend;
pub mod standard;

mod call_stack;
mod color;
mod gasometer;
mod invoker;

pub use evm_interpreter::*;

pub use crate::backend::TransactionalBackend;
pub use crate::call_stack::{transact, HeapTransact};
pub use crate::color::{Color, ColoredMachine};
pub use crate::gasometer::{Gas, Gasometer, StaticGasometer};
pub use crate::invoker::{Invoker, InvokerControl, InvokerMachine};

#[derive(Clone, Debug, Copy)]
pub enum MergeStrategy {
	Commit,
	Revert,
	Discard,
}
