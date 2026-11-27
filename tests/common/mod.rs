//! Helpers shared by the call input tests.
//!
//! Each test builds its bytecode by hand as hex, with a comment for every
//! instruction.

#![allow(dead_code)]

use evm::{
	backend::{MemoryAccount, MemoryBackend, MemoryVicinity},
	executor::stack::{MemoryStackState, PrecompileSet, StackExecutor, StackSubstateMetadata},
	Config, ExitReason,
};
use primitive_types::{H160, U256};
use std::collections::BTreeMap;

/// The transaction sender.
pub const CALLER: H160 = H160([0xc0; 20]);
/// The contract the transaction calls.
pub const CONTRACT: H160 = H160([0x01; 20]);

/// Gas limit of the transaction and of the block.
pub const GAS_LIMIT: u64 = 10_000_000;

/// An address holding the echo callee.
pub const ECHO_ADDR: H160 = address(0x01);
/// An address holding the marker callee.
pub const MARK_ADDR: H160 = address(0x02);
/// An address holding the reverting callee.
pub const REVERT_ADDR: H160 = address(0x03);
/// An address holding the callee that forwards its calldata.
pub const FORWARD_ADDR: H160 = address(0x04);
/// An address of the test precompile.
pub const PRECOMPILE_ADDR: H160 = address(0x05);

/// The test addresses: `0x02` in the nineteenth byte, and `id` in the last.
pub const fn address(id: u8) -> H160 {
	let mut bytes = [0u8; 20];
	bytes[18] = 0x02;
	bytes[19] = id;
	H160(bytes)
}

/// The marker word the marker callee returns.
pub const MARKER: [u8; 32] = [0xa1; 32];
/// The data the reverting callee reverts with.
pub const REVERT_DATA: [u8; 32] = [0xb2; 32];

/// The echo callee returns its whole calldata.
///
/// CALLDATASIZE; PUSH1 0; PUSH1 0; CALLDATACOPY; CALLDATASIZE; PUSH1 0; RETURN
pub const ECHO: &str = "366000600037366000f3";

/// The marker callee ignores its calldata and returns `MARKER`.
///
/// PUSH32 MARKER; PUSH1 0; MSTORE; PUSH1 32; PUSH1 0; RETURN
pub const MARK: &str =
	"7fa1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a160005260206000f3";

/// The reverting callee reverts with `REVERT_DATA`.
///
/// PUSH32 REVERT_DATA; PUSH1 0; MSTORE; PUSH1 32; PUSH1 0; REVERT
pub const REVERT: &str =
	"7fb2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b260005260206000fd";

/// The forwarding callee copies its calldata into memory, calls the echo
/// callee with that range as its input and with an output range of its own,
/// and returns its call's success flag followed by the bytes the echo
/// returned.
///
/// CALLDATASIZE; PUSH1 0; PUSH1 0; CALLDATACOPY
/// CALLDATASIZE (out_len); PUSH2 0x0100 (out_offset); CALLDATASIZE (in_len);
/// PUSH1 0 (in_offset); PUSH1 0 (value); PUSH2 ECHO; GAS; CALL
/// PUSH2 0x00e0; MSTORE
/// CALLDATASIZE; PUSH1 32; ADD; PUSH2 0x00e0; RETURN
pub const FORWARD: &str = "3660006000373661010036600060006102015af16100e052366020016100e0f3";

/// A vicinity with no fees, for the tests.
pub fn vicinity() -> MemoryVicinity {
	MemoryVicinity {
		gas_price: U256::zero(),
		origin: CALLER,
		chain_id: U256::one(),
		block_hashes: Vec::new(),
		block_number: U256::zero(),
		block_coinbase: H160::zero(),
		block_timestamp: U256::zero(),
		block_difficulty: U256::zero(),
		block_gas_limit: U256::from(GAS_LIMIT),
		block_base_fee_per_gas: U256::zero(),
		block_randomness: None,
	}
}

/// An account holding `code`, with no balance and no storage.
pub fn account(code: Vec<u8>) -> MemoryAccount {
	MemoryAccount {
		nonce: U256::zero(),
		balance: U256::zero(),
		storage: BTreeMap::new(),
		code,
	}
}

