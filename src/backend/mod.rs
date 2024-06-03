//! # Backend-related traits and implementations
//!
//! A backend exposes external information that is available to an EVM
//! interpreter. This includes block information such as the current coinbase,
//! block gas limit, etc, as well as the state such as account balance, storage
//! and code.
//!
//! Backends have layers, representing information that may be committed or
//! discard after the current call stack finishes. Due to the vast differences of
//! how different backends behave (for example, in some backends like wasm,
//! pushing/poping layers are dealt by extern functions), layers are handled
//! internally inside a backend.

mod overlayed;

pub use evm_interpreter::runtime::{RuntimeBackend, RuntimeBaseBackend, RuntimeEnvironment};

pub use self::overlayed::{OverlayedBackend, OverlayedChangeSet};

/// Backend with layers that can transactionally be committed or discarded.
pub trait TransactionalBackend {
	/// Push a new substate layer into the backend.
	fn push_substate(&mut self);
	/// Pop the last substate layer from the backend, either committing or
	/// discarding it.
	///
	/// The caller is expected to maintain balance of push/pop, and the backend
	/// are free to panic if it does not.
	fn pop_substate(&mut self, strategy: crate::MergeStrategy);
}
