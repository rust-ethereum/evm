use super::Control;
use crate::{
	CallScheme, Capture, Context, CreateScheme, ExitError, ExitFatal, ExitReason, ExitSucceed,
	Handler, Runtime, Transfer,
};

use core::cmp::min;
use elrond_wasm::{api::ManagedTypeApi, types::ManagedVec};
use eltypes::EH256;
use primitive_types::{H256, U256};
use sha3::{Digest, Keccak256};

pub fn sha3<H: Handler<M>, M: ManagedTypeApi>(runtime: &mut Runtime<M>) -> Control<M, H> {
	pop_u256!(runtime, from, len);

	try_or_fail!(runtime.machine.memory_mut().resize_offset(from, len));
	let data = if len == U256::zero() {
		ManagedVec::new()
	} else {
		let from = as_usize_or_fail!(from);
		let len = as_usize_or_fail!(len);

		runtime.machine.memory_mut().get(from, len)
	};

	let ret = Keccak256::digest(data.into_vec());
	push!(runtime, EH256::from(H256::from_slice(ret.as_slice())));

	Control::Continue
}

pub fn chainid<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	push_u256!(runtime, handler.chain_id());

	Control::Continue
}

pub fn address<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
) -> Control<M, H> {
	let ret = H256::from(runtime.context.address);
	push!(runtime, EH256::from(ret));

	Control::Continue
}

pub fn balance<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	pop!(runtime, address);
	push_u256!(runtime, handler.balance(address.to_h256().into()));

	Control::Continue
}

pub fn selfbalance<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	push_u256!(runtime, handler.balance(runtime.context.address));

	Control::Continue
}

pub fn origin<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	let ret = H256::from(handler.origin());
	push!(runtime, EH256::from(ret));

	Control::Continue
}

pub fn caller<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
) -> Control<M, H> {
	let ret = H256::from(runtime.context.caller);
	push!(runtime, EH256::from(ret));

	Control::Continue
}

pub fn callvalue<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
) -> Control<M, H> {
	let mut ret = H256::default();
	runtime.context.apparent_value.to_big_endian(&mut ret[..]);
	push!(runtime, EH256::from(ret));

	Control::Continue
}

pub fn gasprice<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	let mut ret = H256::default();
	handler.gas_price().to_big_endian(&mut ret[..]);
	push!(runtime, EH256::from(ret));

	Control::Continue
}

pub fn base_fee<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	let mut ret = H256::default();
	handler.block_base_fee_per_gas().to_big_endian(&mut ret[..]);
	push!(runtime, EH256::from(ret));

	Control::Continue
}

pub fn extcodesize<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	pop!(runtime, address);
	push_u256!(runtime, handler.code_size(address.to_h256().into()));

	Control::Continue
}

pub fn extcodehash<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	pop!(runtime, address);
	push!(
		runtime,
		EH256::from(handler.code_hash(address.to_h256().into()))
	);

	Control::Continue
}

pub fn extcodecopy<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	pop!(runtime, address);
	pop_u256!(runtime, memory_offset, code_offset, len);

	try_or_fail!(runtime
		.machine
		.memory_mut()
		.resize_offset(memory_offset, len));
	match runtime.machine.memory_mut().copy_large(
		memory_offset,
		code_offset,
		len,
		&handler.code(address.to_h256().into()),
	) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	};

	Control::Continue
}

pub fn returndatasize<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
) -> Control<M, H> {
	let size = U256::from(runtime.return_data_buffer.len());
	push_u256!(runtime, size);

	Control::Continue
}

pub fn returndatacopy<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
) -> Control<M, H> {
	pop_u256!(runtime, memory_offset, data_offset, len);

	try_or_fail!(runtime
		.machine
		.memory_mut()
		.resize_offset(memory_offset, len));
	if data_offset
		.checked_add(len)
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

pub fn blockhash<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	pop_u256!(runtime, number);
	push!(runtime, EH256::from(handler.block_hash(number)));

	Control::Continue
}

pub fn coinbase<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	push!(runtime, EH256::from(handler.block_coinbase().into()));
	Control::Continue
}

pub fn timestamp<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	push_u256!(runtime, handler.block_timestamp());
	Control::Continue
}

pub fn number<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	push_u256!(runtime, handler.block_number());
	Control::Continue
}

pub fn difficulty<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	push_u256!(runtime, handler.block_difficulty());
	Control::Continue
}

pub fn gaslimit<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	push_u256!(runtime, handler.block_gas_limit());
	Control::Continue
}

pub fn sload<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	pop!(runtime, index);
	let value = handler.storage(runtime.context.address, index);
	push!(runtime, EH256::from(value));

	event!(SLoad {
		address: runtime.context.address,
		index,
		value
	});

	Control::Continue
}

pub fn sstore<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &mut H,
) -> Control<M, H> {
	pop!(runtime, index, value);

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

pub fn gas<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &H,
) -> Control<M, H> {
	push_u256!(runtime, handler.gas_left());

	Control::Continue
}

