use primitive_types::{H160, H256, U256};
use crate::{Capture, Stack, ExitError, Opcode, ExternalOpcode,
			CreateScheme, Context, Machine};

pub trait Handler {
	type CreateInterrupt;
	type CreateFeedback;
	type CallInterrupt;
	type CallFeedback;

	fn balance(&self, address: H160) -> U256;
	fn code_size(&self, address: H160) -> U256;
	fn code_hash(&self, address: H160) -> H256;
	fn code(&self, address: H160) -> Vec<u8>;
	fn storage(&self, address: H160, index: H256) -> H256;
	fn original_storage(&self, address: H160, index: H256) -> H256;

	fn gas_left(&self) -> U256;
	fn gas_price(&self) -> U256;
	fn origin(&self) -> H160;
	fn block_hash(&self, number: U256) -> H256;
	fn block_number(&self) -> U256;
	fn block_coinbase(&self) -> H160;
	fn block_timestamp(&self) -> U256;
	fn block_difficulty(&self) -> U256;
	fn block_gas_limit(&self) -> U256;

	fn create_address(&self, address: H160, scheme: CreateScheme) -> H160;
	fn exists(&self, address: H160) -> bool;
	fn deleted(&self, address: H160) -> bool;

	fn is_recoverable(&self) -> bool;
	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError>;
	fn log(&mut self, address: H160, topcis: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError>;
	fn transfer(&mut self, source: H160, target: H160, value: U256) -> Result<(), ExitError>;
	fn mark_delete(&mut self, address: H160) -> Result<(), ExitError>;
	fn create(
		&mut self,
		address: H160,
		init_code: Vec<u8>,
		target_gas: Option<usize>,
		context: Context,
	) -> Result<Capture<(), Self::CreateInterrupt>, ExitError>;
	fn create_feedback(
		&mut self,
		_feedback: Self::CreateFeedback
	) -> Result<(), ExitError> {
		Ok(())
	}
	fn call(
		&mut self,
		code_address: H160,
		input: Vec<u8>,
		target_gas: Option<usize>,
		is_static: bool,
		context: Context,
	) -> Result<Capture<Vec<u8>, Self::CallInterrupt>, ExitError>;
	fn call_feedback(
		&mut self,
		_feedback: Self::CallFeedback
	) -> Result<(), ExitError> {
		Ok(())
	}

	fn pre_validate(
		&mut self,
		context: &Context,
		opcode: Result<Opcode, ExternalOpcode>,
		stack: &Stack
	) -> Result<(), ExitError>;

	fn other(
		&mut self,
		_opcode: u8,
		_stack: &mut Machine
	) -> Result<(), ExitError> {
		Err(ExitError::OutOfGas)
	}
}
