use super::Control;
use crate::{
	CallScheme, Capture, Context, CreateScheme, ExitError, ExitSucceed, Handler, Runtime, Transfer,
};
use alloc::vec::Vec;
use primitive_types::{H256, U256};
use sha3::{Digest, Keccak256};

pub fn sha3<H: Handler>(runtime: &mut Runtime) -> Control<H> {
	pop_u256!(runtime, from);
	pop_usize!(runtime, len);

	let from = if len == 0 {
		usize::MAX
	} else {
		from.as_usize()
	};

	try_or_fail!(runtime.machine.memory_mut().resize_offset(from, len));
	let data = if len == 0 {
		Vec::new()
	} else {
		runtime.machine.memory_mut().get(from, len)
	};

	let ret = Keccak256::digest(data.as_slice());
	push_h256!(runtime, H256::from_slice(ret.as_slice()));

	Control::Continue
}

pub fn chainid<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	push_u256!(runtime, handler.chain_id());

	Control::Continue
}

pub fn address<H: Handler>(runtime: &mut Runtime) -> Control<H> {
	let ret = H256::from(runtime.context.address);
	push_h256!(runtime, ret);

	Control::Continue
}

pub fn balance<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	pop_h256!(runtime, address);
	push_u256!(runtime, handler.balance(address.into()));

	Control::Continue
}

pub fn selfbalance<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	push_u256!(runtime, handler.balance(runtime.context.address));

	Control::Continue
}

pub fn origin<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	let ret = H256::from(handler.origin());
	push_h256!(runtime, ret);

	Control::Continue
}

pub fn caller<H: Handler>(runtime: &mut Runtime) -> Control<H> {
	let ret = H256::from(runtime.context.caller);
	push_h256!(runtime, ret);

	Control::Continue
}

pub fn callvalue<H: Handler>(runtime: &mut Runtime) -> Control<H> {
	let mut ret = H256::default();
	runtime.context.apparent_value.to_big_endian(&mut ret[..]);
	push_h256!(runtime, ret);

	Control::Continue
}

pub fn gasprice<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	let mut ret = H256::default();
	handler.gas_price().to_big_endian(&mut ret[..]);
	push_h256!(runtime, ret);

	Control::Continue
}

pub fn base_fee<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	let mut ret = H256::default();
	handler.block_base_fee_per_gas().to_big_endian(&mut ret[..]);
	push_h256!(runtime, ret);

	Control::Continue
}

pub fn extcodesize<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	pop_h256!(runtime, address);
	push_u256!(runtime, handler.code_size(address.into()));

	Control::Continue
}

pub fn extcodehash<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	pop_h256!(runtime, address);
	push_h256!(runtime, handler.code_hash(address.into()));

	Control::Continue
}

pub fn extcodecopy<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	pop_h256!(runtime, address);
	pop_u256!(runtime, memory_offset);
	pop_u256!(runtime, code_offset);
	pop_usize!(runtime, len);

	if len == 0 {
		return Control::Continue;
	}

	let memory_offset = memory_offset.as_usize();

	try_or_fail!(runtime
		.machine
		.memory_mut()
		.resize_offset(memory_offset, len));
	match runtime.machine.memory_mut().copy_large(
		memory_offset,
		code_offset,
		len,
		&handler.code(address.into()),
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
	pop_u256!(runtime, memory_offset);
	pop_u256!(runtime, data_offset);
	pop_usize!(runtime, len);

	// If `len` is zero then nothing happens, regardless of the
	// value of the other parameters. In particular, `memory_offset`
	// might be larger than `usize::MAX`, hence why we check this first.
	if len == 0 {
		return Control::Continue;
	}

	// SAFETY: this cast is safe because if `len > 0` then gas cost of memory
	// would have already been taken into account at this point. It is impossible
	// to have a memory offset greater than `usize::MAX` for any gas limit less
	// than `u64::MAX` (and gas limits higher than this are disallowed in general).
	let memory_offset = memory_offset.as_usize();

	try_or_fail!(runtime
		.machine
		.memory_mut()
		.resize_offset(memory_offset, len));
	if data_offset
		.checked_add(len.into())
		.map(|l| l > U256::from(runtime.return_data_buffer.len()))
		.unwrap_or(true)
	{
		return Control::Exit(ExitError::OutOfOffset.into());
	}

	match runtime.machine.memory_mut().copy_large(
		memory_offset,
		data_offset,
		len,
		&runtime.return_data_buffer,
	) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn blockhash<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	pop_u256!(runtime, number);
	push_h256!(runtime, handler.block_hash(number));

	Control::Continue
}

pub fn coinbase<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	push_h256!(runtime, handler.block_coinbase());
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

pub fn prevrandao<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	if let Some(rand) = handler.block_randomness() {
		push_h256!(runtime, rand);
		Control::Continue
	} else {
		difficulty(runtime, handler)
	}
}

pub fn gaslimit<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	push_u256!(runtime, handler.block_gas_limit());
	Control::Continue
}

pub fn sload<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	pop_h256!(runtime, index);
	let value = handler.storage(runtime.context.address, index);
	push_h256!(runtime, value);

	event!(SLoad {
		address: runtime.context.address,
		index,
		value
	});

	Control::Continue
}

