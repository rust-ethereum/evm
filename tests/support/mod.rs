//! Helpers for the call input tests.
//!
//! A test binary that uses this module declares `mod mock;` and then
//! `mod support;`.

#![allow(dead_code)]

pub mod recording;
pub mod wrap;

#[allow(unused_imports)]
pub use self::recording::RecordingInvoker;

use std::{cell::RefCell, convert::Infallible, thread};

#[allow(unused_imports)]
use evm::uint::U256Ext;
use evm::{
	HeapTransact,
	backend::OverlayedBackend,
	interpreter::{
		Capture, ExitError, ExitResult, ExitSucceed, Interpreter,
		etable::{Chained, Single},
		trap::CallCreateTrap,
	},
	standard::{
		Config, DispatchEtable, EtableResolver, Invoker, PrecompileSet, State, TransactArgs,
		TransactArgsCallCreate, TransactValue, TransactValueCallCreate, eval_gasometer,
	},
	uint::{H160, U256},
};

use crate::mock::{MockAccount, MockBackend};

/// Address of the transaction caller. It holds a large balance.
pub const CALLER: u64 = 0x01;
/// Address of the top-level contract that calls the looping contract.
pub const TOP: u64 = 0x0a00;
/// Address of the looping contract.
pub const LOOP: u64 = 0x0b00;
/// Address without code.
pub const NO_CODE: u64 = 0x0c00;
/// Contract that returns a constant word.
pub const RETURN_WORD: u64 = 0x0c01;
/// Contract that reverts with a constant word.
pub const REVERT_WORD: u64 = 0x0c02;
/// Address of the test precompile.
pub const PRECOMPILE: u64 = 0x0c03;
/// Address without code, for a call that carries a value.
pub const VALUE_TARGET: u64 = 0x0c04;
/// Contract that returns its calldata.
pub const ECHO: u64 = 0x0c05;
/// Contract that returns the length of its calldata as a word.
pub const CALLDATA_LEN: u64 = 0x0c06;
/// Contract that forwards its calldata to [`ECHO`].
pub const FORWARD: u64 = 0x0c07;
/// First address of the contracts that pass a call down a chain.
pub const CHAIN_BASE: u64 = 0x1000;
/// Memory length of the looping contract, 64 KiB.
pub const MEMORY_LEN: usize = 64 * 1024;
/// Number of calls that the looping contract repeats.
pub const CALL_COUNT: usize = 16;
/// Number of the contracts in the call chain that reaches the call stack limit.
pub const CHAIN_LEN: usize = 1024;
/// The constant word that [`RETURN_WORD`] returns and [`REVERT_WORD`] reverts with.
pub const WORD: u8 = 0xab;
/// The constant value that a call of the looping contract carries.
pub const VALUE: u64 = 1;
/// Gas limit of a transaction, enough for every test here.
pub const GAS_LIMIT: u64 = 1_000_000_000_000_000;

/// The hex of a `PUSH20` of the given address.
pub fn push_addr(address: u64) -> String {
	format!(
		"73{}",
		hex::encode(H160::from_low_u64_be(address).as_bytes())
	)
}

/// The hex of the shortest `PUSH` of the given value.
pub fn push_usize(value: usize) -> String {
	let bytes = value.to_be_bytes();
	let start = bytes
		.iter()
		.position(|byte| *byte != 0)
		.unwrap_or(bytes.len() - 1);
	let bytes = &bytes[start..];

	format!("{:02x}{}", 0x5f + bytes.len(), hex::encode(bytes))
}

/// The hex of a `PUSH32` of the given word.
pub fn push32(byte: u8) -> String {
	format!("7f{}", format!("{byte:02x}").repeat(32))
}

/// Decode hex fragments, which are concatenated first.
pub fn code(parts: &[&str]) -> Vec<u8> {
	hex::decode(parts.concat()).expect("the test bytecode is hex")
}

/// A word of 32 bytes of the given byte, as hex.
pub fn word_hex(byte: u8) -> String {
	format!("{byte:02x}").repeat(32)
}

