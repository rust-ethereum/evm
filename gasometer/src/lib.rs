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
use evm_interpreter::{
	Capture, Control, Etable, ExitError, ExitResult, ExitSucceed, Machine, Opcode, Trap,
};
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
pub enum MergeStrategy {
	Commit,
	Revert,
	Discard,
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
	fn merge(&mut self, other: Self, strategy: MergeStrategy);
}

pub enum ExecutionResult<S, G> {
	Ok(Machine<S>, G, ExitSucceed),
	ErrLeftGas(Machine<S>, G, ExitError),
	ErrNoGas(Machine<S>, ExitError),
}

pub fn run_with_gasometer<S, H, Tr, G, F>(
	mut machine: Machine<S>,
	mut gasometer: G,
	handler: &mut H,
	is_static: bool,
	etable: &Etable<S, H, Tr, F>,
) -> Capture<ExecutionResult<S, G>, (Tr, G)>
where
	G: Gasometer<S, H>,
	F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr::Data>,
	Tr: Trap<S>,
{
	loop {
		match gasometer.record_stepn(&machine, handler, is_static) {
			Ok((g, stepn)) => {
				gasometer = g;

				match machine.stepn(stepn, handler, etable) {
					Ok(m) => machine = m,
					Err(Capture::Exit((m, Ok(e)))) => {
						return Capture::Exit(ExecutionResult::Ok(m, gasometer, e))
					}
					Err(Capture::Exit((m, Err(e)))) => {
						return Capture::Exit(ExecutionResult::ErrLeftGas(m, gasometer, e))
					}
					Err(Capture::Trap(t)) => return Capture::Trap((t, gasometer)),
				}
			}
			Err(e) => return Capture::Exit(ExecutionResult::ErrNoGas(machine, e)),
		}
	}
}
