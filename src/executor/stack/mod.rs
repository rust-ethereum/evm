//! A stack-based executor with customizable state.
//! A memory-based state is provided, but can replaced by a custom
//! implementation, for exemple one interacting with a database.

mod executor;
mod memory;

pub use self::executor::{
	Accessed, PrecompileFailure, PrecompileFn, PrecompileOutput, PrecompileSet, StackExecutor,
	StackExitKind, StackState, StackSubstateMetadata,
};

pub use self::memory::{MemoryStackAccount, MemoryStackState, MemoryStackSubstate};

pub use ethereum::Log;
