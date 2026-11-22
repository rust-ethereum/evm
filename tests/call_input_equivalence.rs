//! What a callee reads as its input, and what the caller reads back after the
//! call, is the same whatever the input range is.

mod common;

use common::*;
use evm::executor::stack::{PrecompileFailure, PrecompileFn, PrecompileOutput};
use evm::{Config, Context, ExitReason, ExitSucceed};
use primitive_types::H160;
use std::collections::BTreeMap;
use std::sync::Mutex;

/// Assert that the caller returned a set success flag followed by `expected`.
fn assert_flag_and_output(output: &[u8], expected: &[u8]) {
	assert_eq!(output.len(), 32 + expected.len(), "the returned length");
	assert_eq!(&output[..32], &word(1)[..], "the call's success flag");
	assert_eq!(&output[32..], expected, "the bytes the call returned");
}

#[test]
fn a_fully_written_input_range_is_echoed() {
	let payload = payload(64);
	let code = caller(0xf1, ECHO_ADDR, 0, 64, 0x1000, 64);
	let state = state(&[(CONTRACT, code), (ECHO_ADDR, decode_hex(ECHO))]);

	let (reason, output) = run(state, payload.clone(), &Config::cancun());

	assert_eq!(reason, ExitReason::Succeed(ExitSucceed::Returned));
	assert_flag_and_output(&output, &payload);
}

#[test]
fn an_input_range_with_a_zero_tail_is_echoed() {
	// The calldata writes 32 bytes; the input range reads 64, so the tail of
	// the range lies past the physical memory.
	let payload = payload(32);
	let mut expected = payload.clone();
	expected.resize(64, 0);

	let code = caller(0xf1, ECHO_ADDR, 0, 64, 0x1000, 64);
	let state = state(&[(CONTRACT, code), (ECHO_ADDR, decode_hex(ECHO))]);

	let (reason, output) = run(state, payload, &Config::cancun());

	assert_eq!(reason, ExitReason::Succeed(ExitSucceed::Returned));
	assert_flag_and_output(&output, &expected);
}

#[test]
fn an_input_range_past_the_physical_memory_is_echoed_as_zeros() {
	let payload = payload(32);
	let expected = vec![0u8; 64];

	// The input range starts at 0x2000, past the 32 bytes the calldata wrote.
	let code = caller(0xf1, ECHO_ADDR, 0x2000, 64, 0x1000, 64);
	let state = state(&[(CONTRACT, code), (ECHO_ADDR, decode_hex(ECHO))]);

	let (reason, output) = run(state, payload, &Config::cancun());

	assert_eq!(reason, ExitReason::Succeed(ExitSucceed::Returned));
	assert_flag_and_output(&output, &expected);
}

#[test]
fn a_zero_length_input_at_an_unrepresentable_offset_is_empty() {
	// An offset above `usize::MAX`, so that a run which converts the offset
	// before it tests the length cannot pass.
	const ABOVE_USIZE_MAX: &str =
		"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

	// CALLDATASIZE; PUSH1 0; PUSH1 0; CALLDATACOPY
	// PUSH1 0 (out_len); PUSH2 0x0100 (out_offset); PUSH1 0 (in_len);
	// PUSH32 0xffff…ff (in_offset); PUSH1 0 (value); PUSH2 ECHO; GAS; CALL
	// MSIZE; PUSH2 0x00e0; MSTORE
	// PUSH2 0x0100; MSTORE
	// PUSH1 64; PUSH2 0x00e0; RETURN
	let code = decode_hex(&format!(
		"366000600037
		 6000 610100 6000 7f{ABOVE_USIZE_MAX} 6000 610201 5a f1
		 59 6100e052
		 61010052
		 6040 6100e0 f3"
	));
	let state = state(&[(CONTRACT, code), (ECHO_ADDR, decode_hex(ECHO))]);

	let (reason, output) = run(state, payload(32), &Config::cancun());

	assert_eq!(reason, ExitReason::Succeed(ExitSucceed::Returned));
	assert_eq!(output.len(), 64, "the returned length");
	// The calldata copy left the memory at one word, and the call did not
	// grow it, whatever offset it was given.
	assert_eq!(
		&output[..32],
		&word(32)[..],
		"the memory size after the call"
	);
	assert_eq!(&output[32..], &word(1)[..], "the call's success flag");
}