pub fn sstore<H: Handler>(runtime: &mut Runtime, handler: &mut H) -> Control<H> {
	pop_h256!(runtime, index, value);

	event!(SStore {
		address: runtime.context.address,
		index,
		value
	});

	match handler.set_storage(runtime.context.address, index, value) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn gas<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	push_u256!(runtime, handler.gas_left());

	Control::Continue
}

pub fn log<H: Handler>(runtime: &mut Runtime, n: u8, handler: &mut H) -> Control<H> {
	pop_u256!(runtime, offset);
	pop_usize!(runtime, len);

	let offset = if len == 0 {
		usize::MAX
	} else {
		offset.as_usize()
	};

	try_or_fail!(runtime.machine.memory_mut().resize_offset(offset, len));
	let data = if len == 0 {
		Vec::new()
	} else {
		runtime.machine.memory().get(offset, len)
	};

	let mut topics = Vec::new();
	for _ in 0..(n as usize) {
		match runtime.machine.stack_mut().pop_h256() {
			Ok(value) => {
				topics.push(value);
			}
			Err(e) => return Control::Exit(e.into()),
		}
	}

	match handler.log(runtime.context.address, topics, data) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn suicide<H: Handler>(runtime: &mut Runtime, handler: &mut H) -> Control<H> {
	pop_h256!(runtime, target);

	match handler.mark_delete(runtime.context.address, target.into()) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	}

	Control::Exit(ExitSucceed::Suicided.into())
}

pub fn create<H: Handler>(runtime: &mut Runtime, is_create2: bool, handler: &mut H) -> Control<H> {
	runtime.return_data_buffer = Vec::new();

	pop_u256!(runtime, value);
	pop_u256!(runtime, code_offset);
	pop_usize!(runtime, len);

	let code_offset = if len == 0 {
		usize::MAX
	} else {
		code_offset.as_usize()
	};

	try_or_fail!(runtime.machine.memory_mut().resize_offset(code_offset, len));
	let code = if len == 0 {
		Vec::new()
	} else {
		runtime.machine.memory().get(code_offset, len)
	};

	let scheme = if is_create2 {
		pop_h256!(runtime, salt);
		let code_hash = H256::from_slice(Keccak256::digest(&code).as_slice());
		CreateScheme::Create2 {
			caller: runtime.context.address,
			salt,
			code_hash,
		}
	} else {
		CreateScheme::Legacy {
			caller: runtime.context.address,
		}
	};

	match handler.create(runtime.context.address, scheme, value, code, None) {
		Capture::Exit((reason, address, return_data)) => {
			match super::finish_create(runtime, reason, address, return_data) {
				Ok(()) => Control::Continue,
				Err(e) => Control::Exit(e),
			}
		}
		Capture::Trap(interrupt) => Control::CreateInterrupt(interrupt),
	}
}

pub fn call<H: Handler>(runtime: &mut Runtime, scheme: CallScheme, handler: &mut H) -> Control<H> {
	runtime.return_data_buffer = Vec::new();

	pop_u256!(runtime, gas);
	pop_h256!(runtime, to);
	let gas = if gas > U256::from(u64::MAX) {
		None
	} else {
		Some(gas.as_u64())
	};

	let value = match scheme {
		CallScheme::Call | CallScheme::CallCode => {
			pop_u256!(runtime, value);
			value
		}
		CallScheme::DelegateCall | CallScheme::StaticCall => U256::zero(),
	};

	pop_u256!(runtime, in_offset);
	pop_usize!(runtime, in_len);
	pop_u256!(runtime, out_offset);
	pop_usize!(runtime, out_len);

	let in_offset = if in_len == 0 {
		usize::MAX
	} else {
		in_offset.as_usize()
	};
	let out_offset = if out_len == 0 {
		usize::MAX
	} else {
		out_offset.as_usize()
	};

	try_or_fail!(runtime
		.machine
		.memory_mut()
		.resize_offset(in_offset, in_len));
	try_or_fail!(runtime
		.machine
		.memory_mut()
		.resize_offset(out_offset, out_len));

	let input = if in_len == 0 {
		Vec::new()
	} else {
		runtime.machine.memory().get(in_offset, in_len)
	};

	let context = match scheme {
		CallScheme::Call | CallScheme::StaticCall => Context {
			address: to.into(),
			caller: runtime.context.address,
			apparent_value: value,
		},
		CallScheme::CallCode => Context {
			address: runtime.context.address,
			caller: runtime.context.address,
			apparent_value: value,
		},
		CallScheme::DelegateCall => Context {
			address: runtime.context.address,
			caller: runtime.context.caller,
			apparent_value: runtime.context.apparent_value,
		},
	};

	let transfer = if scheme == CallScheme::Call {
		Some(Transfer {
			source: runtime.context.address,
			target: to.into(),
			value,
		})
	} else if scheme == CallScheme::CallCode {
		Some(Transfer {
			source: runtime.context.address,
			target: runtime.context.address,
			value,
		})
	} else {
		None
	};

	match handler.call(
		to.into(),
		transfer,
		input,
		gas,
		scheme == CallScheme::StaticCall,
		context,
	) {
		Capture::Exit((reason, return_data)) => {
			match super::finish_call(runtime, out_len, out_offset, reason, return_data) {
				Ok(()) => Control::Continue,
				Err(e) => Control::Exit(e),
			}
		}
		Capture::Trap(interrupt) => {
			runtime.return_data_len = out_len;
			runtime.return_data_offset = out_offset;
			Control::CallInterrupt(interrupt)
		}
	}
}