/// The backend state holding each given account.
pub fn state(entries: &[(H160, Vec<u8>)]) -> BTreeMap<H160, MemoryAccount> {
	let mut state = BTreeMap::new();
	for (address, code) in entries {
		state.insert(*address, account(code.clone()));
	}
	state
}

/// Decode a hex string, ignoring whitespace.
pub fn decode_hex(code: &str) -> Vec<u8> {
	let code: String = code.chars().filter(|c| !c.is_whitespace()).collect();
	hex::decode(code).expect("the test bytecode is hex")
}

/// The 32-byte word holding `value`.
pub fn word(value: u64) -> Vec<u8> {
	let mut word = vec![0u8; 32];
	word[24..].copy_from_slice(&value.to_be_bytes());
	word
}

/// A payload of `len` bytes, with a distinct value in each byte.
pub fn payload(len: usize) -> Vec<u8> {
	(0..len).map(|index| index as u8 + 1).collect()
}

/// Run a transaction calling `CONTRACT` with `data`, without precompiles.
pub fn run(
	state: BTreeMap<H160, MemoryAccount>,
	data: Vec<u8>,
	config: &Config,
) -> (ExitReason, Vec<u8>) {
	run_with_precompiles(state, data, config, &())
}

/// Run a transaction calling `CONTRACT` with `data`, with a precompile set.
pub fn run_with_precompiles<P: PrecompileSet>(
	state: BTreeMap<H160, MemoryAccount>,
	data: Vec<u8>,
	config: &Config,
	precompiles: &P,
) -> (ExitReason, Vec<u8>) {
	let vicinity = vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);
	let metadata = StackSubstateMetadata::new(GAS_LIMIT, config);
	let state = MemoryStackState::new(metadata, &mut backend);
	let mut executor = StackExecutor::new_with_precompiles(state, config, precompiles);

	executor.transact_call(
		CALLER,
		CONTRACT,
		U256::zero(),
		data,
		GAS_LIMIT,
		Vec::new(),
		Vec::new(),
	)
}

/// The low two bytes of an address, as the tests push it.
pub fn address_word(address: H160) -> u16 {
	u16::from_be_bytes([address.0[18], address.0[19]])
}

/// Bytecode of a contract that copies its calldata into memory at 0, calls
/// `target` with the given input and output ranges, stores the call's success
/// flag in the word before the output range, and returns the flag followed by
/// the output range.
///
/// CALLDATASIZE; PUSH1 0; PUSH1 0; CALLDATACOPY
/// PUSH2 out_len; PUSH2 out_offset; PUSH2 in_len; PUSH2 in_offset
/// [PUSH1 0 (value);] PUSH2 target; GAS; <opcode>
/// PUSH2 flag_offset; MSTORE
/// PUSH2 (32 + out_len); PUSH2 flag_offset; RETURN
pub fn caller(
	opcode: u8,
	target: H160,
	in_offset: u16,
	in_len: u16,
	out_offset: u16,
	out_len: u16,
) -> Vec<u8> {
	assert!(
		out_offset >= 32,
		"the flag word is stored before the output range"
	);

	let flag_offset = out_offset - 32;
	let mut code = String::new();
	// CALLDATASIZE; PUSH1 0; PUSH1 0; CALLDATACOPY
	code += "366000600037";
	// PUSH2 out_len; PUSH2 out_offset; PUSH2 in_len; PUSH2 in_offset
	code += &format!("61{out_len:04x}61{out_offset:04x}61{in_len:04x}61{in_offset:04x}");
	// CALL and CALLCODE take a value; it is zero here
	if opcode == 0xf1 || opcode == 0xf2 {
		code += "6000";
	}
	// PUSH2 target; GAS
	code += &format!("61{:04x}5a", address_word(target));
	// CALL, CALLCODE, DELEGATECALL or STATICCALL
	code += &format!("{opcode:02x}");
	// PUSH2 flag_offset; MSTORE
	code += &format!("61{flag_offset:04x}52");
	// PUSH2 (32 + out_len); PUSH2 flag_offset; RETURN
	code += &format!("61{:04x}61{flag_offset:04x}f3", 32 + out_len);

	decode_hex(&code)
}