/// The top-level contract: it calls `callee` without input and returns the
/// callee's return data.
///
/// ```text
/// 60 00     PUSH1 0      retLength
/// 60 00     PUSH1 0      retOffset
/// 60 00     PUSH1 0      argsLength
/// 60 00     PUSH1 0      argsOffset
/// 60 00     PUSH1 0      value
/// 73 ..     PUSH20 callee address
/// 5a        GAS          gas
/// f1        CALL
/// 50        POP          the call result
/// 3d        RETURNDATASIZE
/// 60 00     PUSH1 0      source offset
/// 60 00     PUSH1 0      target offset
/// 3e        RETURNDATACOPY
/// 3d        RETURNDATASIZE
/// 60 00     PUSH1 0      offset
/// f3        RETURN       return the callee's return data
/// ```
pub fn top_level(callee: u64) -> Vec<u8> {
	code(&[
		"6000",
		"6000",
		"6000",
		"6000",
		"6000",
		&push_addr(callee),
		"5a",
		"f1",
		"50",
		"3d",
		"6000",
		"6000",
		"3e",
		"3d",
		"6000",
		"f3",
	])
}

/// One iteration of the looping contract's loop: a `STATICCALL` with the input
/// range `0..in_len`, then two stores.
///
/// ```text
/// 60 20     PUSH1 32     retLength
/// 60 00     PUSH1 0      retOffset
/// 61 ....   PUSH2 in_len argsLength
/// 60 00     PUSH1 0      argsOffset
/// 73 ..     PUSH20 callee address
/// 5a        GAS          gas
/// fa        STATICCALL
/// 60 00     PUSH1 0      offset
/// 51        MLOAD        the word that the callee returned into 0..32
/// 61 01 00  PUSH2 0x100  offset
/// 52        MSTORE       memory[0x100..0x120) = that word
/// 60 00     PUSH1 0      offset
/// 52        MSTORE       memory[0..32) = the call result
/// ```
pub fn loop_body(in_len: usize, callee: u64) -> String {
	[
		"6020",
		"6000",
		&push_usize(in_len),
		"6000",
		&push_addr(callee),
		"5a",
		"fa",
		"6000",
		"51",
		"610100",
		"52",
		"6000",
		"52",
	]
	.concat()
}

/// One iteration of the looping contract's loop for a call that carries a
/// value, with the same stack as [`loop_body`] plus the value.
///
/// ```text
/// 60 20     PUSH1 32     retLength
/// 60 00     PUSH1 0      retOffset
/// 61 ....   PUSH2 in_len argsLength
/// 60 00     PUSH1 0      argsOffset
/// 60 01     PUSH1 1      value
/// 73 ..     PUSH20 callee address
/// 5a        GAS          gas
/// f1        CALL
/// 60 00     PUSH1 0      offset
/// 51        MLOAD        the word that the callee returned into 0..32
/// 61 01 00  PUSH2 0x100  offset
/// 52        MSTORE       memory[0x100..0x120) = that word
/// 60 00     PUSH1 0      offset
/// 52        MSTORE       memory[0..32) = the call result
/// ```
pub fn loop_body_with_value(in_len: usize, callee: u64) -> String {
	[
		"6020",
		"6000",
		&push_usize(in_len),
		"6000",
		"6001",
		&push_addr(callee),
		"5a",
		"f1",
		"6000",
		"51",
		"610100",
		"52",
		"6000",
		"52",
	]
	.concat()
}

/// The end of the looping contract: it returns `memory[0..64)`, the last call
/// result and the last word that a callee returned.
///
/// ```text
/// 61 01 00  PUSH2 0x100  offset
/// 51        MLOAD        the last word that a callee returned
/// 60 20     PUSH1 32     offset
/// 52        MSTORE       memory[32..64) = that word
/// 60 40     PUSH1 64     size
/// 60 00     PUSH1 0      offset
/// f3        RETURN       return memory[0..64)
/// ```
fn loop_end() -> Vec<u8> {
	code(&["610100", "51", "6020", "52", "6040", "6000", "f3"])
}

