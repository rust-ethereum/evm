use core::convert::Infallible;
use std::collections::HashMap;
use primitive_types::{U256, H256, H160};
use sha3::{Keccak256, Digest};
use crate::{ExitError, Stack, ExternalOpcode, Opcode, Capture, Handler,
			Context, CreateScheme, Runtime, ExitReason, Resolve};

pub struct MemoryAccount {
	pub nonce: U256,
	pub balance: U256,
	pub storage: HashMap<H256, H256>,
	pub code: Vec<u8>,
}

pub struct MemoryContext {
	pub gas_price: U256,
	pub origin: H160,
	pub block_hashes: Vec<H256>,
	pub block_number: U256,
	pub block_coinbase: H160,
	pub block_timestamp: U256,
	pub block_difficulty: U256,
	pub block_gas_limit: U256,
}

pub struct MemoryLog {
	pub address: H160,
	pub topics: Vec<H256>,
	pub data: Vec<u8>,
}

pub struct MemoryExecutor<'ostate> {
	original_state: &'ostate HashMap<H160, MemoryAccount>,
	state: HashMap<H160, MemoryAccount>,
	context: MemoryContext,
}

impl<'ostate> MemoryExecutor<'ostate> {
	pub fn execute(&mut self, mut runtime: Runtime) -> (Runtime, ExitReason) {
		match runtime.run(self) {
			Capture::Exit((runtime, reason)) => (runtime, reason),
			Capture::Trap(_) => unreachable!("Trap is Infallible"),
		}
	}
}

impl<'ostate> Handler for MemoryExecutor<'ostate> {
	type CreateInterrupt = Infallible;
	type CreateFeedback = Infallible;
	type CallInterrupt = Infallible;
	type CallFeedback = Infallible;

	fn balance(&self, address: H160) -> U256 {
		self.state.get(&address).map(|v| v.balance).unwrap_or(U256::zero())
	}

	fn code_size(&self, address: H160) -> U256 {
		self.state.get(&address).map(|v| U256::from(v.code.len())).unwrap_or(U256::zero())
	}

	fn code_hash(&self, address: H160) -> H256 {
		self.state.get(&address).map(|v| {
			H256::from_slice(Keccak256::digest(&v.code).as_slice())
		}).unwrap_or(H256::default())
	}

	fn code(&self, address: H160) -> Vec<u8> {
		self.state.get(&address).map(|v| v.code.clone()).unwrap_or(Vec::new())
	}

	fn storage(&self, address: H160, index: H256) -> H256 {
		self.state.get(&address)
			.map(|v| v.storage.get(&index).cloned().unwrap_or(H256::default()))
			.unwrap_or(H256::default())
	}

	fn original_storage(&self, address: H160, index: H256) -> H256 {
		self.original_state.get(&address)
			.map(|v| v.storage.get(&index).cloned().unwrap_or(H256::default()))
			.unwrap_or(H256::default())
	}

	fn gas_left(&self) -> U256 { unimplemented!() }
	fn gas_price(&self) -> U256 { self.context.gas_price }
	fn origin(&self) -> H160 { self.context.origin }
	fn block_hash(&self, number: U256) -> H256 {
		if number >= self.context.block_number ||
			self.context.block_number - number - U256::one() >= U256::from(self.context.block_hashes.len())
		{
			H256::default()
		} else {
			let index = (self.context.block_number - number - U256::one()).as_usize();
			self.context.block_hashes[index]
		}
	}
	fn block_number(&self) -> U256 { self.context.block_number }
	fn block_coinbase(&self) -> H160 { self.context.block_coinbase }
	fn block_timestamp(&self) -> U256 { self.context.block_timestamp }
	fn block_difficulty(&self) -> U256 { self.context.block_difficulty }
	fn block_gas_limit(&self) -> U256 { self.context.block_gas_limit }

	fn create_address(&self, address: H160, scheme: CreateScheme) -> H160 { unimplemented!() }
	fn exists(&self, address: H160) -> bool { self.state.get(&address).is_some() }
	fn deleted(&self, address: H160) -> bool { unimplemented!() }

	fn is_recoverable(&self) -> bool { true }

	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError> {
		unimplemented!()
	}

	fn log(&mut self, address: H160, topcis: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError> {
		unimplemented!()
	}

	fn transfer(&mut self, source: H160, target: H160, value: U256) -> Result<(), ExitError> {
		unimplemented!()
	}

	fn mark_delete(&mut self, address: H160) -> Result<(), ExitError> {
		unimplemented!()
	}

	fn create(
		&mut self,
		address: H160,
		init_code: Vec<u8>,
		target_gas: Option<usize>,
		context: Context,
	) -> Result<Capture<H160, Self::CreateInterrupt>, ExitError> {
		unimplemented!()
	}

	fn call(
		&mut self,
		code_address: H160,
		input: Vec<u8>,
		target_gas: Option<usize>,
		is_static: bool,
		context: Context,
	) -> Result<Capture<Vec<u8>, Self::CallInterrupt>, ExitError> {
		unimplemented!()
	}

	fn pre_validate(
		&mut self,
		opcode: Result<Opcode, ExternalOpcode>,
		stack: &Stack
	) -> Result<(), ExitError> {
		unimplemented!()
	}
}
