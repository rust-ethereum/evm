//! Runtime state and related traits.

use alloc::{rc::Rc, vec::Vec};

use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

use crate::error::ExitError;

/// Gas state.
pub trait GasState {
	/// Left gas.
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

/// Context of the transaction.
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
	/// Address
	pub address: H160,
	/// Topics
	pub topics: Vec<H256>,
	/// Log data
	pub data: Vec<u8>,
}

/// Identify if the origin of set_code() comes from a transaction or subcall.
#[derive(Clone, Debug)]
pub enum SetCodeOrigin {
	/// Comes from a transaction.
	Transaction,
	/// Comes from a subcall.
	Subcall(H160),
}

/// Runtime environment.
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

/// Runtime base backend. The immutable and limited part of [RuntimeBackend].
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
		if !self.is_empty(address) {
			H256::from_slice(&Keccak256::digest(&self.code(address)[..]))
		} else {
			H256::default()
		}
	}
	/// Get code of address.
	fn code(&self, address: H160) -> Vec<u8>;
	/// Get storage value of address at index.
	fn storage(&self, address: H160, index: H256) -> H256;
	/// Get transient storage value of address at index.
	fn transient_storage(&self, address: H160, index: H256) -> H256;

	/// Check whether an address exists. Used pre EIP161.
	fn exists(&self, address: H160) -> bool;
	/// Check whether an address is empty. Used after EIP161. Note that the meaning is opposite.
	fn is_empty(&self, address: H160) -> bool {
		self.balance(address) == U256::zero()
			&& self.code_size(address) == U256::zero()
			&& self.nonce(address) == U256::zero()
	}

	/// Get the current nonce of an account.
	fn nonce(&self, address: H160) -> U256;
}

/// For what reason is the address marked hot.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TouchKind {
	/// State change according to EIP-161.
	StateChange,
	/// Coinbase address.
	Coinbase,
	/// A normal access.
	Access,
}

/// The distinguish between `RuntimeBaseBackend` and `RuntimeBackend` is for the implementation of
/// overlays.
pub trait RuntimeBackend: RuntimeBaseBackend {
	/// Get original storage value of address at index.
	fn original_storage(&self, address: H160, index: H256) -> H256;
	/// Check whether an address has already been deleted.
	fn deleted(&self, address: H160) -> bool;
	/// Check whether an address has already been created in the transaction.
	fn created(&self, address: H160) -> bool;
	/// Checks if the address or (address, index) pair has been previously accessed.
	fn is_cold(&self, address: H160, index: Option<H256>) -> bool;
	/// Checks if the address is hot. Opposite of [RuntimeBackend::is_cold].
	fn is_hot(&self, address: H160, index: Option<H256>) -> bool {
		!self.is_cold(address, index)
	}

	/// Mark an address as hot.
	fn mark_hot(&mut self, address: H160, kind: TouchKind);
	/// Mark an (address, index) pair as hot.
	fn mark_storage_hot(&mut self, address: H160, index: H256);
	/// Set storage value of address at index.
	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError>;
	/// Set transient storage value of address at index, transient storage gets discarded after every transaction. (see EIP-1153)
	fn set_transient_storage(
		&mut self,
		address: H160,
		index: H256,
		value: H256,
	) -> Result<(), ExitError>;
	/// Create a log owned by address with given topics and data.
	fn log(&mut self, log: Log) -> Result<(), ExitError>;
	/// Mark an address to be deleted and its balance to be reset.
	fn mark_delete_reset(&mut self, address: H160);
	/// Mark an address as created in the current transaction.
	fn mark_create(&mut self, address: H160);
	/// Fully delete storages of an account.
	fn reset_storage(&mut self, address: H160);
	/// Set code of an account.
	fn set_code(
		&mut self,
		address: H160,
		code: Vec<u8>,
		origin: SetCodeOrigin,
	) -> Result<(), ExitError>;
	/// Deposit value into the target.
	fn deposit(&mut self, target: H160, value: U256);
	/// Withdrawal value from the source.
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
