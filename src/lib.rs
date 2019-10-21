use primitive_types::{H256, H160, U256};
use evm_gasometer::Gasometer;
use evm_core::Machine;

pub struct BlockContext {
	pub past_hashes: Vec<H256>,
	pub coinbase: H160,
	pub timestamp: u64,
	pub number: U256,
	pub difficulty: U256,
	pub gas_limit: usize,
}

pub enum ActionValue {
	Transfer(U256),
	Apparent(U256),
}

pub enum CallType {
	Call,
	CallCode,
	DelegateCall,
	StaticCall,
}

pub struct ActionContext {
	pub code_address: H160,
	pub address: H160,
	pub sender: H160,
	pub origin: H160,
	pub gas_limit: usize,
	pub gas_price: U256,
	pub value: ActionValue,
	pub call_type: CallType,
}

pub struct Config {

}

pub struct Runtime<'block, 'action, 'config, 'gconfig> {
	machine: Machine,
	gasometer: Gasometer<'gconfig>,
	block_context: &'block BlockContext,
	action_context: &'action ActionContext,
	config: &'config Config,
}

impl<'block, 'action, 'config, 'gconfig> Runtime<'block, 'action, 'config, 'gconfig> {

}

pub struct Interrupt {

}

pub enum Resolve<'block, 'action, 'config, 'gconfig> {
	Runtime(Runtime<'block, 'action, 'config, 'gconfig>),
	Interrupt(Box<Interrupt>),
}