/// The looping contract: it grows its memory to [`MEMORY_LEN`] bytes, then
/// repeats [`CALL_COUNT`] calls with the input range `0..in_len`, and returns
/// the last call result and the last word that a callee returned.
///
/// ```text
/// 60 00     PUSH1 0         value
/// 61 ff e0  PUSH2 0xffe0    offset = MEMORY_LEN - 32
/// 52        MSTORE          memory[0xffe0..0x10000) = 0
/// <loop_body, CALL_COUNT times>
/// <loop_end>
/// ```
pub fn looping_contract(in_len: usize, callee: u64) -> Vec<u8> {
	let body = loop_body(in_len, callee).repeat(CALL_COUNT);
	let mut full = code(&["6000", "61ffe0", "52", &body]);
	full.extend(loop_end());
	full
}

/// The looping contract, for a call that carries [`VALUE`].
pub fn looping_contract_with_value(in_len: usize, callee: u64) -> Vec<u8> {
	let body = loop_body_with_value(in_len, callee).repeat(CALL_COUNT);
	let mut full = code(&["6000", "61ffe0", "52", &body]);
	full.extend(loop_end());
	full
}

/// One contract of a call chain: it calls `next` without input, and returns
/// the callee's return data.
pub fn chain_contract(next: u64) -> Vec<u8> {
	top_level(next)
}

/// The contract that returns a constant word of [`WORD`].
///
/// ```text
/// 7f ..     PUSH32 word   value
/// 60 00     PUSH1 0       offset
/// 52        MSTORE        memory[0..32) = word
/// 60 20     PUSH1 32      size
/// 60 00     PUSH1 0       offset
/// f3        RETURN        return memory[0..32)
/// ```
pub fn return_word_contract() -> Vec<u8> {
	code(&[&push32(WORD), "6000", "52", "6020", "6000", "f3"])
}

/// The contract that reverts with a constant word of [`WORD`].
///
/// ```text
/// 7f ..     PUSH32 word   value
/// 60 00     PUSH1 0       offset
/// 52        MSTORE        memory[0..32) = word
/// 60 20     PUSH1 32      size
/// 60 00     PUSH1 0       offset
/// fd        REVERT        revert with memory[0..32)
/// ```
pub fn revert_word_contract() -> Vec<u8> {
	code(&[&push32(WORD), "6000", "52", "6020", "6000", "fd"])
}

/// The contract that returns its whole calldata.
///
/// ```text
/// 36        CALLDATASIZE    size
/// 60 00     PUSH1 0         source offset
/// 60 00     PUSH1 0         target offset
/// 37        CALLDATACOPY    memory[0..size) = calldata[0..size)
/// 36        CALLDATASIZE    size
/// 60 00     PUSH1 0         offset
/// f3        RETURN          return memory[0..size)
/// ```
pub fn echo_contract() -> Vec<u8> {
	code(&["36", "6000", "6000", "37", "36", "6000", "f3"])
}

/// The contract that returns the length of its calldata as a word.
///
/// ```text
/// 36        CALLDATASIZE    size
/// 60 00     PUSH1 0         offset
/// 52        MSTORE          memory[0..32) = size
/// 60 20     PUSH1 32        size
/// 60 00     PUSH1 0         offset
/// f3        RETURN          return memory[0..32)
/// ```
pub fn calldata_len_contract() -> Vec<u8> {
	code(&["36", "6000", "52", "6020", "6000", "f3"])
}

/// The contract that forwards its calldata to `target` and returns what the
/// target returns.
///
/// ```text
/// 36        CALLDATASIZE    size
/// 60 00     PUSH1 0         source offset
/// 60 00     PUSH1 0         target offset
/// 37        CALLDATACOPY    memory[0..size) = calldata[0..size)
/// 36        CALLDATASIZE    retLength = size
/// 60 00     PUSH1 0         retOffset
/// 36        CALLDATASIZE    argsLength = size
/// 60 00     PUSH1 0         argsOffset
/// 73 ..     PUSH20 target   address
/// 5a        GAS             gas
/// fa        STATICCALL
/// 50        POP             the call result
/// 36        CALLDATASIZE    size
/// 60 00     PUSH1 0         offset
/// f3        RETURN          return memory[0..size)
/// ```
pub fn forwarding_contract(target: u64) -> Vec<u8> {
	code(&[
		"36",
		"6000",
		"6000",
		"37",
		"36",
		"6000",
		"36",
		"6000",
		&push_addr(target),
		"5a",
		"fa",
		"50",
		"36",
		"6000",
		"f3",
	])
}

