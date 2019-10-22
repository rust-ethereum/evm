use primitive_types::{H160, H256, U256};
use crate::{Capture, Stack, MultiError, ExitError, Opcode, ExternalOpcode,
			CallScheme, CreateScheme, Context};

pub trait Handler {
	type CreateInterrupt;
	type CallInterrupt;

	fn ext_balance(&self, address: H160) -> U256;
	fn ext_code_size(&self, address: H160) -> U256;
	fn ext_code_hash(&self, address: H160) -> H256;
	fn ext_code(&self, address: H160) -> Vec<u8>;
	fn gas_left(&self) -> U256;
	fn storage(&self, index: H256) -> H256;
	fn original_storage(&self, index: H256) -> H256;
	fn block_hash(&self, number: U256) -> H256;
	fn block_number(&self) -> U256;
	fn block_coinbase(&self) -> H160;
	fn block_timestamp(&self) -> U256;
	fn block_difficulty(&self) -> U256;
	fn block_gas_limit(&self) -> U256;

	fn set_storage(&mut self, index: H256, value: H256) -> Result<(), ExitError>;
	fn log(&mut self, topcis: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError>;
	fn transfer(&mut self, source: H160, target: H160, value: Option<U256>) -> Result<(), ExitError>;
	fn mark_delete(&mut self, address: H160) -> Result<(), ExitError>;
	fn create(
		&mut self,
		scheme: CreateScheme,
		gas_limit: usize,
		init_code: &[u8],
		context: Context
	) -> Result<Capture<H160, Self::CreateInterrupt>, MultiError<ExitError>>;
	fn call(
		&mut self,
		scheme: CallScheme,
		gas_limit: usize,
		action_context: Context
	) -> Result<Capture<H160, Self::CallInterrupt>, MultiError<ExitError>>;

	fn pre_validate(
		&mut self,
		opcode: Result<Opcode, ExternalOpcode>,
		stack: &Stack
	) -> Result<(), ExitError>;
}
