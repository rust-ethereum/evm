use super::Control;
use crate::{
	CallScheme, Context, CreateScheme, ExitException, ExitFatal, ExitSucceed, Handler, Machine,
	RuntimeCallTrapData, RuntimeCreateTrapData, RuntimeState, RuntimeTrapData, Transfer,
};
use alloc::vec::Vec;
use primitive_types::{H256, U256};
use sha3::{Digest, Keccak256};

pub fn sha3<S: AsRef<RuntimeState> + AsMut<RuntimeState>, Td: From<RuntimeTrapData>>(
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

pub fn chainid<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push_u256!(machine, handler.chain_id());

	Control::Continue
}

pub fn address<S: AsRef<RuntimeState> + AsMut<RuntimeState>, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
) -> Control<Td> {
	let ret = H256::from(machine.state.as_ref().context.address);
	push!(machine, ret);

	Control::Continue
}

pub fn balance<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Td> {
	pop!(machine, address);
	try_or_fail!(handler.mark_hot(address.into(), None));
	push_u256!(machine, handler.balance(address.into()));

	Control::Continue
}

pub fn selfbalance<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push_u256!(
		machine,
		handler.balance(machine.state.as_ref().context.address)
	);

	Control::Continue
}

pub fn origin<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	let ret = H256::from(handler.origin());
	push!(machine, ret);

	Control::Continue
}

pub fn caller<S: AsRef<RuntimeState> + AsMut<RuntimeState>, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
) -> Control<Td> {
	let ret = H256::from(machine.state.as_ref().context.caller);
	push!(machine, ret);

	Control::Continue
}

pub fn callvalue<S: AsRef<RuntimeState> + AsMut<RuntimeState>, Td: From<RuntimeTrapData>>(
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

pub fn gasprice<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	let mut ret = H256::default();
	handler.gas_price().to_big_endian(&mut ret[..]);
	push!(machine, ret);

	Control::Continue
}

pub fn basefee<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	let mut ret = H256::default();
	handler.block_base_fee_per_gas().to_big_endian(&mut ret[..]);
	push!(machine, ret);

	Control::Continue
}

pub fn extcodesize<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Td> {
	pop!(machine, address);
	try_or_fail!(handler.mark_hot(address.into(), None));
	let code_size = handler.code_size(address.into());
	push_u256!(machine, code_size);

	Control::Continue
}

pub fn extcodehash<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Td> {
	pop!(machine, address);
	try_or_fail!(handler.mark_hot(address.into(), None));
	let code_hash = handler.code_hash(address.into());
	push!(machine, code_hash);

	Control::Continue
}

pub fn extcodecopy<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
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

pub fn returndatasize<S: AsRef<RuntimeState> + AsMut<RuntimeState>, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
) -> Control<Td> {
	let size = U256::from(machine.state.as_ref().retbuf.len());
	push_u256!(machine, size);

	Control::Continue
}

pub fn returndatacopy<S: AsRef<RuntimeState> + AsMut<RuntimeState>, Td: From<RuntimeTrapData>>(
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

pub fn blockhash<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	pop_u256!(machine, number);
	push!(machine, handler.block_hash(number));

	Control::Continue
}

pub fn coinbase<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push!(machine, handler.block_coinbase().into());
	Control::Continue
}

pub fn timestamp<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push_u256!(machine, handler.block_timestamp());
	Control::Continue
}

pub fn number<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push_u256!(machine, handler.block_number());
	Control::Continue
}

pub fn difficulty<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push_u256!(machine, handler.block_difficulty());
	Control::Continue
}

pub fn prevrandao<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
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

pub fn gaslimit<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push_u256!(machine, handler.block_gas_limit());
	Control::Continue
}

pub fn sload<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
	machine: &mut Machine<S>,
	handler: &mut H,
) -> Control<Td> {
	pop!(machine, index);
	try_or_fail!(handler.mark_hot(machine.state.as_ref().context.address, Some(index)));
	let value = handler.storage(machine.state.as_ref().context.address, index);
	push!(machine, value);

	Control::Continue
}

pub fn sstore<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
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

