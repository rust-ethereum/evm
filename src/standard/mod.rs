//! # Standard machines and gasometers
//!
//! This module implements the standard configurations of the interpreter, like how it works on
//! Ethereum mainnet. Most of them can still be customized to add additional functionality, by
//! wrapping them or replacing the generic parameters.

mod config;
mod gasometer;
mod invoker;

pub use self::config::Config;
pub use self::gasometer::{Gasometer, TransactGasometer};
pub use self::invoker::{EtableResolver, Invoker, PrecompileSet, Resolver, TransactArgs};

/// Standard EVM machine, where the runtime state is [crate::RuntimeState].
pub type Machine = crate::Machine<crate::RuntimeState>;

/// Standard Etable opcode handle function.
pub type Efn<H> = crate::Efn<crate::RuntimeState, H, crate::Opcode>;

/// Standard Etable.
pub type Etable<H, F = Efn<H>> = crate::Etable<crate::RuntimeState, H, crate::Opcode, F>;

/// Standard colored machine, combining an interpreter machine, a gasometer, and the standard
/// "color" -- an etable.
pub type ColoredMachine<'etable, G, H, F = Efn<H>> =
	crate::ColoredMachine<crate::RuntimeState, G, &'etable Etable<H, F>>;

/// Simply [Invoker] with common generics fixed, using standard [Gasometer] and standard trap
/// [crate::Opcode].
pub type SimpleInvoker<'config, 'resolver, H, R> =
	Invoker<'config, 'resolver, crate::RuntimeState, Gasometer<'config>, H, R, crate::Opcode>;

/// A runtime state that can be merged across call stack substate layers.
pub trait MergeableRuntimeState<M>:
	AsRef<crate::RuntimeState> + AsMut<crate::RuntimeState>
{
	/// Derive a new substate from the substate runtime.
	fn substate(&self, runtime: crate::RuntimeState, parent: &M) -> Self;
	/// Merge a substate into the current runtime state, using the given
	/// strategy.
	fn merge(&mut self, substate: Self, strategy: crate::MergeStrategy);
	/// Create a new top-layer runtime state with a call transaction.
	fn new_transact_call(runtime: crate::RuntimeState) -> Self;
	/// Create a new top-layer runtime state with a create transaction.
	fn new_transact_create(runtime: crate::RuntimeState) -> Self;
}

impl<M> MergeableRuntimeState<M> for crate::RuntimeState {
	fn substate(&self, runtime: crate::RuntimeState, _parent: &M) -> Self {
		runtime
	}
	fn merge(&mut self, _substate: Self, _strategy: crate::MergeStrategy) {}
	fn new_transact_call(runtime: crate::RuntimeState) -> Self {
		runtime
	}
	fn new_transact_create(runtime: crate::RuntimeState) -> Self {
		runtime
	}
}
