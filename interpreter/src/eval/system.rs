use super::Control;
use crate::{ExitException, ExitFatal, ExitSucceed, Handler, RuntimeMachine};
use alloc::vec::Vec;
use primitive_types::{H256, U256};
use sha3::{Digest, Keccak256};

pub fn sha3(machine: &mut RuntimeMachine) -> Control {
	pop_u256!(machine, from, len);

	try_or_fail!(machine.memory.resize_offset(from, len));
	let data = if len == U256::zero() {
		Vec::new()
	} else {
		let from = as_usize_or_fail!(from);
		let len = as_usize_or_fail!(len);

		machine.memory.get(from, len)
	};

	let ret = Keccak256::digest(data.as_slice());
	push!(machine, H256::from_slice(ret.as_slice()));

	Control::Continue
}

pub fn chainid<H: Handler>(machine: &mut RuntimeMachine, handler: &H) -> Control {
	push_u256!(machine, handler.chain_id());

	Control::Continue
}

pub fn address(machine: &mut RuntimeMachine) -> Control {
	let ret = H256::from(machine.state.context.address);
	push!(machine, ret);

	Control::Continue
}

pub fn balance<H: Handler>(machine: &mut RuntimeMachine, handler: &mut H) -> Control {
	pop!(machine, address);
	try_or_fail!(handler.mark_hot(address.into(), None));
	push_u256!(machine, handler.balance(address.into()));

	Control::Continue
}

pub fn selfbalance<H: Handler>(machine: &mut RuntimeMachine, handler: &H) -> Control {
	push_u256!(machine, handler.balance(machine.state.context.address));

	Control::Continue
}

pub fn origin<H: Handler>(machine: &mut RuntimeMachine, handler: &H) -> Control {
	let ret = H256::from(handler.origin());
	push!(machine, ret);

	Control::Continue
}

pub fn caller(machine: &mut RuntimeMachine) -> Control {
	let ret = H256::from(machine.state.context.caller);
	push!(machine, ret);

	Control::Continue
}

pub fn callvalue(machine: &mut RuntimeMachine) -> Control {
	let mut ret = H256::default();
	machine
		.state
		.context
		.apparent_value
		.to_big_endian(&mut ret[..]);
	push!(machine, ret);

	Control::Continue
}

pub fn gasprice<H: Handler>(machine: &mut RuntimeMachine, handler: &H) -> Control {
	let mut ret = H256::default();
	handler.gas_price().to_big_endian(&mut ret[..]);
	push!(machine, ret);

	Control::Continue
}

pub fn basefee<H: Handler>(machine: &mut RuntimeMachine, handler: &H) -> Control {
	let mut ret = H256::default();
	handler.block_base_fee_per_gas().to_big_endian(&mut ret[..]);
	push!(machine, ret);

	Control::Continue
}

pub fn extcodesize<H: Handler>(machine: &mut RuntimeMachine, handler: &mut H) -> Control {
	pop!(machine, address);
	try_or_fail!(handler.mark_hot(address.into(), None));
	let code_size = handler.code_size(address.into());
	push_u256!(machine, code_size);

	Control::Continue
}

pub fn extcodehash<H: Handler>(machine: &mut RuntimeMachine, handler: &mut H) -> Control {
	pop!(machine, address);
	try_or_fail!(handler.mark_hot(address.into(), None));
	let code_hash = handler.code_hash(address.into());
	push!(machine, code_hash);

	Control::Continue
}

pub fn extcodecopy<H: Handler>(machine: &mut RuntimeMachine, handler: &mut H) -> Control {
	pop!(machine, address);
	pop_u256!(machine, memory_offset, code_offset, len);

	try_or_fail!(handler.mark_hot(address.into(), None));
	try_or_fail!(machine.memory.resize_offset(memory_offset, len));

	let code = handler.code(address.into());
	match machine
		.memory
		.copy_large(memory_offset, code_offset, len, &code)
	{
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	};

	Control::Continue
}