#[test]
fn a_second_call_reads_what_the_first_call_wrote_back() {
	// CALLDATASIZE; PUSH1 0; PUSH1 0; CALLDATACOPY
	// PUSH1 32 (out_len); PUSH1 0 (out_offset); PUSH1 32 (in_len);
	// PUSH1 0 (in_offset); PUSH1 0 (value); PUSH2 MARK; GAS; CALL; POP
	// PUSH1 32 (out_len); PUSH1 32 (out_offset); PUSH1 32 (in_len);
	// PUSH1 0 (in_offset); PUSH1 0 (value); PUSH2 ECHO; GAS; CALL; POP
	// PUSH1 64; PUSH1 0; RETURN
	let code = decode_hex(
		"366000600037
		 6020 6000 6020 6000 6000 610202 5a f1 50
		 6020 6020 6020 6000 6000 610201 5a f1 50
		 6040 6000 f3",
	);
	let state = state(&[
		(CONTRACT, code),
		(MARK_ADDR, decode_hex(MARK)),
		(ECHO_ADDR, decode_hex(ECHO)),
	]);

	let (reason, output) = run(state, payload(32), &Config::cancun());

	assert_eq!(reason, ExitReason::Succeed(ExitSucceed::Returned));

	// The first call writes the marker over the input range; the second call
	// reads that range and returns it into the second word.
	let mut expected = MARKER.to_vec();
	expected.extend_from_slice(&MARKER);
	assert_eq!(output, expected);
}

#[test]
fn a_nested_call_forwards_its_calldata() {
	let payload = payload(64);
	// The caller shares its input range with the forwarder, which in turn
	// shares its own memory with the echo callee, so two shared ranges are
	// alive at once. Its output range holds the forwarder's flag and bytes.
	let code = caller(0xf1, FORWARD_ADDR, 0, 64, 0x1000, 96);
	let state = state(&[
		(CONTRACT, code),
		(FORWARD_ADDR, decode_hex(FORWARD)),
		(ECHO_ADDR, decode_hex(ECHO)),
	]);

	let (reason, output) = run(state, payload.clone(), &Config::cancun());

	assert_eq!(reason, ExitReason::Succeed(ExitSucceed::Returned));

	// The caller's flag, then the forwarder's flag, then the bytes the echo
	// returned to the forwarder's own output range.
	assert_eq!(output.len(), 32 + 32 + 64, "the returned length");
	assert_eq!(&output[..32], &word(1)[..], "the caller's success flag");
	assert_eq!(
		&output[32..64],
		&word(1)[..],
		"the forwarder's success flag"
	);
	assert_eq!(&output[64..], &payload[..], "the bytes the echo returned");
}

#[test]
fn a_revert_writes_its_data_into_an_overlapping_range() {
	// CALLDATASIZE; PUSH1 0; PUSH1 0; CALLDATACOPY
	// PUSH1 32 (out_len); PUSH1 32 (out_offset); PUSH1 64 (in_len);
	// PUSH1 0 (in_offset); PUSH1 0 (value); PUSH2 REVERT; GAS; CALL; POP
	// PUSH1 0x22; PUSH1 0; MSTORE
	// PUSH1 64; PUSH1 0; RETURN
	let code = decode_hex(
		"366000600037
		 6020 6020 6040 6000 6000 610203 5a f1 50
		 6022 6000 52
		 6040 6000 f3",
	);
	let state = state(&[(CONTRACT, code), (REVERT_ADDR, decode_hex(REVERT))]);

	let (reason, output) = run(state, payload(64), &Config::cancun());

	assert_eq!(reason, ExitReason::Succeed(ExitSucceed::Returned));

	// The revert data lands in the second half of the input range, and the
	// caller's own write lands in the first half.
	let mut expected = word(0x22);
	expected.extend_from_slice(&REVERT_DATA);
	assert_eq!(output, expected);
}

