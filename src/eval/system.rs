use primitive_types::{H256, U256};
use sha3::{Keccak256, Digest};
use crate::{Runtime, ExitError, Interrupt};
use super::Control;

pub fn sha3(runtime: &mut Runtime) -> Control {
	pop_u256!(runtime, from, len);
	let from = as_usize_or_fail!(from);
	let len = as_usize_or_fail!(len);
	let data = runtime.machine.memory().get(from, len);
	let ret = Keccak256::digest(data.as_slice());
	push!(runtime, H256::from_slice(ret.as_slice()));

	Control::Continue
}

pub fn address(runtime: &mut Runtime) -> Control {
	let ret = H256::from(runtime.action_context.address);
	push!(runtime, ret);

	Control::Continue
}

pub fn balance(runtime: &mut Runtime) -> Control {
	pop!(runtime, address);
	push!(runtime, H256::default());

	Control::Interrupt(Interrupt::ExtBalance(address.into()))
}

pub fn origin(runtime: &mut Runtime) -> Control {
	let ret = H256::from(runtime.action_context.origin);
	push!(runtime, ret);

	Control::Continue
}

pub fn caller(runtime: &mut Runtime) -> Control {
	let ret = H256::from(runtime.action_context.caller);
	push!(runtime, ret);

	Control::Continue
}

pub fn callvalue(runtime: &mut Runtime) -> Control {
	let mut ret = H256::default();
	runtime.action_context.value.value().to_big_endian(&mut ret[..]);
	push!(runtime, ret);

	Control::Continue
}

pub fn gasprice(runtime: &mut Runtime) -> Control {
	let mut ret = H256::default();
	runtime.action_context.gas_price.to_big_endian(&mut ret[..]);
	push!(runtime, ret);

	Control::Continue
}

pub fn extcodesize(runtime: &mut Runtime) -> Control {
	pop!(runtime, address);
	push!(runtime, H256::default());

	Control::Interrupt(Interrupt::ExtCodeSize(address.into()))
}

pub fn extcodehash(runtime: &mut Runtime) -> Control {
	pop!(runtime, address);
	push!(runtime, H256::default());

	Control::Interrupt(Interrupt::ExtCodeHash(address.into()))
}

pub fn returndatasize(runtime: &mut Runtime) -> Control {
	let size = U256::from(runtime.return_data_buffer.len());
	push_u256!(runtime, size);

	Control::Continue
}

pub fn returndatacopy(runtime: &mut Runtime) -> Control {
	pop_u256!(runtime, memory_offset, data_offset, len);

	match runtime.machine.memory_mut().copy_large(memory_offset, data_offset, len, &runtime.return_data_buffer) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}
