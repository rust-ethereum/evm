use crate::{ExitError, Opcode};
use primitive_types::{H160, H256, U256};

/// Runtime state.
#[derive(Clone, Debug)]
pub struct RuntimeState {
	/// Runtime context.
	pub context: Context,
	/// Return data buffer.
	pub retbuf: Vec<u8>,
}

impl AsRef<RuntimeState> for RuntimeState {
	fn as_ref(&self) -> &Self {
		self
	}
}

impl AsMut<RuntimeState> for RuntimeState {
	fn as_mut(&mut self) -> &mut Self {
		self
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

pub trait CallCreateTrap: Sized {
	fn call_create_trap(opcode: Opcode) -> Self;
}

impl CallCreateTrap for Opcode {
	fn call_create_trap(opcode: Opcode) -> Self {
		opcode
	}
}

pub trait RuntimeBackend {
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
	/// Get the gas price value.
	fn gas_price(&self) -> U256;
	/// Get execution origin.
	fn origin(&self) -> H160;

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
	/// Get original storage value of address at index.
	fn original_storage(&self, address: H160, index: H256) -> H256;

	/// Check whether an address exists.
	fn exists(&self, address: H160) -> bool;
	/// Check whether an address has already been deleted.
	fn deleted(&self, address: H160) -> bool;
	/// Checks if the address or (address, index) pair has been previously accessed.
	fn is_cold(&self, address: H160, index: Option<H256>) -> bool;
	/// Mark an address or (address, index) pair as hot.
	fn mark_hot(&mut self, address: H160, index: Option<H256>) -> Result<(), ExitError>;
	/// Set storage value of address at index.
	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError>;

	/// Create a log owned by address with given topics and data.
	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError>;
	/// Mark an address to be deleted, with funds transferred to target.
	fn mark_delete(&mut self, address: H160, target: H160) -> Result<(), ExitError>;
}

pub trait RuntimeGasometer {
	/// Get the gas left value.
	fn gas(&self) -> U256;
}

/// Handler trait for runtime.
///
/// The handler is generally expected to be a `(backend, gasometer)` tuple, with extensions added
/// to `backend`.
pub trait RuntimeHandler: RuntimeBackend + RuntimeGasometer {}

impl<'b, 'g, G: RuntimeGasometer, H: RuntimeBackend> RuntimeBackend for (&'b mut G, &'g mut H) {
	fn block_hash(&self, number: U256) -> H256 {
		self.1.block_hash(number)
	}
	fn block_number(&self) -> U256 {
		self.1.block_number()
	}
	fn block_coinbase(&self) -> H160 {
		self.1.block_coinbase()
	}
	fn block_timestamp(&self) -> U256 {
		self.1.block_timestamp()
	}
	fn block_difficulty(&self) -> U256 {
		self.1.block_difficulty()
	}
	fn block_randomness(&self) -> Option<H256> {
		self.1.block_randomness()
	}
	fn block_gas_limit(&self) -> U256 {
		self.1.block_gas_limit()
	}
	fn block_base_fee_per_gas(&self) -> U256 {
		self.1.block_base_fee_per_gas()
	}
	fn chain_id(&self) -> U256 {
		self.1.chain_id()
	}
	fn gas_price(&self) -> U256 {
		self.1.gas_price()
	}
	fn origin(&self) -> H160 {
		self.1.origin()
	}

	fn balance(&self, address: H160) -> U256 {
		self.1.balance(address)
	}
	fn code_size(&self, address: H160) -> U256 {
		self.1.code_size(address)
	}
	fn code_hash(&self, address: H160) -> H256 {
		self.1.code_hash(address)
	}
	fn code(&self, address: H160) -> Vec<u8> {
		self.1.code(address)
	}
	fn storage(&self, address: H160, index: H256) -> H256 {
		self.1.storage(address, index)
	}
	fn original_storage(&self, address: H160, index: H256) -> H256 {
		self.1.original_storage(address, index)
	}

	fn exists(&self, address: H160) -> bool {
		self.1.exists(address)
	}
	fn deleted(&self, address: H160) -> bool {
		self.1.deleted(address)
	}
	fn is_cold(&self, address: H160, index: Option<H256>) -> bool {
		self.1.is_cold(address, index)
	}
	fn mark_hot(&mut self, address: H160, index: Option<H256>) -> Result<(), ExitError> {
		self.1.mark_hot(address, index)
	}
	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError> {
		self.1.set_storage(address, index, value)
	}

	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError> {
		self.1.log(address, topics, data)
	}
	fn mark_delete(&mut self, address: H160, target: H160) -> Result<(), ExitError> {
		self.1.mark_delete(address, target)
	}
}

impl<'b, 'g, G: RuntimeGasometer, H: RuntimeBackend> RuntimeGasometer for (&'b mut G, &'g mut H) {
	fn gas(&self) -> U256 {
		self.0.gas()
	}
}

impl<'b, 'g, G: RuntimeGasometer, H: RuntimeBackend> RuntimeHandler for (&'b mut G, &'g mut H) {}
