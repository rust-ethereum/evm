use super::Control;
use crate::prelude::*;
use crate::{
	CallScheme, Capture, Context, CreateScheme, ExitError, ExitSucceed, Handler, Runtime, Transfer,
};
use core::cmp::max;
use evm_core::utils::{U64_MAX, USIZE_MAX};
use primitive_types::{H256, U256};
use sha3::{Digest, Keccak256};

pub fn sha3<H: Handler>(runtime: &mut Runtime) -> Control<H> {
	pop_u256!(runtime, from, len);

	// Cast to `usize` after length checking to avoid overflow
	let from = if len == U256::zero() {
		usize::MAX
	} else {
		as_usize_or_fail!(from)
	};
	let len = as_usize_or_fail!(len);

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
	push_u256!(runtime, handler.block_base_fee_per_gas());
	Control::Continue
}

/// CANCUN hard fork
/// EIP-7516: BLOBBASEFEE opcode
pub fn blob_base_fee<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	let blob_base_fee = U256::from(handler.blob_base_fee().unwrap_or_default());
	push_u256!(runtime, blob_base_fee);
	Control::Continue
}

/// CANCUN hard fork
/// EIP-4844: Shard Blob Transactions
/// Logic related to operating with BLOBHASH opcode described:
/// - https://eips.ethereum.org/EIPS/eip-4844#opcode-to-get-versioned-hashes
pub fn blob_hash<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	// Peek index from the top of the stack
	let raw_index = match runtime.machine.stack().peek(0) {
		Ok(value) => value,
		Err(e) => return Control::Exit(e.into()),
	};
	// Safely cast to usize
	let index = if raw_index > USIZE_MAX {
		usize::MAX
	} else {
		raw_index.as_usize()
	};
	// Get blob_hash from `tx.blob_versioned_hashes[index]`
	// as described:
	// - https://eips.ethereum.org/EIPS/eip-4844#opcode-to-get-versioned-hashes
	let blob_hash = handler.get_blob_hash(index).unwrap_or(U256::zero());
	// Set top stack index with `blob_hash` value
	if let Err(e) = runtime.machine.stack_mut().set(0, blob_hash) {
		return Control::Exit(e.into());
	}
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
	pop_u256!(runtime, memory_offset, code_offset, len);

	if len == U256::zero() {
		return Control::Continue;
	}
	let len = as_usize_or_fail!(len);

	// Cast to `usize` after length checking to avoid overflow
	let memory_offset = as_usize_or_fail!(memory_offset);

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
	pop_u256!(runtime, memory_offset, data_offset, len);

	// If `len` is zero then nothing happens to the memory, regardless
	// of the value of `memory_offset`. In particular, the value taken
	// from the stack might be larger than `usize::MAX`, hence why the
	// `as_usize` cast is not always safe. But because the value does
	// not matter when `len == 0` we can safely set it equal to zero instead.
	let memory_offset = if len == U256::zero() {
		0
	} else {
		// SAFETY: this cast is safe because if `len > 0` then gas cost of memory
		// would have already been taken into account at this point. It is impossible
		// to have a memory offset greater than `usize::MAX` for any gas limit less
		// than `u64::MAX` (and gas limits higher than this are disallowed in general).
		as_usize_or_fail!(memory_offset)
	};
	let len = as_usize_or_fail!(len);

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

/// EIP-1153: Transient storage opcodes
/// Load value from transient storage
pub fn tload<H: Handler>(runtime: &mut Runtime, handler: &mut H) -> Control<H> {
	// Peek index from the top of the stack
	let index = match runtime.machine.stack().peek(0) {
		Ok(value) => {
			let mut h = H256::default();
			value.to_big_endian(&mut h[..]);
			h
		}
		Err(e) => return Control::Exit(e.into()),
	};
	// Load value from transient storage
	let value = match handler.tload(runtime.context.address, index) {
		Ok(value) => value,
		Err(e) => return Control::Exit(e.into()),
	};
	// Set top stack index with `transient` value result
	match runtime.machine.stack_mut().set(0, value) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	}

	Control::Continue
}