pub fn log<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	n: u8,
	handler: &mut H,
) -> Control<M, H> {
	pop_u256!(runtime, offset, len);

	try_or_fail!(runtime.machine.memory_mut().resize_offset(offset, len));
	let data = if len == U256::zero() {
		ManagedVec::new()
	} else {
		let offset = as_usize_or_fail!(offset);
		let len = as_usize_or_fail!(len);

		runtime.machine.memory().get(offset, len)
	};

	let mut topics = ManagedVec::new();
	for _ in 0..(n as usize) {
		match runtime.machine.stack_mut().pop() {
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

pub fn suicide<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	handler: &mut H,
) -> Control<M, H> {
	pop!(runtime, target);

	match handler.mark_delete(runtime.context.address, target.to_h256().into()) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	}

	Control::Exit(ExitSucceed::Suicided.into())
}

pub fn create<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	is_create2: bool,
	handler: &mut H,
) -> Control<M, H> {
	runtime.return_data_buffer = ManagedVec::new();

	pop_u256!(runtime, value, code_offset, len);

	try_or_fail!(runtime.machine.memory_mut().resize_offset(code_offset, len));
	let code = if len == U256::zero() {
		ManagedVec::new()
	} else {
		let code_offset = as_usize_or_fail!(code_offset);
		let len = as_usize_or_fail!(len);

		runtime.machine.memory().get(code_offset, len)
	};

	let scheme = if is_create2 {
		pop!(runtime, salt);
		let code_hash = H256::from_slice(Keccak256::digest(&code.clone().into_vec()).as_slice());
		CreateScheme::Create2 {
			caller: runtime.context.address,
			salt: salt.to_h256(),
			code_hash,
		}
	} else {
		CreateScheme::Legacy {
			caller: runtime.context.address,
		}
	};

	match handler.create(runtime.context.address, scheme, value, code, None) {
		Capture::Exit((reason, address, return_data)) => {
			runtime.return_data_buffer = return_data;
			let create_address: H256 = address.map(|a| a.into()).unwrap_or_default();

			match reason {
				ExitReason::Succeed(_) => {
					push!(runtime, EH256::from(create_address));
					Control::Continue
				}
				ExitReason::Revert(_) => {
					push!(runtime, EH256::from(H256::default()));
					Control::Continue
				}
				ExitReason::Error(_) => {
					push!(runtime, EH256::from(H256::default()));
					Control::Continue
				}
				ExitReason::Fatal(e) => {
					push!(runtime, EH256::from(H256::default()));
					Control::Exit(e.into())
				}
			}
		}
		Capture::Trap(interrupt) => {
			push!(runtime, EH256::from(H256::default()));
			Control::CreateInterrupt(interrupt)
		}
	}
}

pub fn call<'config, H: Handler<M>, M: ManagedTypeApi>(
	runtime: &mut Runtime<'config, M>,
	scheme: CallScheme,
	handler: &mut H,
) -> Control<M, H> {
	runtime.return_data_buffer = ManagedVec::new();

	pop_u256!(runtime, gas);
	pop!(runtime, to);
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

	pop_u256!(runtime, in_offset, in_len, out_offset, out_len);

	try_or_fail!(runtime
		.machine
		.memory_mut()
		.resize_offset(in_offset, in_len));
	try_or_fail!(runtime
		.machine
		.memory_mut()
		.resize_offset(out_offset, out_len));

	let input = if in_len == U256::zero() {
		ManagedVec::new()
	} else {
		let in_offset = as_usize_or_fail!(in_offset);
		let in_len = as_usize_or_fail!(in_len);

		runtime.machine.memory().get(in_offset, in_len)
	};

	let context = match scheme {
		CallScheme::Call | CallScheme::StaticCall => Context {
			address: to.clone().to_h256().into(),
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
			target: to.clone().to_h256().into(),
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
		to.to_h256().into(),
		transfer,
		input,
		gas,
		scheme == CallScheme::StaticCall,
		context,
	) {
		Capture::Exit((reason, return_data)) => {
			runtime.return_data_buffer = return_data;
			let target_len = min(out_len, U256::from(runtime.return_data_buffer.len()));

			match reason {
				ExitReason::Succeed(_) => {
					match runtime.machine.memory_mut().copy_large(
						out_offset,
						U256::zero(),
						target_len,
						&runtime.return_data_buffer,
					) {
						Ok(()) => {
							push_u256!(runtime, U256::one());
							Control::Continue
						}
						Err(_) => {
							push_u256!(runtime, U256::zero());
							Control::Continue
						}
					}
				}
				ExitReason::Revert(_) => {
					push_u256!(runtime, U256::zero());

					let _ = runtime.machine.memory_mut().copy_large(
						out_offset,
						U256::zero(),
						target_len,
						&runtime.return_data_buffer,
					);

					Control::Continue
				}
				ExitReason::Error(_) => {
					push_u256!(runtime, U256::zero());

					Control::Continue
				}
				ExitReason::Fatal(e) => {
					push_u256!(runtime, U256::zero());

					Control::Exit(e.into())
				}
			}
		}
		Capture::Trap(interrupt) => {
			push!(runtime, EH256::from(H256::default()));
			Control::CallInterrupt(interrupt)
		}
	}
}
