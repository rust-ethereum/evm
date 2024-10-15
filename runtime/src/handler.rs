use crate::prelude::*;
use crate::{Capture, Context, CreateScheme, ExitError, ExitReason, Machine, Opcode};
use primitive_types::{H160, H256, U256};

/// Transfer from source to target, with given value.
#[derive(Clone, Debug)]
pub struct Transfer {
	/// Source address.
	pub source: H160,
	/// Target address.
	pub target: H160,
	/// Transfer value.
	pub value: U256,
}

/// EVM context handler.
#[auto_impl::auto_impl(& mut, Box)]
pub trait Handler {
	/// Type of `CREATE` interrupt.
	type CreateInterrupt;
	/// Feedback value for `CREATE` interrupt.
	type CreateFeedback;
	/// Type of `CALL` interrupt.
	type CallInterrupt;
	/// Feedback value of `CALL` interrupt.
	type CallFeedback;

	/// Get balance of address.
	fn balance(&self, address: H160) -> U256;
	/// Get code size of address.
	fn code_size(&self, address: H160) -> U256;
	/// Get code hash of address.
	fn code_hash(&self, address: H160) -> H256;
	/// Get code of address.
	fn code(&self, address: H160) -> Vec<u8>;
	/// Get storage value of address at index.
	fn storage(&self, address: H160, index: H256) -> H256;
	/// Check if the storage of the address is empty.
	fn is_empty_storage(&self, address: H160) -> bool;
	/// Get original storage value of address at index.
	fn original_storage(&self, address: H160, index: H256) -> H256;

	/// Get the gas left value.
	fn gas_left(&self) -> U256;
	/// Get the gas price value.
	fn gas_price(&self) -> U256;
	/// Get execution origin.
	fn origin(&self) -> H160;
	/// Get environmental block hash.
	fn block_hash(&self, number: U256) -> H256;
	/// Get environmental block number.
	fn block_number(&self) -> U256;
	/// Get environmental coinbase.
	fn block_coinbase(&self) -> H160;
	/// Get environmental block timestamp.
	fn block_timestamp(&self) -> U256;
	/// Get environmental block difficulty.
	fn block_difficulty(&self) -> U256;
	/// Get environmental block randomness.
	fn block_randomness(&self) -> Option<H256>;
	/// Get environmental gas limit.
	fn block_gas_limit(&self) -> U256;
	/// Environmental block base fee.
	fn block_base_fee_per_gas(&self) -> U256;
	/// Get environmental chain ID.
	fn chain_id(&self) -> U256;

	/// Check whether an address exists.
	fn exists(&self, address: H160) -> bool;
	/// Check whether an address has already been deleted.
	fn deleted(&self, address: H160) -> bool;
	/// Checks if the address or (address, index) pair has been previously accessed
	/// (or set in `accessed_addresses` / `accessed_storage_keys` via an access list
	/// transaction).
	/// References:
	/// * <https://eips.ethereum.org/EIPS/eip-2929>
	/// * <https://eips.ethereum.org/EIPS/eip-2930>
	///
	/// # Errors
	/// Return `ExitError`
	fn is_cold(&mut self, address: H160, index: Option<H256>) -> bool;

	/// Set storage value of address at index.
	///
	/// # Errors
	/// Return `ExitError`
	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError>;
	/// Create a log owned by address with given topics and data.
	///
	/// # Errors
	/// Return `ExitError`
	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError>;
	/// Mark an address to be deleted, with funds transferred to target.
	///
	/// # Errors
	/// Return `ExitError`
	fn mark_delete(&mut self, address: H160, target: H160) -> Result<(), ExitError>;
	/// Invoke a create operation.
	fn create(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<u64>,
	) -> Capture<(ExitReason, Vec<u8>), Self::CreateInterrupt>;
	/// Feed in create feedback.
	///
	/// # Errors
	/// Return `ExitError`
	fn create_feedback(
		&mut self,
		#[allow(clippy::used_underscore_binding)] _feedback: Self::CreateFeedback,
	) -> Result<(), ExitError> {
		Ok(())
	}
	/// Invoke a call operation.
	fn call(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<u64>,
		is_static: bool,
		context: Context,
	) -> Capture<(ExitReason, Vec<u8>), Self::CallInterrupt>;
	/// Feed in call feedback.
	///
	/// # Errors
	/// Return `ExitError`
	fn call_feedback(
		&mut self,
		#[allow(clippy::used_underscore_binding)] _feedback: Self::CallFeedback,
	) -> Result<(), ExitError> {
		Ok(())
	}
	/// Handle other unknown external opcodes.
	///
	/// # Errors
	/// Return `ExitError`
	fn other(
		&mut self,
		opcode: Opcode,
		#[allow(clippy::used_underscore_binding)] _stack: &mut Machine,
	) -> Result<(), ExitError> {
		Err(ExitError::InvalidCode(opcode))
	}

	/// Records some associated `ExternalOperation`.
	///
	/// # Errors
	/// Return `ExitError`
	fn record_external_operation(&mut self, op: crate::ExternalOperation) -> Result<(), ExitError>;

	/// Returns `None` if `Cancun` is not enabled.
	/// CANCUN hard fork.
	/// [EIP-4844]: Shard Blob Transactions
	/// [EIP-7516]: BLOBBASEFEE instruction
	fn blob_base_fee(&self) -> Option<u128>;
	/// Get `blob_hash` from `blob_versioned_hashes` by index
	/// [EIP-4844]: BLOBHASH - https://eips.ethereum.org/EIPS/eip-4844#opcode-to-get-versioned-hashes
	fn get_blob_hash(&self, index: usize) -> Option<U256>;
	/// Set tstorage value of address at index.
	/// [EIP-1153]: Transient storage
	///
	/// # Errors
	/// Return `ExitError`
	fn tstore(&mut self, address: H160, index: H256, value: U256) -> Result<(), ExitError>;
	/// Get tstorage value of address at index.
	/// [EIP-1153]: Transient storage
	///
	/// # Errors
	/// Return `ExitError`
	fn tload(&mut self, address: H160, index: H256) -> Result<U256, ExitError>;
}