/// EIP-1153: Transient storage
/// Store value to transient storage
pub fn tstore<H: Handler>(runtime: &mut Runtime, handler: &mut H) -> Control<H> {
	pop_h256!(runtime, index);
	pop_u256!(runtime, value);
	match handler.tstore(runtime.context.address, index, value) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

/// CANCUN hard fork
/// EIP-5656: MCOPY - Memory copying instruction
pub fn mcopy<H: Handler>(runtime: &mut Runtime, _handler: &mut H) -> Control<H> {
	pop_u256!(runtime, dst, src, len);
	if len == U256::zero() {
		return Control::Continue;
	}
	let len = as_usize_or_fail!(len, ExitError::OutOfGas);
	let dst = as_usize_or_fail!(dst, ExitError::OutOfGas);
	let src = as_usize_or_fail!(src, ExitError::OutOfGas);

	try_or_fail!(runtime
		.machine
		.memory_mut()
		.resize_offset(max(src, dst), len));

	// copy memory
	match runtime.machine.memory_mut().copy(src, dst, len) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	};

	Control::Continue
}

pub fn gas<H: Handler>(runtime: &mut Runtime, handler: &H) -> Control<H> {
	push_u256!(runtime, handler.gas_left());

	Control::Continue
}

pub fn log<H: Handler>(runtime: &mut Runtime, n: u8, handler: &mut H) -> Control<H> {
	pop_u256!(runtime, offset, len);

	// Cast to `usize` after length checking to avoid overflow
	let offset = if len == U256::zero() {
		usize::MAX
	} else {
		as_usize_or_fail!(offset)
	};
	let len = as_usize_or_fail!(len);

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

/// Performances SELFDESTRUCT action.
/// Transfers balance from address to target. Check if target exist/is_cold
///
/// Note: balance will be lost if address and target are the same BUT when
/// current spec enables Cancun, this happens only when the account associated to address
/// is created in the same tx
///
/// references:
///  * <https://github.com/ethereum/go-ethereum/blob/141cd425310b503c5678e674a8c3872cf46b7086/core/vm/instructions.go#L832-L833>
///  * <https://github.com/ethereum/go-ethereum/blob/141cd425310b503c5678e674a8c3872cf46b7086/core/state/statedb.go#L449>
///  * <https://eips.ethereum.org/EIPS/eip-6780>
pub fn selfdestruct<H: Handler>(runtime: &mut Runtime, handler: &mut H) -> Control<H> {
	pop_h256!(runtime, target);

	match handler.mark_delete(runtime.context.address, target.into()) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	}

	Control::Exit(ExitSucceed::Suicided.into())
}

pub fn create<H: Handler>(runtime: &mut Runtime, is_create2: bool, handler: &mut H) -> Control<H> {
	runtime.return_data_buffer = Vec::new();

	pop_u256!(runtime, value, code_offset, len);

	// Cast to `usize` after length checking to avoid overflow
	let code_offset = if len == U256::zero() {
		usize::MAX
	} else {
		as_usize_or_fail!(code_offset)
	};
	let len = as_usize_or_fail!(len);

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
	let gas = if gas > U64_MAX {
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

	pop_u256!(runtime, in_offset, in_len);
	pop_u256!(runtime, out_offset, out_len);

	// Cast to `usize` after length checking to avoid overflow
	let in_offset = if in_len == U256::zero() {
		usize::MAX
	} else {
		as_usize_or_fail!(in_offset)
	};
	let in_len = as_usize_or_fail!(in_len);
	// Cast to `usize` after length checking to avoid overflow
	let out_offset = if out_len == U256::zero() {
		usize::MAX
	} else {
		as_usize_or_fail!(out_offset)
	};
	let out_len = as_usize_or_fail!(out_len);

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
