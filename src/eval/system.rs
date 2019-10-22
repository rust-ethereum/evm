use primitive_types::{H256, U256};
use sha3::{Keccak256, Digest};
use crate::{Runtime, ExitError, Handler};
use super::Control;

pub fn sha3<H: Handler>(runtime: &mut Runtime) -> Control<H> {
	pop_u256!(runtime, from, len);
	let from = as_usize_or_fail!(from);
	let len = as_usize_or_fail!(len);
	let data = runtime.machine.memory().get(from, len);
	let ret = Keccak256::digest(data.as_slice());
	push!(runtime, H256::from_slice(ret.as_slice()));

	Control::Continue
}

pub fn address<H: Handler>(runtime: &mut Runtime) -> Control<H> {
	let ret = H256::from(runtime.context.address);
	push!(runtime, ret);

	Control::Continue
}

pub fn balance<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	pop!(runtime, address);
	push_u256!(runtime, handler.ext_balance(address.into()));

	Control::Continue
}

pub fn origin<H: Handler>(runtime: &mut Runtime) -> Control<H> {
	let ret = H256::from(runtime.context.origin);
	push!(runtime, ret);

	Control::Continue
}

pub fn caller<H: Handler>(runtime: &mut Runtime) -> Control<H> {
	let ret = H256::from(runtime.context.caller);
	push!(runtime, ret);

	Control::Continue
}

pub fn callvalue<H: Handler>(runtime: &mut Runtime) -> Control<H> {
	let mut ret = H256::default();
	runtime.context.value.value().to_big_endian(&mut ret[..]);
	push!(runtime, ret);

	Control::Continue
}

pub fn gasprice<H: Handler>(runtime: &mut Runtime) -> Control<H> {
	let mut ret = H256::default();
	runtime.context.gas_price.to_big_endian(&mut ret[..]);
	push!(runtime, ret);

	Control::Continue
}

pub fn extcodesize<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	pop!(runtime, address);
	push_u256!(runtime, handler.ext_code_size(address.into()));

	Control::Continue
}

pub fn extcodehash<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	pop!(runtime, address);
	push!(runtime, handler.ext_code_hash(address.into()));

	Control::Continue
}

pub fn extcodecopy<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	pop!(runtime, address);
	pop_u256!(runtime, memory_offset, code_offset, len);

	match runtime.machine.memory_mut().copy_large(
		memory_offset,
		code_offset,
		len,
		&handler.ext_code(address.into())
	) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	};

	Control::Continue
}

pub fn returndatasize<H: Handler>(runtime: &mut Runtime) -> Control<H> {
	let size = U256::from(runtime.return_data_buffer.len());
	push_u256!(runtime, size);

	Control::Continue
}

pub fn returndatacopy<H: Handler>(runtime: &mut Runtime) -> Control<H> {
	pop_u256!(runtime, memory_offset, data_offset, len);

	match runtime.machine.memory_mut().copy_large(memory_offset, data_offset, len, &runtime.return_data_buffer) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn blockhash<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	pop_u256!(runtime, number);
	push!(runtime, handler.block_hash(number));

	Control::Continue
}

pub fn coinbase<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	push!(runtime, handler.block_coinbase().into());
	Control::Continue
}

pub fn timestamp<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	push_u256!(runtime, handler.block_timestamp());
	Control::Continue
}

pub fn number<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	push_u256!(runtime, handler.block_number());
	Control::Continue
}

pub fn difficulty<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	push_u256!(runtime, handler.block_difficulty());
	Control::Continue
}

pub fn gaslimit<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	push_u256!(runtime, handler.block_gas_limit());
	Control::Continue
}

pub fn sload<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	pop!(runtime, index);
	push!(runtime, handler.storage(index));

	Control::Continue
}

pub fn sstore<H: Handler>(runtime: &mut Runtime, handler: &mut H) -> Control<H> {
	pop!(runtime, index, value);
	match handler.set_storage(index, value) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn gas<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	push_u256!(runtime, handler.gas_left());

	Control::Continue
}

pub fn log<H: Handler>(runtime: &mut Runtime, n: u8, handler: &mut H) -> Control<H> {
	pop_u256!(runtime, offset, len);
	let offset = as_usize_or_fail!(offset);
	let len = as_usize_or_fail!(len);
	let data = runtime.machine.memory().get(offset, len);

	let mut topics = Vec::new();
	for _ in 0..(n as usize) {
		match runtime.machine.stack_mut().pop() {
			Ok(value) => { topics.push(value); }
			Err(e) => return Control::Exit(e.into()),
		}
	}

	match handler.log(topics, data) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn suicide<H: Handler>(runtime: &mut Runtime, handler: &mut H) -> Control<H> {
	pop!(runtime, target);

	match handler.transfer(runtime.context.address, target.into(), None) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	}

	match handler.mark_delete(runtime.context.address) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	}

	Control::Continue
}
