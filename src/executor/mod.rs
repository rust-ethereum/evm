//! # EVM executors
//!
//! Executors are structs that hook gasometer and the EVM core together. It
//! also handles the call stacks in EVM.

pub mod stack;

pub use self::stack::{
	MemoryStackState, Precompile, PrecompileOutput, StackExecutor, StackExitKind, StackState,
	StackSubstateMetadata,
};
