//! Ethereum Virtual Machine implementation in Rust

#![deny(warnings)]
#![forbid(unused_variables)]

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use evm_core::*;
pub use evm_gasometer as gasometer;
pub use evm_runtime::*;

#[cfg(feature = "tracing")]
pub mod tracing;

#[cfg(feature = "tracing")]
macro_rules! event {
	($x:expr) => {
		use crate::tracing::Event::*;
		if crate::tracing::is_tracing_enabled() {
			$x.emit();
		}
	}
}

#[cfg(not(feature = "tracing"))]
macro_rules! event {
	($x:expr) => {};
}

pub mod backend;
pub mod executor;