/// The hex that writes a word of `byte` at `offset` in memory.
///
/// ```text
/// 7f ..     PUSH32 word   value
/// 60 ..     PUSH offset   offset
/// 52        MSTORE        memory[offset..offset + 32) = word
/// ```
pub fn store_word(byte: u8, offset: usize) -> String {
	[&push32(byte), &push_usize(offset), "52"].concat()
}

/// The hex that returns a range of memory.
///
/// ```text
/// 60 ..     PUSH size     size
/// 60 ..     PUSH offset   offset
/// f3        RETURN        return memory[offset..offset + size)
/// ```
pub fn return_range(offset: usize, len: usize) -> String {
	[&push_usize(len), &push_usize(offset), "f3"].concat()
}

/// The hex that stops the machine.
pub const STOP: &str = "00";

/// One call of a test contract.
#[derive(Clone, Copy, Debug)]
pub struct CallOp {
	/// The opcode of the call.
	pub opcode: u8,
	/// The callee's address.
	pub callee: u64,
	/// The input range.
	pub input: (usize, usize),
	/// The output range.
	pub output: (usize, usize),
	/// The value that the call carries, for `CALL` and `CALLCODE`.
	pub value: u64,
}

impl CallOp {
	/// A `STATICCALL` without value.
	///
	/// ```text
	/// 60 ..     PUSH retLength
	/// 60 ..     PUSH retOffset
	/// 60 ..     PUSH argsLength
	/// 60 ..     PUSH argsOffset
	/// 73 ..     PUSH20 callee
	/// 5a        GAS
	/// fa        STATICCALL
	/// ```
	pub const fn static_call(callee: u64, input: (usize, usize), output: (usize, usize)) -> Self {
		Self {
			opcode: 0xfa,
			callee,
			input,
			output,
			value: 0,
		}
	}

	/// A `CALL` without value.
	///
	/// ```text
	/// 60 ..     PUSH retLength
	/// 60 ..     PUSH retOffset
	/// 60 ..     PUSH argsLength
	/// 60 ..     PUSH argsOffset
	/// 60 00     PUSH1 0        value
	/// 73 ..     PUSH20 callee
	/// 5a        GAS
	/// f1        CALL
	/// ```
	pub const fn call(callee: u64, input: (usize, usize), output: (usize, usize)) -> Self {
		Self {
			opcode: 0xf1,
			callee,
			input,
			output,
			value: 0,
		}
	}

	/// A `CALL` that carries `value`.
	pub const fn call_with_value(
		callee: u64,
		input: (usize, usize),
		output: (usize, usize),
		value: u64,
	) -> Self {
		Self {
			opcode: 0xf1,
			callee,
			input,
			output,
			value,
		}
	}

	/// A `CALLCODE` without value.
	///
	/// ```text
	/// 60 ..     PUSH retLength
	/// 60 ..     PUSH retOffset
	/// 60 ..     PUSH argsLength
	/// 60 ..     PUSH argsOffset
	/// 60 00     PUSH1 0        value
	/// 73 ..     PUSH20 callee
	/// 5a        GAS
	/// f2        CALLCODE
	/// ```
	pub const fn call_code(callee: u64, input: (usize, usize), output: (usize, usize)) -> Self {
		Self {
			opcode: 0xf2,
			callee,
			input,
			output,
			value: 0,
		}
	}

	/// A `DELEGATECALL`.
	///
	/// ```text
	/// 60 ..     PUSH retLength
	/// 60 ..     PUSH retOffset
	/// 60 ..     PUSH argsLength
	/// 60 ..     PUSH argsOffset
	/// 73 ..     PUSH20 callee
	/// 5a        GAS
	/// f4        DELEGATECALL
	/// ```
	pub const fn delegate_call(callee: u64, input: (usize, usize), output: (usize, usize)) -> Self {
		Self {
			opcode: 0xf4,
			callee,
			input,
			output,
			value: 0,
		}
	}