pub fn gas<S: AsRef<RuntimeState> + AsMut<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
	machine: &mut Machine<S>,
	handler: &H,
) -> Control<Td> {
	push_u256!(machine, handler.gas_left());

	Control::Continue
}

pub fn log<S: AsRef<RuntimeState> + AsMut<RuntimeState>, H: Handler, Td: From<RuntimeTrapData>>(
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

pub fn suicide<
	S: AsRef<RuntimeState> + AsMut<RuntimeState>,
	H: Handler,
	Td: From<RuntimeTrapData>,
>(
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

pub fn create<S: AsRef<RuntimeState> + AsMut<RuntimeState>, Td: From<RuntimeTrapData>>(
	is_create2: bool,
	machine: &mut Machine<S>,
) -> Control<Td> {
	machine.state.as_mut().retbuf = Vec::new();

	pop_u256!(machine, value, code_offset, len);

	try_or_fail!(machine.memory.resize_offset(code_offset, len));
	let code = if len == U256::zero() {
		Vec::new()
	} else {
		let code_offset = as_usize_or_fail!(code_offset);
		let len = as_usize_or_fail!(len);

		machine.memory.get(code_offset, len)
	};

	let scheme = if is_create2 {
		pop!(machine, salt);
		let code_hash = H256::from_slice(Keccak256::digest(&code).as_slice());
		CreateScheme::Create2 {
			caller: machine.state.as_ref().context.address,
			salt,
			code_hash,
		}
	} else {
		CreateScheme::Legacy {
			caller: machine.state.as_ref().context.address,
		}
	};

	Control::Trap(
		RuntimeTrapData::Create(Box::new(RuntimeCreateTrapData {
			scheme,
			value,
			code,
		}))
		.into(),
	)
}

pub fn call<S: AsRef<RuntimeState> + AsMut<RuntimeState>, Td: From<RuntimeTrapData>>(
	scheme: CallScheme,
	machine: &mut Machine<S>,
) -> Control<Td> {
	machine.state.as_mut().retbuf = Vec::new();

	pop_u256!(machine, gas);
	pop!(machine, to);

	let value = match scheme {
		CallScheme::Call | CallScheme::CallCode => {
			pop_u256!(machine, value);
			value
		}
		CallScheme::DelegateCall | CallScheme::StaticCall => U256::zero(),
	};

	pop_u256!(machine, in_offset, in_len, out_offset, out_len);

	try_or_fail!(machine.memory.resize_offset(in_offset, in_len));
	try_or_fail!(machine.memory.resize_offset(out_offset, out_len));

	let input = if in_len == U256::zero() {
		Vec::new()
	} else {
		let in_offset = as_usize_or_fail!(in_offset);
		let in_len = as_usize_or_fail!(in_len);

		machine.memory.get(in_offset, in_len)
	};

	let context = match scheme {
		CallScheme::Call | CallScheme::StaticCall => Context {
			address: to.into(),
			caller: machine.state.as_ref().context.address,
			apparent_value: value,
		},
		CallScheme::CallCode => Context {
			address: machine.state.as_ref().context.address,
			caller: machine.state.as_ref().context.address,
			apparent_value: value,
		},
		CallScheme::DelegateCall => Context {
			address: machine.state.as_ref().context.address,
			caller: machine.state.as_ref().context.caller,
			apparent_value: machine.state.as_ref().context.apparent_value,
		},
	};

	let transfer = if scheme == CallScheme::Call {
		Some(Transfer {
			source: machine.state.as_ref().context.address,
			target: to.into(),
			value,
		})
	} else if scheme == CallScheme::CallCode {
		Some(Transfer {
			source: machine.state.as_ref().context.address,
			target: machine.state.as_ref().context.address,
			value,
		})
	} else {
		None
	};

	Control::Trap(
		RuntimeTrapData::Call(Box::new(RuntimeCallTrapData {
			target: to.into(),
			transfer,
			input,
			gas,
			is_static: scheme == CallScheme::StaticCall,
			context,
			out_offset,
			out_len,
		}))
		.into(),
	)
}
