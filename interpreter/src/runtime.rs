use alloc::{rc::Rc, vec::Vec};

use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

use crate::error::ExitError;

/// Gas state.
pub trait GasState {
	fn gas(&self) -> U256;
}

/// Runtime state.
#[derive(Clone, Debug)]
pub struct RuntimeState {
	/// Runtime context.
	pub context: Context,
	/// Transaction context.
	pub transaction_context: Rc<TransactionContext>,
	/// Return data buffer.
	pub retbuf: Vec<u8>,
}

impl AsRef<Self> for RuntimeState {
	fn as_ref(&self) -> &Self {
		self
	}
}

impl AsMut<Self> for RuntimeState {
	fn as_mut(&mut self) -> &mut Self {
		self
	}
}

impl GasState for RuntimeState {
	fn gas(&self) -> U256 {
		U256::zero()
	}
}

/// Context of the runtime.
#[derive(Clone, Debug)]
pub struct Context {
	/// Execution address.
	pub address: H160,
	/// Caller of the EVM.
	pub caller: H160,
	/// Apparent value of the EVM.
	pub apparent_value: U256,
}

#[derive(Clone, Debug)]
pub struct TransactionContext {
	/// Gas price.
	pub gas_price: U256,
	/// Origin.
	pub origin: H160,
}

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

/// Log
#[derive(Clone, Debug)]
pub struct Log {
	pub address: H160,
	pub topics: Vec<H256>,
	pub data: Vec<u8>,
}

#[auto_impl::auto_impl(&, Box)]
pub trait RuntimeEnvironment {
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
}

#[auto_impl::auto_impl(&, Box)]
pub trait RuntimeBaseBackend {
	/// Get balance of address.
	fn balance(&self, address: H160) -> U256;
	/// Get code size of address.
	fn code_size(&self, address: H160) -> U256 {
		U256::from(self.code(address).len())
	}
	/// Get code hash of address.
	fn code_hash(&self, address: H160) -> H256 {
		H256::from_slice(&Keccak256::digest(&self.code(address)[..]))
	}
	/// Get code of address.
	fn code(&self, address: H160) -> Vec<u8>;
	/// Get storage value of address at index.
	fn storage(&self, address: H160, index: H256) -> H256;

	/// Check whether an address exists.
	fn exists(&self, address: H160) -> bool;

	/// Get the current nonce of an account.
	fn nonce(&self, address: H160) -> U256;
}

/// The distinguish between `RuntimeBaseBackend` and `RuntimeBackend` is for the implementation of
/// overlays.
pub trait RuntimeBackend: RuntimeBaseBackend {
	/// Get original storage value of address at index.
	fn original_storage(&self, address: H160, index: H256) -> H256;
	/// Check whether an address has already been deleted.
	fn deleted(&self, address: H160) -> bool;
	/// Checks if the address or (address, index) pair has been previously accessed.
	fn is_cold(&self, address: H160, index: Option<H256>) -> bool;
	fn is_hot(&self, address: H160, index: Option<H256>) -> bool {
		!self.is_cold(address, index)
	}

	/// Mark an address or (address, index) pair as hot.
	fn mark_hot(&mut self, address: H160, index: Option<H256>);
	/// Set storage value of address at index.
	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError>;
	/// Create a log owned by address with given topics and data.
	fn log(&mut self, log: Log) -> Result<(), ExitError>;
	/// Mark an address to be deleted.
	fn mark_delete(&mut self, address: H160);
	/// Fully delete storages of an account.
	fn reset_storage(&mut self, address: H160);
	/// Set code of an account.
	fn set_code(&mut self, address: H160, code: Vec<u8>) -> Result<(), ExitError>;
	/// Reset balance of an account.
	fn reset_balance(&mut self, address: H160);
	fn deposit(&mut self, target: H160, value: U256);
	fn withdrawal(&mut self, source: H160, value: U256) -> Result<(), ExitError>;
	/// Initiate a transfer.
	fn transfer(&mut self, transfer: Transfer) -> Result<(), ExitError> {
		self.withdrawal(transfer.source, transfer.value)?;
		self.deposit(transfer.target, transfer.value);
		Ok(())
	}
	/// Increase the nonce value.
	fn inc_nonce(&mut self, address: H160) -> Result<(), ExitError>;
}