	/// The hex of the call, which leaves the call result on the stack.
	pub fn hex(&self) -> String {
		let with_value = matches!(self.opcode, 0xf1 | 0xf2);

		[
			push_usize(self.output.1).as_str(),
			push_usize(self.output.0).as_str(),
			push_usize(self.input.1).as_str(),
			push_usize(self.input.0).as_str(),
			if with_value {
				push_usize(self.value as usize)
			} else {
				String::new()
			}
			.as_str(),
			push_addr(self.callee).as_str(),
			"5a",
			&format!("{:02x}", self.opcode),
		]
		.concat()
	}

	/// The hex of the call, and popping its result.
	pub fn hex_and_pop(&self) -> String {
		[&self.hex(), "50"].concat()
	}
}

/// The accounts of a chain of contracts that passes a call down to `target`,
/// and the address of its first contract.
pub fn chain(target: u64) -> (u64, Vec<(u64, Vec<u8>)>) {
	let mut accounts = Vec::new();

	for index in 0..CHAIN_LEN {
		let next = if index + 1 < CHAIN_LEN {
			CHAIN_BASE + index as u64 + 1
		} else {
			target
		};
		accounts.push((CHAIN_BASE + index as u64, chain_contract(next)));
	}

	(CHAIN_BASE, accounts)
}

/// The 32-byte words of the given bytes.
pub fn words(bytes: &[u8]) -> Vec<u8> {
	bytes.iter().flat_map(|byte| [*byte; 32]).collect()
}

/// The 32-byte big-endian word of the given value.
pub fn word(value: u64) -> Vec<u8> {
	let mut bytes = vec![0u8; 32];
	bytes[24..].copy_from_slice(&value.to_be_bytes());
	bytes
}

/// A mock backend that holds the given accounts, plus the caller with a large
/// balance.
pub fn backend(accounts: &[(u64, Vec<u8>)]) -> MockBackend {
	let mut backend = MockBackend::default();
	backend.state.insert(
		H160::from_low_u64_be(CALLER),
		MockAccount {
			balance: U256::from(GAS_LIMIT) * U256::from(1000),
			code: Vec::new(),
			nonce: U256::ONE,
			storage: Default::default(),
			transient_storage: Default::default(),
		},
	);
	for (address, code) in accounts {
		backend.state.insert(
			H160::from_low_u64_be(*address),
			MockAccount {
				balance: U256::ZERO,
				code: code.clone(),
				nonce: U256::ZERO,
				storage: Default::default(),
				transient_storage: Default::default(),
			},
		);
	}
	backend
}

/// The transaction arguments of a call to `address`.
pub fn call_args<'config>(
	config: &'config Config,
	address: u64,
	data: Vec<u8>,
) -> TransactArgs<'config> {
	TransactArgs {
		call_create: TransactArgsCallCreate::Call {
			address: H160::from_low_u64_be(address),
			data,
		},
		caller: H160::from_low_u64_be(CALLER),
		value: U256::ZERO,
		gas_limit: U256::from(GAS_LIMIT),
		gas_price: U256::ONE.into(),
		access_list: Vec::new(),
		config,
	}
}

/// The backend that the tests run against.
pub type TestBackend<'config> = OverlayedBackend<'config, MockBackend>;

/// The standard etable type of the tests.
pub type TestEtable<'config> = Chained<
	Single<State<'config>, TestBackend<'config>, CallCreateTrap>,
	DispatchEtable<'config, TestBackend<'config>>,
>;

/// The standard etable.
pub fn etable<'config>() -> TestEtable<'config> {
	let gasometer: evm::standard::Efn<'config, TestBackend<'config>> = eval_gasometer;
	Chained(Single::new(gasometer), DispatchEtable::runtime())
}

/// The standard resolver, over a precompile set.
pub fn resolver<'a, 'config, Pre>(
	etable: &'a TestEtable<'config>,
	precompiles: &'a Pre,
) -> EtableResolver<'a, 'a, Pre, TestEtable<'config>> {
	EtableResolver::new(precompiles, etable)
}

