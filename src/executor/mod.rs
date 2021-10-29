//! # EVM executors
//!
//! Executors are structs that hook gasometer and the EVM core together. It
//! also handles the call stacks in EVM.

mod memory;
mod stack;

pub use self::stack::{
	PrecompileFailure, PrecompileFn, PrecompileOutput, PrecompileSet, StackExecutor, StackExitKind,
	StackState, StackSubstateMetadata,
};

pub use self::memory::{MemoryStackAccount, MemoryStackState, MemoryStackSubstate};

pub use ethereum::Log;
