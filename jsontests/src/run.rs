use std::{
	collections::{BTreeMap, BTreeSet},
	fs::{self, File},
	io::BufReader,
};

use evm::{
	backend::OverlayedBackend,
	interpreter::{error::Capture, runtime::GasState, utils::u256_to_h256, Interpreter},
	standard::{Config, Etable, EtableResolver, Invoker, TransactArgs},
};
use evm_precompile::StandardPrecompileSet;
use primitive_types::U256;

use crate::{
	error::{Error, TestError},
	in_memory::{InMemoryAccount, InMemoryBackend, InMemoryEnvironment},
	types::{Fork, TestCompletionStatus, TestData, TestExpectException, TestMulti},
};

const BASIC_FILE_PATH_TO_TRIM: [&str; 2] = [
	"jsontests/res/ethtests/GeneralStateTests/",
	"res/ethtests/GeneralStateTests/",
];

fn get_short_file_name(filename: &str) -> String {
	let mut short_file_name = String::from(filename);
	for pattern in BASIC_FILE_PATH_TO_TRIM {
		short_file_name = short_file_name.replace(pattern, "");
	}
	short_file_name.clone().to_string()
}

/// Run tests for specific json file with debug flag
fn run_file(filename: &str, debug: bool) -> Result<TestCompletionStatus, Error> {
	let test_multi: BTreeMap<String, TestMulti> =
		serde_json::from_reader(BufReader::new(File::open(filename)?))?;
	let mut tests_status = TestCompletionStatus::default();

	for (test_name, test_multi) in test_multi {
		let tests = test_multi.tests();
		let short_file_name = get_short_file_name(filename);
		for test in &tests {
			if debug {
				print!(
					"[{:?}] {} | {}/{} DEBUG: ",
					test.fork, short_file_name, test_name, test.index
				);
			} else {
				print!(
					"[{:?}] {} | {}/{}: ",
					test.fork, short_file_name, test_name, test.index
				);
			}
			match run_test(filename, &test_name, test.clone(), debug) {
				Ok(()) => {
					tests_status.inc_completed();
					println!("ok")
				}
				Err(Error::UnsupportedFork) => {
					tests_status.inc_skipped();
					println!("skipped")
				}
				Err(err) => {
					println!("ERROR: {:?}", err);
					return Err(err);
				}
			}
			if debug {
				println!();
			}
		}

		tests_status.print_completion();
	}

	Ok(tests_status)
}

/// Run test for single json file or directory
pub fn run_single(filename: &str, debug: bool) -> Result<TestCompletionStatus, Error> {
	if fs::metadata(filename)?.is_dir() {
		let mut tests_status = TestCompletionStatus::default();

		for filename in fs::read_dir(filename)? {
			let filepath = filename?.path();
			let filename = filepath.to_str().ok_or(Error::NonUtf8Filename)?;
			println!("RUM for: {filename}");
			tests_status += run_file(filename, debug)?;
		}
		tests_status.print_total_for_dir(filename);
		Ok(tests_status)
	} else {
		run_file(filename, debug)
	}
}

/// Run single test
pub fn run_test(
	_filename: &str,
	_test_name: &str,
	test: TestData,
	debug: bool,
) -> Result<(), Error> {
	let config = match test.fork {
		Fork::Berlin => Config::berlin(),
		_ => return Err(Error::UnsupportedFork),
	};

	if test.post.expect_exception == Some(TestExpectException::TR_TypeNotSupported) {
		// The `evm` crate does not understand transaction format, only the `ethereum` crate. So
		// there's nothing for us to test here for `TR_TypeNotSupported`.
		return Ok(());
	}

	let env = InMemoryEnvironment {
		block_hashes: BTreeMap::new(), // TODO: fill in this field.
		block_number: test.env.current_number,
		block_coinbase: test.env.current_coinbase,
		block_timestamp: test.env.current_timestamp,
		block_difficulty: test.env.current_difficulty,
		block_randomness: Some(test.env.current_random),
		block_gas_limit: test.env.current_gas_limit,
		block_base_fee_per_gas: U256::zero(), // TODO: fill in this field.
		chain_id: U256::zero(),               // TODO: fill in this field.
	};

	let state = test
		.pre
		.clone()
		.into_iter()
		.map(|(address, account)| {
			let storage = account
				.storage
				.into_iter()
				.filter(|(_, value)| *value != U256::zero())
				.map(|(key, value)| (u256_to_h256(key), u256_to_h256(value)))
				.collect::<BTreeMap<_, _>>();

			(
				address,
				InMemoryAccount {
					balance: account.balance,
					code: account.code.0,
					nonce: account.nonce,
					storage,
				},
			)
		})
		.collect::<BTreeMap<_, _>>();

	let gas_etable = Etable::single(evm::standard::eval_gasometer);
	let exec_etable = Etable::runtime();
	let etable = (gas_etable, exec_etable);
	let precompiles = StandardPrecompileSet::new(&config);
	let resolver = EtableResolver::new(&config, &precompiles, &etable);
	let invoker = Invoker::new(&config, &resolver);
	let args = TransactArgs::Call {
		caller: test.transaction.sender,
		address: test.transaction.to,
		value: test.transaction.value,
		data: test.transaction.data,
		gas_limit: test.transaction.gas_limit,
		gas_price: test.transaction.gas_price,
		access_list: test
			.transaction
			.access_list
			.into_iter()
			.map(|access| (access.address, access.storage_keys))
			.collect(),
	};

	let initial_accessed = {
		let mut hots = BTreeSet::new();
		for i in 1..10 {
			hots.insert((u256_to_h256(U256::from(i)).into(), None));
		}
		hots
	};

	let base_backend = InMemoryBackend {
		environment: env,
		state,
	};

	let mut run_backend = OverlayedBackend::new(&base_backend, initial_accessed.clone());
	let mut step_backend = OverlayedBackend::new(&base_backend, initial_accessed.clone());

	// Run
	let run_result = evm::transact(args.clone(), Some(4), &mut run_backend, &invoker);
	let run_changeset = run_backend.deconstruct().1;
	let mut run_backend = base_backend.clone();
	run_backend.apply_overlayed(&run_changeset);

	// Step
	if debug {
		let _step_result = evm::HeapTransact::new(args, &invoker, &mut step_backend).and_then(
			|mut stepper| loop {
				{
					if let Some(machine) = stepper.last_interpreter() {
						println!(
							"pc: {}, opcode: {:?}, gas: 0x{:x}",
							machine.position(),
							machine.peek_opcode(),
							machine.machine().state.gas(),
						);
					}
				}
				if let Err(Capture::Exit(result)) = stepper.step() {
					break result;
				}
			},
		);
		let step_changeset = step_backend.deconstruct().1;
		let mut step_backend = base_backend.clone();
		step_backend.apply_overlayed(&step_changeset);
	}

	let state_root = crate::hash::state_root(&run_backend);

	if test.post.expect_exception.is_some() {
		if run_result.is_err() {
			return Ok(());
		} else {
			return Err(TestError::ExpectException.into());
		}
	}

	if state_root != test.post.hash {
		if debug {
			for (address, account) in &run_backend.state {
				println!(
					"address: {:?}, balance: {}, nonce: {}, code: 0x{}, storage: {:?}",
					address,
					account.balance,
					account.nonce,
					hex::encode(&account.code),
					account.storage
				);
			}
		}

		return Err(TestError::StateMismatch.into());
	}

	Ok(())
}