#[test]
fn a_call_past_the_call_stack_limit_leaves_the_memory_writable() {
	let payload = payload(64);
	let mut config = Config::cancun();
	// The callee's frame is depth 1, so a limit of 0 rejects it.
	config.call_stack_limit = 0;

	// CALLDATASIZE; PUSH1 0; PUSH1 0; CALLDATACOPY
	// PUSH2 0x0140 (out_len); PUSH2 0x0100 (out_offset); PUSH1 64 (in_len);
	// PUSH1 0 (in_offset); PUSH1 0 (value); PUSH2 ECHO; GAS; CALL; POP
	// PUSH1 0x33; PUSH2 0x0120; MSTORE
	// PUSH2 0x0140; PUSH1 0; RETURN
	let code = decode_hex(
		"366000600037
		 610140 610100 6040 6000 6000 610201 5a f1 50
		 6033 610120 52
		 610140 6000 f3",
	);
	let state = state(&[(CONTRACT, code), (ECHO_ADDR, decode_hex(ECHO))]);

	let (reason, output) = run(state, payload.clone(), &config);

	assert_eq!(reason, ExitReason::Succeed(ExitSucceed::Returned));

	// The input range is intact, the failed call wrote nothing into its output
	// range, and the write after it landed.
	let mut expected = payload;
	expected.resize(0x120, 0);
	expected.extend_from_slice(&word(0x33));
	assert_eq!(output, expected);
}

#[test]
fn a_call_that_cannot_pay_leaves_the_memory_writable() {
	let payload = payload(64);

	// CALLDATASIZE; PUSH1 0; PUSH1 0; CALLDATACOPY
	// PUSH2 0x0140 (out_len); PUSH2 0x0100 (out_offset); PUSH1 64 (in_len);
	// PUSH1 0 (in_offset); PUSH1 1 (value); PUSH2 ECHO; GAS; CALL; POP
	// PUSH1 0x44; PUSH2 0x0120; MSTORE
	// PUSH2 0x0140; PUSH1 0; RETURN
	let code = decode_hex(
		"366000600037
		 610140 610100 6040 6000 6001 610201 5a f1 50
		 6044 610120 52
		 610140 6000 f3",
	);
	let state = state(&[(CONTRACT, code), (ECHO_ADDR, decode_hex(ECHO))]);

	// The contract holds no balance, so it cannot transfer the value.
	let (reason, output) = run(state, payload.clone(), &Config::cancun());

	assert_eq!(reason, ExitReason::Succeed(ExitSucceed::Returned));

	let mut expected = payload;
	expected.resize(0x120, 0);
	expected.extend_from_slice(&word(0x44));
	assert_eq!(output, expected);
}

#[test]
fn every_call_opcode_reads_the_same_input() {
	let payload = payload(64);

	// CALL, CALLCODE, DELEGATECALL and STATICCALL.
	for opcode in [0xf1, 0xf2, 0xf4, 0xfa] {
		let code = caller(opcode, ECHO_ADDR, 0, 64, 0x1000, 64);
		let state = state(&[(CONTRACT, code), (ECHO_ADDR, decode_hex(ECHO))]);

		let (reason, output) = run(state, payload.clone(), &Config::cancun());

		assert_eq!(
			reason,
			ExitReason::Succeed(ExitSucceed::Returned),
			"the exit reason of the {opcode:#04x} call"
		);
		assert_eq!(&output[..32], &word(1)[..], "the {opcode:#04x} flag");
		assert_eq!(&output[32..], &payload[..], "the {opcode:#04x} output");
	}
}

/// The input the test precompile last received.
static RECORDED: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// A precompile that records the input it receives and returns it.
fn record_and_echo(
	input: &[u8],
	_gas_limit: Option<u64>,
	_context: &Context,
	_is_static: bool,
) -> Result<(PrecompileOutput, u64), PrecompileFailure> {
	*RECORDED.lock().unwrap() = input.to_vec();

	Ok((
		PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: input.to_vec(),
		},
		0,
	))
}

#[test]
fn a_precompile_receives_the_input_range() {
	let payload = payload(64);
	let code = caller(0xf1, PRECOMPILE_ADDR, 0, 64, 0x1000, 64);
	let state = state(&[(CONTRACT, code)]);

	let mut precompiles: BTreeMap<H160, PrecompileFn> = BTreeMap::new();
	precompiles.insert(PRECOMPILE_ADDR, record_and_echo as PrecompileFn);

	let (reason, output) =
		run_with_precompiles(state, payload.clone(), &Config::cancun(), &precompiles);

	assert_eq!(reason, ExitReason::Succeed(ExitSucceed::Returned));
	assert_flag_and_output(&output, &payload);
	assert_eq!(*RECORDED.lock().unwrap(), payload);
}
