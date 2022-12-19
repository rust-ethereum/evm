//! A stack-based executor with customizable state.
//! A memory-based state is provided, but can replaced by a custom
//! implementation, for exemple one interacting with a database.

mod executor;
mod memory;
mod tagged_runtime;

pub use self::executor::{
	Accessed, PrecompileFailure, PrecompileFn, PrecompileHandle, PrecompileOutput, PrecompileSet,
	StackExecutor, StackExitKind, StackState, StackSubstateMetadata,
};

pub use self::memory::{MemoryStackAccount, MemoryStackState, MemoryStackSubstate};

pub use ethereum::Log;
