//! EVM gasometer.

#![cfg_attr(not(feature = "std"), no_std)]

mod config;
mod consts;
mod costs;
mod standard;
mod utils;

pub use crate::config::Config;
pub use crate::standard::StandardGasometer;

use core::ops::{Add, AddAssign, Sub, SubAssign};
use evm_interpreter::{ExitError, Machine};
use primitive_types::U256;

pub trait Gas:
	Copy
	+ Into<U256>
	+ Add<Self, Output = Self>
	+ AddAssign<Self>
	+ Sub<Self, Output = Self>
	+ SubAssign<Self>
{
}

impl Gas for u64 {}
impl Gas for U256 {}

#[derive(Clone, Copy)]
pub enum GasometerMergeStrategy {
	Commit,
	Revert,
}

pub trait Gasometer<S, H>: Sized {
	type Gas: Gas;
	type Config;

	fn new(gas_limit: Self::Gas, machine: &Machine<S>, config: Self::Config) -> Self;
	fn record_stepn(
		self,
		machine: &Machine<S>,
		handler: &H,
		is_static: bool,
	) -> Result<(Self, usize), ExitError>;
	fn record_codedeposit(self, len: usize) -> Result<Self, ExitError>;
	fn gas(&self) -> Self::Gas;
	fn merge(&mut self, other: Self, strategy: GasometerMergeStrategy);
}