/// Run a transaction with the standard invoker.
pub fn transact_with<'config, Pre>(
	etable: &TestEtable<'config>,
	precompiles: &Pre,
	heap_depth: Option<usize>,
	backend: &mut TestBackend<'config>,
	args: TransactArgs<'config>,
) -> Result<TransactValue, ExitError>
where
	Pre: PrecompileSet<State<'config>, TestBackend<'config>>,
{
	let resolver = resolver(etable, precompiles);
	let invoker = Invoker::new(&resolver);

	evm::transact(args, heap_depth, backend, &invoker)
}

/// Run a transaction with the standard invoker, without precompiles.
pub fn transact_recursive<'config>(
	etable: &TestEtable<'config>,
	backend: &mut TestBackend<'config>,
	args: TransactArgs<'config>,
) -> Result<TransactValue, ExitError> {
	transact_with(etable, &(), None, backend, args)
}

/// Run a transaction with the standard invoker, with a heap depth.
pub fn transact_hybrid<'config>(
	etable: &TestEtable<'config>,
	heap_depth: usize,
	backend: &mut TestBackend<'config>,
	args: TransactArgs<'config>,
) -> Result<TransactValue, ExitError> {
	transact_with(etable, &(), Some(heap_depth), backend, args)
}

/// Run a transaction on the heap call stack, with the standard invoker.
pub fn heap_transact_with<'config, Pre>(
	etable: &TestEtable<'config>,
	precompiles: &Pre,
	backend: &mut TestBackend<'config>,
	args: TransactArgs<'config>,
) -> Result<TransactValue, ExitError>
where
	Pre: PrecompileSet<State<'config>, TestBackend<'config>>,
{
	let resolver = resolver(etable, precompiles);
	let invoker = Invoker::new(&resolver);

	match HeapTransact::new(args, &invoker, backend)?.run() {
		Capture::Exit(result) => result,
		Capture::Trap(infallible) => match *infallible {},
	}
}

/// Run a transaction on the heap call stack, with the standard invoker and no
/// precompiles.
pub fn heap_transact<'config>(
	etable: &TestEtable<'config>,
	backend: &mut TestBackend<'config>,
	args: TransactArgs<'config>,
) -> Result<TransactValue, ExitError> {
	heap_transact_with(etable, &(), backend, args)
}

/// The call stack that a run goes through.
#[derive(Clone, Copy, Debug)]
pub enum Path {
	/// `evm::transact` with no heap depth, so every call recurses.
	Recursive,
	/// `evm::transact` with a heap depth of 1, so the callees of the
	/// transaction run on the call stack.
	Hybrid,
	/// [HeapTransact] from the top.
	Heap,
}

impl Path {
	/// Every call stack of the tests.
	pub const ALL: [Self; 3] = [Self::Recursive, Self::Hybrid, Self::Heap];

	/// Name of the call stack.
	pub const fn name(self) -> &'static str {
		match self {
			Self::Recursive => "recursive",
			Self::Hybrid => "hybrid",
			Self::Heap => "heap",
		}
	}
}

/// Run a transaction with the standard invoker on the given call stack, and
/// return the result and the precompile set that it ran with.
pub fn run_path(
	accounts: Vec<(u64, Vec<u8>)>,
	entry: u64,
	path: Path,
	precompiles: TestPrecompiles,
) -> (Result<TransactValue, ExitError>, TestPrecompiles) {
	let config = Config::cancun();
	let etable = etable();
	let mut backend = OverlayedBackend::new(backend(&accounts), &config.runtime);
	let args = call_args(&config, entry, Vec::new());

	let result = match path {
		Path::Recursive => transact_with(&etable, &precompiles, None, &mut backend, args),
		Path::Hybrid => transact_with(&etable, &precompiles, Some(1), &mut backend, args),
		Path::Heap => heap_transact_with(&etable, &precompiles, &mut backend, args),
	};

	(result, precompiles)
}

/// Run the given function on a thread with a large stack, which a call chain of
/// the call stack limit needs.
pub fn on_a_large_stack<T, F>(f: F) -> T
where
	T: Send + 'static,
	F: FnOnce() -> T + Send + 'static,
{
	thread::Builder::new()
		.stack_size(64 * 1024 * 1024)
		.spawn(f)
		.expect("the test can start a thread")
		.join()
		.expect("the run does not panic")
}

