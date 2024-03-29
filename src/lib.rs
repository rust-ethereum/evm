//! Ethereum Virtual Machine implementation in Rust

#![deny(warnings)]
#![forbid(unsafe_code, unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
pub mod prelude {
	pub use alloc::{
		boxed::Box,
		collections::{BTreeMap, BTreeSet},
		rc::Rc,
		vec::Vec,
	};
}
#[cfg(feature = "std")]
pub mod prelude {
	pub use std::{
		collections::{BTreeMap, BTreeSet},
		rc::Rc,
		vec::Vec,
	};
}

pub use evm_core::*;
pub use evm_gasometer as gasometer;
pub use evm_runtime::*;

#[cfg(feature = "tracing")]
pub mod tracing;

#[cfg(feature = "tracing")]
macro_rules! event {
	($x:expr) => {
		use crate::tracing::Event::*;
		crate::tracing::with(|listener| listener.event($x));
	};
}

#[cfg(not(feature = "tracing"))]
macro_rules! event {
	($x:expr) => {};
}

pub mod backend;
pub mod executor;
pub mod maybe_borrowed;
