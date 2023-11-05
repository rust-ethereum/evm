use super::Control;
use crate::{
	ExitException, ExitFatal, ExitSucceed, Handler, Machine, RuntimeState, RuntimeTrapData,
};
use alloc::vec::Vec;
use primitive_types::{H256, U256};
use sha3::{Digest, Keccak256};

pub fn sha3<S: AsRef<RuntimeState>, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
) -> Control<Td> {
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

pub fn chainid<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push_u256!(machine, handler.chain_id());

	Control::Continue
}

pub fn address<S: AsRef<RuntimeState>, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
) -> Control<Td> {
	let ret = H256::from(machine.state.as_ref().context.address);
	push!(machine, ret);

	Control::Continue
}

pub fn balance<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Td> {
	pop!(machine, address);
	try_or_fail!(handler.mark_hot(address.into(), None));
	push_u256!(machine, handler.balance(address.into()));

	Control::Continue
}

pub fn selfbalance<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push_u256!(
		machine,
		handler.balance(machine.state.as_ref().context.address)
	);

	Control::Continue
}

pub fn origin<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	let ret = H256::from(handler.origin());
	push!(machine, ret);

	Control::Continue
}

pub fn caller<S: AsRef<RuntimeState>, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
) -> Control<Td> {
	let ret = H256::from(machine.state.as_ref().context.caller);
	push!(machine, ret);

	Control::Continue
}

pub fn callvalue<S: AsRef<RuntimeState>, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
) -> Control<Td> {
	let mut ret = H256::default();
	machine
		.state
		.as_ref()
		.context
		.apparent_value
		.to_big_endian(&mut ret[..]);
	push!(machine, ret);

	Control::Continue
}

pub fn gasprice<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	let mut ret = H256::default();
	handler.gas_price().to_big_endian(&mut ret[..]);
	push!(machine, ret);

	Control::Continue
}

pub fn basefee<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	let mut ret = H256::default();
	handler.block_base_fee_per_gas().to_big_endian(&mut ret[..]);
	push!(machine, ret);

	Control::Continue
}

pub fn extcodesize<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Td> {
	pop!(machine, address);
	try_or_fail!(handler.mark_hot(address.into(), None));
	let code_size = handler.code_size(address.into());
	push_u256!(machine, code_size);

	Control::Continue
}

pub fn extcodehash<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Td> {
	pop!(machine, address);
	try_or_fail!(handler.mark_hot(address.into(), None));
	let code_hash = handler.code_hash(address.into());
	push!(machine, code_hash);

	Control::Continue
}

pub fn extcodecopy<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Td> {
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

pub fn returndatasize<S: AsRef<RuntimeState>, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
) -> Control<Td> {
	let size = U256::from(machine.state.as_ref().retbuf.len());
	push_u256!(machine, size);

	Control::Continue
}

pub fn returndatacopy<S: AsRef<RuntimeState>, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
) -> Control<Td> {
	pop_u256!(machine, memory_offset, data_offset, len);

	try_or_fail!(machine.memory.resize_offset(memory_offset, len));
	if data_offset
		.checked_add(len)
		.map(|l| l > U256::from(machine.state.as_ref().retbuf.len()))
		.unwrap_or(true)
	{
		return Control::Exit(ExitException::OutOfOffset.into());
	}

	match machine.memory.copy_large(
		memory_offset,
		data_offset,
		len,
		&machine.state.as_ref().retbuf,
	) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn blockhash<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	pop_u256!(machine, number);
	push!(machine, handler.block_hash(number));

	Control::Continue
}

pub fn coinbase<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push!(machine, handler.block_coinbase().into());
	Control::Continue
}

pub fn timestamp<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push_u256!(machine, handler.block_timestamp());
	Control::Continue
}

pub fn number<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push_u256!(machine, handler.block_number());
	Control::Continue
}

pub fn difficulty<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push_u256!(machine, handler.block_difficulty());
	Control::Continue
}

pub fn prevrandao<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	if let Some(rand) = handler.block_randomness() {
		push!(machine, rand);
		Control::Continue
	} else {
		difficulty(machine, handler)
	}
}

pub fn gaslimit<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push_u256!(machine, handler.block_gas_limit());
	Control::Continue
}

pub fn sload<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Td> {
	pop!(machine, index);
	try_or_fail!(handler.mark_hot(machine.state.as_ref().context.address, Some(index)));
	let value = handler.storage(machine.state.as_ref().context.address, index);
	push!(machine, value);

	Control::Continue
}

pub fn sstore<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Td> {
	pop!(machine, index, value);
	try_or_fail!(handler.mark_hot(machine.state.as_ref().context.address, Some(index)));

	match handler.set_storage(machine.state.as_ref().context.address, index, value) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn gas<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push_u256!(machine, handler.gas_left());

	Control::Continue
}

pub fn log<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	n: u8,
	handler: &mut H,
) -> Control<Td> {
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

	match handler.log(machine.state.as_ref().context.address, topics, data) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn suicide<S: AsRef<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Td> {
	pop!(machine, target);

	match handler.mark_delete(machine.state.as_ref().context.address, target.into()) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	}

	Control::Exit(ExitSucceed::Suicided.into())
}