pub fn returndatasize(machine: &mut RuntimeMachine) -> Control {
	let size = U256::from(machine.state.retbuf.len());
	push_u256!(machine, size);

	Control::Continue
}

pub fn returndatacopy(machine: &mut RuntimeMachine) -> Control {
	pop_u256!(machine, memory_offset, data_offset, len);

	try_or_fail!(machine.memory.resize_offset(memory_offset, len));
	if data_offset
		.checked_add(len)
		.map(|l| l > U256::from(machine.state.retbuf.len()))
		.unwrap_or(true)
	{
		return Control::Exit(ExitException::OutOfOffset.into());
	}

	match machine
		.memory
		.copy_large(memory_offset, data_offset, len, &machine.state.retbuf)
	{
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn blockhash<H: Handler>(machine: &mut RuntimeMachine, handler: &H) -> Control {
	pop_u256!(machine, number);
	push!(machine, handler.block_hash(number));

	Control::Continue
}

pub fn coinbase<H: Handler>(machine: &mut RuntimeMachine, handler: &H) -> Control {
	push!(machine, handler.block_coinbase().into());
	Control::Continue
}

pub fn timestamp<H: Handler>(machine: &mut RuntimeMachine, handler: &H) -> Control {
	push_u256!(machine, handler.block_timestamp());
	Control::Continue
}

pub fn number<H: Handler>(machine: &mut RuntimeMachine, handler: &H) -> Control {
	push_u256!(machine, handler.block_number());
	Control::Continue
}

pub fn difficulty<H: Handler>(machine: &mut RuntimeMachine, handler: &H) -> Control {
	push_u256!(machine, handler.block_difficulty());
	Control::Continue
}

pub fn prevrandao<H: Handler>(machine: &mut RuntimeMachine, handler: &H) -> Control {
	if let Some(rand) = handler.block_randomness() {
		push!(machine, rand);
		Control::Continue
	} else {
		difficulty(machine, handler)
	}
}

pub fn gaslimit<H: Handler>(machine: &mut RuntimeMachine, handler: &H) -> Control {
	push_u256!(machine, handler.block_gas_limit());
	Control::Continue
}

pub fn sload<H: Handler>(machine: &mut RuntimeMachine, handler: &mut H) -> Control {
	pop!(machine, index);
	try_or_fail!(handler.mark_hot(machine.state.context.address, Some(index)));
	let value = handler.storage(machine.state.context.address, index);
	push!(machine, value);

	Control::Continue
}

pub fn sstore<H: Handler>(machine: &mut RuntimeMachine, handler: &mut H) -> Control {
	pop!(machine, index, value);
	try_or_fail!(handler.mark_hot(machine.state.context.address, Some(index)));

	match handler.set_storage(machine.state.context.address, index, value) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn gas<H: Handler>(machine: &mut RuntimeMachine, handler: &H) -> Control {
	push_u256!(machine, handler.gas_left());

	Control::Continue
}

pub fn log<H: Handler>(machine: &mut RuntimeMachine, n: u8, handler: &mut H) -> Control {
	pop_u256!(machine, offset, len);

	try_or_fail!(machine.memory.resize_offset(offset, len));
	let data = if len == U256::zero() {
		Vec::new()
	} else {
		let offset = as_usize_or_fail!(offset);
		let len = as_usize_or_fail!(len);

		machine.memory.get(offset, len)
	};

	let mut topics = Vec::new();
	for _ in 0..(n as usize) {
		match machine.stack.pop() {
			Ok(value) => {
				topics.push(value);
			}
			Err(e) => return Control::Exit(e.into()),
		}
	}

	match handler.log(machine.state.context.address, topics, data) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn suicide<H: Handler>(machine: &mut RuntimeMachine, handler: &mut H) -> Control {
	pop!(machine, target);

	match handler.mark_delete(machine.state.context.address, target.into()) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	}

	Control::Exit(ExitSucceed::Suicided.into())
}
