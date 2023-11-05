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
use evm_interpreter::{Capture, Control, Etable, ExitError, ExitResult, Machine, Opcode, Trap};
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
		&mut self,
		machine: &Machine<S>,
		handler: &H,
		is_static: bool,
	) -> Result<usize, ExitError>;
	fn record_codedeposit(&mut self, len: usize) -> Result<(), ExitError>;
	fn gas(&self) -> Self::Gas;
	fn merge(&mut self, other: Self, strategy: GasometerMergeStrategy);
}

pub fn run_with_gasometer<S, H, G, F>(
	machine: &mut Machine<S>,
	gasometer: &mut G,
	handler: &mut H,
	is_static: bool,
	etable: &Etable<S, H, F>,
) -> Capture<ExitResult, Trap>
where
	G: Gasometer<S, H>,
	F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control,
{
	loop {
		match gasometer.record_stepn(&machine, handler, is_static) {
			Ok(stepn) => match machine.stepn(stepn, handler, etable) {
				Ok(()) => (),
				Err(c) => return c,
			},
			Err(e) => return Capture::Exit(Err(e)),
		}
	}
}
