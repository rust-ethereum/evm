use alloc::vec::Vec;

use primitive_types::{H256, U256};
use sha3::{Digest, Keccak256};

use crate::{
	error::{ExitException, ExitFatal, ExitSucceed},
	etable::Control,
	machine::Machine,
	runtime::{GasState, Log, RuntimeBackend, RuntimeEnvironment, RuntimeState, Transfer},
};

pub fn sha3<S: AsRef<RuntimeState>, Tr>(machine: &mut Machine<S>) -> Control<Tr> {
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

pub fn chainid<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Tr> {
	push_u256!(machine, handler.chain_id());

	Control::Continue
}

pub fn address<S: AsRef<RuntimeState>, Tr>(machine: &mut Machine<S>) -> Control<Tr> {
	let ret = H256::from(machine.state.as_ref().context.address);
	push!(machine, ret);

	Control::Continue
}

pub fn balance<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Tr> {
	pop!(machine, address);
	handler.mark_hot(address.into(), None);
	push_u256!(machine, handler.balance(address.into()));

	Control::Continue
}

pub fn selfbalance<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Tr> {
	push_u256!(
		machine,
		handler.balance(machine.state.as_ref().context.address)
	);

	Control::Continue
}

pub fn origin<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handler: &H,
) -> Control<Tr> {
	let ret = H256::from(machine.state.as_ref().transaction_context.origin);
	push!(machine, ret);

	Control::Continue
}

pub fn caller<S: AsRef<RuntimeState>, Tr>(machine: &mut Machine<S>) -> Control<Tr> {
	let ret = H256::from(machine.state.as_ref().context.caller);
	push!(machine, ret);

	Control::Continue
}

pub fn callvalue<S: AsRef<RuntimeState>, Tr>(machine: &mut Machine<S>) -> Control<Tr> {
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

pub fn gasprice<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handler: &H,
) -> Control<Tr> {
	let mut ret = H256::default();
	machine
		.state
		.as_ref()
		.transaction_context
		.gas_price
		.to_big_endian(&mut ret[..]);
	push!(machine, ret);

	Control::Continue
}

pub fn basefee<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Tr> {
	let mut ret = H256::default();
	handler.block_base_fee_per_gas().to_big_endian(&mut ret[..]);
	push!(machine, ret);

	Control::Continue
}

pub fn extcodesize<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Tr> {
	pop!(machine, address);
	handler.mark_hot(address.into(), None);
	let code_size = handler.code_size(address.into());
	push_u256!(machine, code_size);

	Control::Continue
}

pub fn extcodehash<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Tr> {
	pop!(machine, address);
	handler.mark_hot(address.into(), None);
	let code_hash = handler.code_hash(address.into());
	push!(machine, code_hash);

	Control::Continue
}

pub fn extcodecopy<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Tr> {
	pop!(machine, address);
	pop_u256!(machine, memory_offset, code_offset, len);

	handler.mark_hot(address.into(), None);
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

pub fn returndatasize<S: AsRef<RuntimeState>, Tr>(machine: &mut Machine<S>) -> Control<Tr> {
	let size = U256::from(machine.state.as_ref().retbuf.len());
	push_u256!(machine, size);

	Control::Continue
}

pub fn returndatacopy<S: AsRef<RuntimeState>, Tr>(machine: &mut Machine<S>) -> Control<Tr> {
	pop_u256!(machine, memory_offset, data_offset, len);

	try_or_fail!(machine.memory.resize_offset(memory_offset, len));
	if data_offset.checked_add(len).map_or(true, |l| {
		l > U256::from(machine.state.as_ref().retbuf.len())
	}) {
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

pub fn blockhash<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Tr> {
	pop_u256!(machine, number);
	push!(machine, handler.block_hash(number));

	Control::Continue
}

pub fn coinbase<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Tr> {
	push!(machine, handler.block_coinbase().into());
	Control::Continue
}

pub fn timestamp<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Tr> {
	push_u256!(machine, handler.block_timestamp());
	Control::Continue
}

pub fn number<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Tr> {
	push_u256!(machine, handler.block_number());
	Control::Continue
}

pub fn difficulty<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Tr> {
	push_u256!(machine, handler.block_difficulty());
	Control::Continue
}

pub fn prevrandao<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Tr> {
	if let Some(rand) = handler.block_randomness() {
		push!(machine, rand);
		Control::Continue
	} else {
		difficulty(machine, handler)
	}
}

pub fn gaslimit<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Tr> {
	push_u256!(machine, handler.block_gas_limit());
	Control::Continue
}

pub fn sload<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Tr> {
	pop!(machine, index);
	handler.mark_hot(machine.state.as_ref().context.address, Some(index));
	let value = handler.storage(machine.state.as_ref().context.address, index);
	push!(machine, value);

	Control::Continue
}

pub fn sstore<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Tr> {
	pop!(machine, index, value);
	handler.mark_hot(machine.state.as_ref().context.address, Some(index));

	match handler.set_storage(machine.state.as_ref().context.address, index, value) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn gas<S: GasState, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	_handler: &H,
) -> Control<Tr> {
	push_u256!(machine, machine.state.gas());

	Control::Continue
}

pub fn log<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	n: u8,
	handler: &mut H,
) -> Control<Tr> {
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

	match handler.log(Log {
		address: machine.state.as_ref().context.address,
		topics,
		data,
	}) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn suicide<S: AsRef<RuntimeState>, H: RuntimeEnvironment + RuntimeBackend, Tr>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Tr> {
	let address = machine.state.as_ref().context.address;

	match machine.stack.perform_pop1_push0(|target| {
		let balance = handler.balance(address);

		handler.transfer(Transfer {
			source: address,
			target: (*target).into(),
			value: balance,
		})?;

		handler.mark_delete(address);
		handler.reset_balance(address);

		Ok(((), ()))
	}) {
		Ok(()) => Control::Exit(ExitSucceed::Suicided.into()),
		Err(e) => Control::Exit(Err(e)),
	}
}
