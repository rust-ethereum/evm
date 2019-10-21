mod eval;

pub use evm_core::*;
pub use evm_gasometer::*;

use primitive_types::{H256, H160, U256};

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

impl ActionValue {
	pub fn value(&self) -> &U256 {
		match self {
			ActionValue::Transfer(val) => val,
			ActionValue::Apparent(val) => val,
		}
	}
}

pub enum CallType {
	Call,
	CallCode,
	DelegateCall,
	StaticCall,
}

pub struct ActionContext {
	pub address: H160,
	pub caller: H160,
	pub origin: H160,
	pub gas_price: U256,
	pub value: ActionValue,
	pub call_type: CallType,
}

pub struct Config {

}

pub struct Runtime<'block, 'action, 'config> {
	machine: Machine,
	status: Result<(), ExitReason>,
	return_data_buffer: Vec<u8>,
	block_context: &'block BlockContext,
	action_context: &'action ActionContext,
	config: &'config Config,
}

impl<'block, 'action, 'config> Runtime<'block, 'action, 'config> {
	pub fn step(
		mut self
	) -> Result<Self, Capture<(Self, ExitReason), Resolve<'block, 'action, 'config>>> {
		match self.status.clone() {
			Ok(()) => (),
			Err(exit) => return Err(Capture::Exit((self, exit))),
		}

		// TODO: Add gasometer here.

		match self.machine.step() {
			Ok(()) => Ok(self),
			Err(Capture::Exit(exit)) => {
				self.status = Err(exit);
				Err(Capture::Exit((self, exit)))
			},
			Err(Capture::Trap(opcode)) => {
				match eval::eval(&mut self, opcode) {
					eval::Control::Continue => Ok(self),
					eval::Control::Interrupt(interrupt) => {
						let resolve = Resolve::from_interrupt(self, interrupt);
						Err(Capture::Trap(resolve))
					},
					eval::Control::Exit(exit) => {
						self.status = Err(exit);
						Err(Capture::Exit((self, exit)))
					},
				}
			},
		}
	}

	pub fn run(
		mut self
	) -> Capture<(Self, ExitReason), Resolve<'block, 'action, 'config>> {
		let mut current = self;

		loop {
			match current.step() {
				Ok(value) => {
					current = value
				},
				Err(capture) => return capture,
			}
		}
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Interrupt {
	ExtBalance { address: H160 },
	ExtCodeSize { address: H160 },
	ExtCodeHash { address: H160 },
	ExtCodeCopy {
		address: H160,
		memory_offset: U256,
		code_offset: U256,
		len: U256,
	},
	SLoad { index: H256 },
	SStore { index: H256, value: H256 },
	Log { topics: Vec<H256>, data: Vec<u8> },
}

