//! # Ethereum Virtual Machine in Rust
//!
//! Rust EVM is a flexible Ethereum Virtual Machine interpreter that can be
//! easily customized.
//!
//! ## Basic usage
//!
//! The entrypoint of a normal EVM execution is through the [transact] function.
//! The [transact] function implements a hybrid (stack-based, and then
//! heap-based) call stack.
//!
//! To use the [transact] function, you will need to first implement a
//! backend. This is anything that implements [RuntimeEnvironment],
//! [RuntimeBaseBackend] and [RuntimeBackend] traits. You will also need to
//! select a few other components to construct the `invoker` parameter needed
//! for the function.
//!
//! * Select an [Invoker]. The invoker defines all details of the execution
//!   environment except the external backend. [standard::Invoker] is
//!   probably want you want if you are not extending EVM.
//! * For the standard invoker, select a [standard::Config], which represents
//!   different Ethereum hard forks.
//! * Select the precompile set. You may want the `StandardPrecompileSet` in
//!   `evm-precompile` crate.
//! * Select a resolver. This defines how the interpreter machines are resolved
//!   given a code address for call or an init code for create. You may want
//!   [standard::EtableResolver], which accepts a precompile set.
//!
//! ## Debugging
//!
//! Rust EVM supports two different methods for debugging. You can either single
//! step the execution, or you can trace the opcodes.
//!
//! ### Single stepping
//!
//! Single stepping allows you to examine the full machine internal state every
//! time the interpreter finishes executing a single opcode. To do this, use the
//! heap-only call stack [HeapTransact]. Parameters passed to [HeapTransact] are
//! the same as [transact].
//!
//! ### Tracing
//!
//! The interpreter machine uses information from an [Etable] to decide how each
//! opcode behaves. An [Etable] is fully customizable and a helper function is
//! also provided [Etable::wrap].
//!
//! If you also want to trace inside gasometers, simply create a wrapper struct
//! of the gasometer you use, and pass that into the invoker.
//!
//! ## Customization
//!
//! All aspects of the interpreter can be customized individually.
//!
//! * New opcodes can be added or customized through [Etable].
//! * Gas metering behavior can be customized by wrapping [standard::Gasometer] or creating new
//!   ones.
//! * Code resolution and precompiles can be customized by [standard::Resolver].
//! * Call invocation and transaction behavior can be customized via [standard::Invoker].
//! * Finally, each machine on the call stack has the concept of [Color], which allows you to
//!   implement account versioning, or specialized precompiles that invoke subcalls.

#![deny(warnings)]
#![forbid(unsafe_code, unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod backend;
pub mod standard;

mod call_stack;
mod gasometer;
mod invoker;

pub use evm_interpreter as interpreter;

pub use crate::{
	backend::TransactionalBackend,
	call_stack::{transact, HeapTransact},
	gasometer::GasMutState,
	invoker::{Invoker, InvokerControl},
};

/// Merge strategy of a backend substate layer or a call stack gasometer layer.
#[derive(Clone, Debug, Copy)]
pub enum MergeStrategy {
	/// Fully commit the sub-layer into the parent. This happens if the sub-machine executes
	/// successfully.
	Commit,
	/// Revert the state, but keep remaining gases. This happens with the `REVERT` opcode.
	Revert,
	/// Discard the state and gases. This happens in all situations where the machine encounters an
	/// error.
	Discard,
}
