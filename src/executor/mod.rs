//! # EVM executors
//!
//! Executors are structs that hook gasometer and the EVM core together. It
//! also handles the call stacks in EVM.
//!
//! Currently only a stack-based (customizable) executor is provided.

pub mod stack;
