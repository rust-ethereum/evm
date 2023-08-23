use crate::{Context, ExitError, ExitFatal, ExitReason, ExitRevert, ExitSucceed, Transfer};
use alloc::{collections::BTreeMap, vec::Vec};
use primitive_types::{H160, H256};

/// A precompile result.
pub type PrecompileResult = Result<PrecompileOutput, PrecompileFailure>;

/// Data returned by a precompile on success.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PrecompileOutput {
	pub exit_status: ExitSucceed,
	pub output: Vec<u8>,
}

/// Data returned by a precompile in case of failure.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PrecompileFailure {
	/// Reverts the state changes and consume all the gas.
	Error { exit_status: ExitError },
	/// Reverts the state changes.
	/// Returns the provided error message.
	Revert {
		exit_status: ExitRevert,
		output: Vec<u8>,
	},
	/// Mark this failure as fatal, and all EVM execution stacks must be exited.
	Fatal { exit_status: ExitFatal },
}

impl From<ExitError> for PrecompileFailure {
	fn from(error: ExitError) -> PrecompileFailure {
		PrecompileFailure::Error { exit_status: error }
	}
}

/// Handle provided to a precompile to interact with the EVM.
pub trait PrecompileHandle {
	/// Perform subcall in provided context.
	/// Precompile specifies in which context the subcall is executed.
	fn call(
		&mut self,
		to: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		gas_limit: Option<u64>,
		is_static: bool,
		context: &Context,
	) -> (ExitReason, Vec<u8>);

	/// Record cost to the Runtime gasometer.
	fn record_cost(&mut self, cost: u64) -> Result<(), ExitError>;

	/// Record Substrate specific cost.
	fn record_external_cost(
		&mut self,
		ref_time: Option<u64>,
		proof_size: Option<u64>,
		storage_growth: Option<u64>,
	) -> Result<(), ExitError>;

	/// Refund Substrate specific cost.
	fn refund_external_cost(&mut self, ref_time: Option<u64>, proof_size: Option<u64>);

	/// Retreive the remaining gas.
	fn remaining_gas(&self) -> u64;

	/// Record a log.
	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError>;

	/// Retreive the code address (what is the address of the precompile being called).
	fn code_address(&self) -> H160;

	/// Retreive the input data the precompile is called with.
	fn input(&self) -> &[u8];

	/// Retreive the context in which the precompile is executed.
	fn context(&self) -> &Context;

	/// Is the precompile call is done statically.
	fn is_static(&self) -> bool;

	/// Retreive the gas limit of this call.
	fn gas_limit(&self) -> Option<u64>;
}

/// A set of precompiles.
///
/// Checks if the provided address is in the precompile set. This should be
/// as cheap as possible since it may be called often.
pub trait PrecompileSet {
	/// Tries to execute a precompile in the precompile set.
	/// If the provided address is not a precompile, returns None.
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult>;

	/// Check if the given address is a precompile. Should only be called to
	/// perform the check while not executing the precompile afterward, since
	/// `execute` already performs a check internally.
	fn is_precompile(&self, address: H160, remaining_gas: u64) -> IsPrecompileResult;
}

pub enum IsPrecompileResult {
	Answer {
		is_precompile: bool,
		extra_cost: u64,
	},
	OutOfGas,
}

impl PrecompileSet for () {
	fn execute(&self, _: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		None
	}

	fn is_precompile(&self, _: H160, _: u64) -> IsPrecompileResult {
		IsPrecompileResult::Answer {
			is_precompile: false,
			extra_cost: 0,
		}
	}
}

/// Precompiles function signature. Expected input arguments are:
///  * Input
///  * Gas limit
///  * Context
///  * Is static
///
/// In case of success returns the output and the cost.
pub type PrecompileFn =
	fn(&[u8], Option<u64>, &Context, bool) -> Result<(PrecompileOutput, u64), PrecompileFailure>;

impl PrecompileSet for BTreeMap<H160, PrecompileFn> {
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		let address = handle.code_address();

		self.get(&address).map(|precompile| {
			let input = handle.input();
			let gas_limit = handle.gas_limit();
			let context = handle.context();
			let is_static = handle.is_static();

			match (*precompile)(input, gas_limit, context, is_static) {
				Ok((output, cost)) => {
					handle.record_cost(cost)?;
					Ok(output)
				}
				Err(err) => Err(err),
			}
		})
	}

	/// Check if the given address is a precompile. Should only be called to
	/// perform the check while not executing the precompile afterward, since
	/// `execute` already performs a check internally.
	fn is_precompile(&self, address: H160, _: u64) -> IsPrecompileResult {
		IsPrecompileResult::Answer {
			is_precompile: self.contains_key(&address),
			extra_cost: 0,
		}
	}
}