/// Run a transaction with the given invoker.
pub fn transact_with_invoker<'config, I>(
	invoker: &I,
	heap_depth: Option<usize>,
	backend: &mut TestBackend<'config>,
	args: TransactArgs<'config>,
) -> Result<TransactValue, ExitError>
where
	I: evm::Invoker<
			TestBackend<'config>,
			TransactArgs = TransactArgs<'config>,
			TransactValue = TransactValue,
			Interrupt = Infallible,
		>,
	I::Interpreter: Interpreter<TestBackend<'config>>,
{
	evm::transact(args, heap_depth, backend, invoker)
}

/// Run a transaction on the heap call stack with the given invoker.
pub fn heap_transact_with_invoker<'config, I>(
	invoker: &I,
	backend: &mut TestBackend<'config>,
	args: TransactArgs<'config>,
) -> Result<TransactValue, ExitError>
where
	I: evm::Invoker<
			TestBackend<'config>,
			TransactArgs = TransactArgs<'config>,
			TransactValue = TransactValue,
			Interrupt = Infallible,
		>,
	I::Interpreter: Interpreter<TestBackend<'config>>,
{
	match HeapTransact::new(args, invoker, backend)?.run() {
		Capture::Exit(result) => result,
		Capture::Trap(infallible) => match *infallible {},
	}
}

/// The return data of a call transaction.
pub fn retval(value: &TransactValue) -> &[u8] {
	match &value.call_create {
		TransactValueCallCreate::Call { retval, .. } => retval,
		TransactValueCallCreate::Create { .. } => panic!("expected a call transaction"),
	}
}

/// The result of a call transaction.
pub fn succeed(value: &TransactValue) -> ExitSucceed {
	match &value.call_create {
		TransactValueCallCreate::Call { succeed, .. } => *succeed,
		TransactValueCallCreate::Create { .. } => panic!("expected a call transaction"),
	}
}

/// A precompile set for the tests. It answers at [`PRECOMPILE`] only.
pub struct TestPrecompiles {
	/// The inputs that the precompile received, when it records them.
	pub received: RefCell<Vec<Vec<u8>>>,
	/// The length of every input that the precompile received.
	pub lengths: RefCell<Vec<usize>>,
	/// Whether the precompile records its inputs.
	pub record: bool,
	/// The bytes that the precompile returns.
	pub response: Vec<u8>,
	/// Whether the precompile reports success.
	pub success: bool,
}

impl TestPrecompiles {
	/// A precompile that returns the given bytes and reports success, and
	/// records the inputs that it receives.
	pub fn new(response: Vec<u8>) -> Self {
		Self {
			received: RefCell::new(Vec::new()),
			lengths: RefCell::new(Vec::new()),
			record: true,
			response,
			success: true,
		}
	}

	/// A precompile that returns the given bytes and reports success, and
	/// records only the lengths of the inputs that it receives.
	pub fn lengths_only(response: Vec<u8>) -> Self {
		Self {
			record: false,
			..Self::new(response)
		}
	}

	/// The inputs that the precompile received so far.
	pub fn received(&self) -> Vec<Vec<u8>> {
		self.received.borrow().clone()
	}

	/// The lengths of the inputs that the precompile received so far.
	pub fn lengths(&self) -> Vec<usize> {
		self.lengths.borrow().clone()
	}
}

impl<S, H> PrecompileSet<S, H> for TestPrecompiles {
	fn execute(
		&self,
		code_address: H160,
		input: &[u8],
		_state: &mut S,
		_handler: &mut H,
	) -> Option<(ExitResult, Vec<u8>)> {
		if code_address != H160::from_low_u64_be(PRECOMPILE) {
			return None;
		}

		self.lengths.borrow_mut().push(input.len());
		if self.record {
			self.received.borrow_mut().push(input.to_vec());
		}

		let result = if self.success {
			Ok(ExitSucceed::Returned)
		} else {
			Err(ExitError::Reverted)
		};

		Some((result, self.response.clone()))
	}
}
