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

	Control::Interrupt(vec![Interrupt::ExtBalance { address: address.into() }])
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

	Control::Interrupt(vec![Interrupt::ExtCodeSize { address: address.into() }])
}

pub fn extcodehash(runtime: &mut Runtime) -> Control {
	pop!(runtime, address);
	push!(runtime, H256::default());

	Control::Interrupt(vec![Interrupt::ExtCodeHash { address: address.into() }])
}

pub fn extcodecopy(runtime: &mut Runtime) -> Control {
	pop!(runtime, address);
	pop_u256!(runtime, memory_offset, code_offset, len);

	match runtime.machine.memory_mut().copy_large(memory_offset, code_offset, len, &[]) {
		Ok(()) => (),
		Err(e) => return Control::Exit(e.into()),
	};

	Control::Interrupt(vec![Interrupt::ExtCodeCopy { address: address.into(), memory_offset, code_offset, len }])
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

pub fn blockhash(runtime: &mut Runtime) -> Control {
	pop_u256!(runtime, number);
	let current_number = runtime.block_context.number;
	push!(runtime,
		  if !(number >= current_number || current_number - number > U256::from(256u64)) {
			  let n = (current_number - number).as_usize();
			  runtime.block_context.past_hashes[n]
		  } else {
			  H256::default()
		  });

	Control::Continue
}

pub fn coinbase(runtime: &mut Runtime) -> Control {
	push!(runtime, H256::from(runtime.block_context.coinbase));
	Control::Continue
}

pub fn timestamp(runtime: &mut Runtime) -> Control {
	push_u256!(runtime, U256::from(runtime.block_context.timestamp));
	Control::Continue
}

pub fn number(runtime: &mut Runtime) -> Control {
	push_u256!(runtime, U256::from(runtime.block_context.number));
	Control::Continue
}

pub fn difficulty(runtime: &mut Runtime) -> Control {
	push_u256!(runtime, U256::from(runtime.block_context.difficulty));
	Control::Continue
}

pub fn gaslimit(runtime: &mut Runtime) -> Control {
	push_u256!(runtime, U256::from(runtime.block_context.gas_limit));
	Control::Continue
}

pub fn sload(runtime: &mut Runtime) -> Control {
	pop!(runtime, index);
	push!(runtime, H256::default());

	Control::Interrupt(vec![Interrupt::SLoad { index }])
}

pub fn sstore(runtime: &mut Runtime) -> Control {
	pop!(runtime, index, value);
	Control::Interrupt(vec![Interrupt::SStore { index, value }])
}

pub fn log(runtime: &mut Runtime, n: u8) -> Control {
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

	Control::Interrupt(vec![Interrupt::Log { topics, data }])
}

pub fn suicide(runtime: &mut Runtime) -> Control {
	pop!(runtime, target);

	Control::Interrupt(vec![
		Interrupt::MarkDelete { address: runtime.action_context.address },
		Interrupt::Transfer {
			source: runtime.action_context.address, target: target.into(), value: None,
		}
	])
}