pub enum Resolve<'block, 'action, 'config> {
	ExtBalance(ResolveExtBalance<'block, 'action, 'config>),
	ExtCodeSize(ResolveExtCodeSize<'block, 'action, 'config>),
	ExtCodeHash(ResolveExtCodeHash<'block, 'action, 'config>),
	ExtCodeCopy(ResolveExtCodeCopy<'block, 'action, 'config>),
	SLoad(ResolveSLoad<'block, 'action, 'config>),
	SStore(ResolveSStore<'block, 'action, 'config>),
	Log(ResolveLog<'block, 'action, 'config>),
}

impl<'block, 'action, 'config> Resolve<'block, 'action, 'config> {
	pub fn from_interrupt(
		runtime: Runtime<'block, 'action, 'config>,
		interrupt: Interrupt,
	) -> Resolve<'block, 'action, 'config> {
		match interrupt {
			Interrupt::ExtBalance { address } =>
				Resolve::ExtBalance(ResolveExtBalance {
					runtime, address,
				}),
			Interrupt::ExtCodeSize { address } =>
				Resolve::ExtCodeSize(ResolveExtCodeSize {
					runtime, address,
				}),
			Interrupt::ExtCodeHash { address } =>
				Resolve::ExtCodeHash(ResolveExtCodeHash {
					runtime, address,
				}),
			Interrupt::ExtCodeCopy { address, memory_offset, code_offset, len } =>
				Resolve::ExtCodeCopy(ResolveExtCodeCopy {
					runtime, address, memory_offset, code_offset, len
				}),
			Interrupt::SLoad { index } =>
				Resolve::SLoad(ResolveSLoad {
					runtime, index
				}),
			Interrupt::SStore { index, value } =>
				Resolve::SStore(ResolveSStore {
					runtime, index, value
				}),
			Interrupt::Log { topics, data } =>
				Resolve::Log(ResolveLog {
					runtime, topics, data
				}),
		}
	}
}

pub struct ResolveExtBalance<'block, 'action, 'config> {
	runtime: Runtime<'block, 'action, 'config>,
	address: H160,
}

impl<'block, 'action, 'config> ResolveExtBalance<'block, 'action, 'config> {
	pub fn resolve(mut self, balance: U256) -> Runtime<'block, 'action, 'config> {
		let mut value = H256::default();
		balance.to_big_endian(&mut value[..]);
		self.runtime.machine.stack_mut().set(0, value)
			.expect("Interrupt had at least one push; qed");
		self.runtime
	}
}

pub struct ResolveExtCodeSize<'block, 'action, 'config> {
	runtime: Runtime<'block, 'action, 'config>,
	address: H160,
}

impl<'block, 'action, 'config> ResolveExtCodeSize<'block, 'action, 'config> {
	pub fn resolve(mut self, size: U256) -> Runtime<'block, 'action, 'config> {
		let mut value = H256::default();
		size.to_big_endian(&mut value[..]);
		self.runtime.machine.stack_mut().set(0, value)
			.expect("Interrupt had at least one push; qed");
		self.runtime
	}
}

pub struct ResolveExtCodeHash<'block, 'action, 'config> {
	runtime: Runtime<'block, 'action, 'config>,
	address: H160,
}

impl<'block, 'action, 'config> ResolveExtCodeHash<'block, 'action, 'config> {
	pub fn resolve(mut self, hash: H256) -> Runtime<'block, 'action, 'config> {
		self.runtime.machine.stack_mut().set(0, hash)
			.expect("Interrupt had at least one push; qed");
		self.runtime
	}
}

pub struct ResolveExtCodeCopy<'block, 'action, 'config> {
	runtime: Runtime<'block, 'action, 'config>,
	address: H160,
	memory_offset: U256,
	code_offset: U256,
	len: U256,
}

impl<'block, 'action, 'config> ResolveExtCodeCopy<'block, 'action, 'config> {
	pub fn resolve(mut self, code: &[u8]) -> Runtime<'block, 'action, 'config> {
		self.runtime.machine.memory_mut().copy_large(
			self.memory_offset, self.code_offset, self.len, code
		).expect("Interrupt had tried copy; qed");
		self.runtime
	}
}

pub struct ResolveSLoad<'block, 'action, 'config> {
	runtime: Runtime<'block, 'action, 'config>,
	index: H256,
}

impl<'block, 'action, 'config> ResolveSLoad<'block, 'action, 'config> {
	pub fn resolve(mut self, value: H256) -> Runtime<'block, 'action, 'config> {
		self.runtime.machine.stack_mut().set(0, value)
			.expect("Interrupt had at least one push; qed");
		self.runtime
	}
}

pub struct ResolveSStore<'block, 'action, 'config> {
	runtime: Runtime<'block, 'action, 'config>,
	index: H256,
	value: H256,
}

impl<'block, 'action, 'config> ResolveSStore<'block, 'action, 'config> {
	pub fn resolve(mut self) -> Runtime<'block, 'action, 'config> {
		self.runtime
	}
}

pub struct ResolveLog<'block, 'action, 'config> {
	runtime: Runtime<'block, 'action, 'config>,
	topics: Vec<H256>,
	data: Vec<u8>,
}

impl<'block, 'action, 'config> ResolveLog<'block, 'action, 'config> {
	pub fn resolve(mut self) -> Runtime<'block, 'action, 'config> {
		self.runtime
	}
}
