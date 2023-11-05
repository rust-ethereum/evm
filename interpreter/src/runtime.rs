use crate::{ExitError, ExitResult, Machine, Trap};
use core::cmp::min;
use primitive_types::{H160, H256, U256};

pub enum RuntimeTrapData {
	Call(Box<RuntimeCallTrapData>),
	Create(Box<RuntimeCreateTrapData>),
}

pub struct RuntimeCallTrapData {
	pub target: H160,
	pub transfer: Option<Transfer>,
	pub input: Vec<u8>,
	pub gas: U256,
	pub is_static: bool,
	pub out_offset: U256,
	pub out_len: U256,
	pub context: Context,
}

pub struct RuntimeCreateTrapData {
	pub scheme: CreateScheme,
	pub value: U256,
	pub code: Vec<u8>,
}

pub enum RuntimeTrap<S> {
	Call(RuntimeCallTrap<S>),
	Create(RuntimeCreateTrap<S>),
}

impl<S: AsRef<RuntimeState>> Trap<S> for RuntimeTrap<S> {
	type Data = RuntimeTrapData;

	fn from_data(data: RuntimeTrapData, machine: Machine<S>) -> Self {
		match data {
			RuntimeTrapData::Call(data) => Self::Call(RuntimeCallTrap { data, machine }),
			RuntimeTrapData::Create(data) => Self::Create(RuntimeCreateTrap { data, machine }),
		}
	}
}

pub struct RuntimeCallTrap<S> {
	data: Box<RuntimeCallTrapData>,
	machine: Machine<S>,
}

impl<S: AsMut<RuntimeState>> RuntimeCallTrap<S> {
	pub fn data(&self) -> &RuntimeCallTrapData {
		self.data.as_ref()
	}

	pub fn feedback(
		mut self,
		reason: ExitResult,
		retbuf: Vec<u8>,
	) -> (Machine<S>, Result<(), ExitError>) {
		let target_len = min(self.data.out_len, U256::from(retbuf.len()));
		let out_offset = self.data.out_offset;

		let ret = self.machine.perform(|machine| match reason {
			Ok(_) => {
				match machine
					.memory
					.copy_large(out_offset, U256::zero(), target_len, &retbuf[..])
				{
					Ok(()) => {
						let mut value = H256::default();
						U256::one().to_big_endian(&mut value[..]);
						machine.stack.push(value)?;

						Ok(())
					}
					Err(_) => {
						machine.stack.push(H256::default())?;

						Ok(())
					}
				}
			}
			Err(ExitError::Reverted) => {
				machine.stack.push(H256::default())?;

				let _ =
					machine
						.memory
						.copy_large(out_offset, U256::zero(), target_len, &retbuf[..]);

				Ok(())
			}
			Err(ExitError::Exception(_)) => {
				machine.stack.push(H256::default())?;

				Ok(())
			}
			Err(ExitError::Fatal(e)) => {
				machine.stack.push(H256::default())?;

				Err(e.into())
			}
		});

		match ret {
			Ok(()) => {
				self.machine.state.as_mut().retbuf = retbuf;
				(self.machine, Ok(()))
			}
			Err(e) => (self.machine, Err(e)),
		}
	}
}

pub struct RuntimeCreateTrap<S> {
	data: Box<RuntimeCreateTrapData>,
	machine: Machine<S>,
}

impl<S: AsMut<RuntimeState>> RuntimeCreateTrap<S> {
	pub fn data(&self) -> &RuntimeCreateTrapData {
		self.data.as_ref()
	}

	pub fn feedback(
		mut self,
		reason: Result<H160, ExitError>,
		retbuf: Vec<u8>,
	) -> (Machine<S>, Result<(), ExitError>) {
		let ret = self.machine.perform(|machine| match reason {
			Ok(address) => {
				machine.stack.push(address.into())?;
				Ok(())
			}
			Err(ExitError::Reverted) => {
				machine.stack.push(H256::default())?;
				Ok(())
			}
			Err(ExitError::Exception(_)) => {
				machine.stack.push(H256::default())?;
				Ok(())
			}
			Err(ExitError::Fatal(e)) => {
				machine.stack.push(H256::default())?;
				Err(e.into())
			}
		});

		match ret {
			Ok(()) => {
				self.machine.state.as_mut().retbuf = retbuf;
				(self.machine, Ok(()))
			}
			Err(e) => (self.machine, Err(e)),
		}
	}
}

/// Create scheme.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CreateScheme {
	/// Legacy create scheme of `CREATE`.
	Legacy {
		/// Caller of the create.
		caller: H160,
	},
	/// Create scheme of `CREATE2`.
	Create2 {
		/// Caller of the create.
		caller: H160,
		/// Code hash.
		code_hash: H256,
		/// Salt.
		salt: H256,
	},
	/// Create at a fixed location.
	Fixed(H160),
}

/// Call scheme.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CallScheme {
	/// `CALL`
	Call,
	/// `CALLCODE`
	CallCode,
	/// `DELEGATECALL`
	DelegateCall,
	/// `STATICCALL`
	StaticCall,
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

/// Runtime state.
#[derive(Clone, Debug)]
pub struct RuntimeState {
	/// Runtime context.
	pub context: Context,
	/// Return data buffer.
	pub retbuf: Vec<u8>,
}

impl AsRef<RuntimeState> for RuntimeState {
	fn as_ref(&self) -> &RuntimeState {
		self
	}
}

impl AsMut<RuntimeState> for RuntimeState {
	fn as_mut(&mut self) -> &mut RuntimeState {
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

pub trait RuntimeBackend {
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

pub trait RuntimeFullBackend: RuntimeBackend {
	/// Fully delete storages of an account.
	fn reset_storage(&mut self, address: H160);
	/// Set code of an account.
	fn set_code(&mut self, address: H160, code: Vec<u8>);
	/// Reset balance of an account.
	fn reset_balance(&mut self, address: H160);
	/// Initiate a transfer.
	fn transfer(&mut self, transfer: Transfer) -> Result<(), ExitError>;
	/// Increase the nonce value.
	fn inc_nonce(&mut self, address: H160) -> Result<(), ExitError>;
}

pub trait RuntimeEnvironmentalBackend {
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
}

pub trait RuntimeGasometerBackend {
	/// Get the gas left value.
	fn gas_left(&self) -> U256;
}

/// EVM context handler.
pub trait Handler: RuntimeBackend + RuntimeEnvironmentalBackend + RuntimeGasometerBackend {}
