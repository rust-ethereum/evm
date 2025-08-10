use std::{
	collections::{BTreeMap, BTreeSet},
	fs::{self, File},
	io::{BufReader, BufWriter},
};

use evm::{
	backend::OverlayedBackend,
	interpreter::{
		error::Capture,
		etable::{Chained, Single},
		machine::AsMachine,
		runtime::GasState,
		utils::u256_to_h256,
	},
	standard::{Config, Etable, EtableResolver, Invoker, TransactArgs},
};
use evm_precompile::StandardPrecompileSet;
use primitive_types::{H256, U256};

use crate::{
	error::{Error, TestError},
	in_memory::{InMemoryAccount, InMemoryBackend, InMemoryEnvironment},
	types::{
		Fork, HexBytes, TestCompletionStatus, TestData, TestExpectException, TestMulti,
		TestMultiTransaction, TestPostStateIndexes,
	},
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
fn run_file(
	filename: &str,
	debug: bool,
	write_failed: Option<&str>,
) -> Result<TestCompletionStatus, Error> {
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
			match run_test(filename, &test_name, test.clone(), debug, write_failed) {
				Ok(()) => {
					tests_status.inc_completed();
					println!("ok")
				}
				Err(Error::UnsupportedFork) => {
					tests_status.inc_skipped();
					println!("skipped")
				}
				Err(err) => {
					println!("ERROR: {err:?}");
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
pub fn run_single(
	filename: &str,
	debug: bool,
	write_failed: Option<&str>,
) -> Result<TestCompletionStatus, Error> {
	if fs::metadata(filename)?.is_dir() {
		let mut tests_status = TestCompletionStatus::default();

		for filename in fs::read_dir(filename)? {
			let filepath = filename?.path();
			let filename = filepath.to_str().ok_or(Error::NonUtf8Filename)?;

			if filename.ends_with(".json") {
				println!("RUN for: {filename}");
				tests_status += run_file(filename, debug, write_failed)?;
			}
		}
		tests_status.print_total_for_dir(filename);
		Ok(tests_status)
	} else {
		run_file(filename, debug, write_failed)
	}
}

/// Run single test
pub fn run_test(
	_filename: &str,
	test_name: &str,
	test: TestData,
	debug: bool,
	write_failed: Option<&str>,
) -> Result<(), Error> {
	let config = match test.fork {
		Fork::Istanbul => Config::istanbul(),
		_ => return Err(Error::UnsupportedFork),
	};

	if test.post.expect_exception == Some(TestExpectException::TR_TypeNotSupported) {
		// The `evm` crate does not understand transaction format, only the `ethereum` crate. So
		// there's nothing for us to test here for `TR_TypeNotSupported`.
		return Ok(());
	}

	if test.post.expect_exception == Some(TestExpectException::TR_RLP_WRONGVALUE)
		&& test.transaction.value.0.is_err()
	{
		return Ok(());
	}

	let env = InMemoryEnvironment {
		block_hashes: {
			let mut block_hashes = BTreeMap::new();
			// Add the previous block hash to the block_hashes map
			// In EVM, BLOCKHASH opcode can access hashes of the last 256 blocks
			if test.env.current_number > U256::zero() {
				block_hashes.insert(
					test.env.current_number - U256::one(),
					test.env.previous_hash,
				);
			}
			block_hashes
		},
		block_number: test.env.current_number,
		block_coinbase: test.env.current_coinbase,
		block_timestamp: test.env.current_timestamp,
		block_difficulty: test.env.current_difficulty,
		block_randomness: Some(test.env.current_random),
		block_gas_limit: test.env.current_gas_limit,
		block_base_fee_per_gas: test.transaction.gas_price,
		chain_id: U256::one(),
	};

	let state = test
		.pre
		.clone()
		.into_iter()
		.map(|(address, account)| {
			let storage = account
				.storage
				.into_iter()
				.filter(|(_, value)| *value != H256::default())
				.collect::<BTreeMap<_, _>>();

			(
				address,
				InMemoryAccount {
					balance: account.balance,
					code: account.code.0,
					nonce: account.nonce,
					storage,
					transient_storage: Default::default(),
				},
			)
		})
		.collect::<BTreeMap<_, _>>();

	let gas_etable = Single::new(evm::standard::eval_gasometer);
	let exec_etable = Etable::runtime();
	let etable = Chained(gas_etable, exec_etable);
	let precompiles = StandardPrecompileSet::new(&config);
	let resolver = EtableResolver::new(&config, &precompiles, &etable);
	let invoker = Invoker::new(&config, &resolver);
	let args = if let Some(to) = test.transaction.to {
		TransactArgs::Call {
			caller: test.transaction.sender,
			address: to,
			value: test
				.transaction
				.value
				.0
				.map_err(|()| TestError::UnexpectedDecoding)?,
			data: test.transaction.data.clone(),
			gas_limit: test.transaction.gas_limit,
			gas_price: test.transaction.gas_price,
			access_list: test
				.transaction
				.access_list
				.clone()
				.into_iter()
				.map(|access| (access.address, access.storage_keys))
				.collect(),
		}
	} else {
		TransactArgs::Create {
			caller: test.transaction.sender,
			value: test
				.transaction
				.value
				.0
				.map_err(|()| TestError::UnexpectedDecoding)?,
			salt: None,
			init_code: test.transaction.data.clone(),
			gas_limit: test.transaction.gas_limit,
			gas_price: test.transaction.gas_price,
			access_list: test
				.transaction
				.access_list
				.clone()
				.into_iter()
				.map(|access| (access.address, access.storage_keys))
				.collect(),
		}
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

	let mut run_backend = OverlayedBackend::new(&base_backend, initial_accessed.clone(), &config);
	let mut step_backend = OverlayedBackend::new(&base_backend, initial_accessed.clone(), &config);

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
							"pc: {}, opcode: {:?}, gas: 0x{:x}, stack: {:?}",
							machine.position(),
							machine.peek_opcode(),
							machine.as_machine().state.gas(),
							machine
								.as_machine()
								.stack
								.data()
								.clone()
								.into_iter()
								.map(|v| format!("0x{v:x}"))
								.collect::<Vec<_>>(),
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
			println!(
				"test state root mismatch: {state_root:?} != {:?}",
				test.post.hash
			);

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

			if let Some(failed_file) = write_failed {
				let mut failed_multi = BTreeMap::new();
				let mut post_single = test.post.clone();
				post_single.indexes = TestPostStateIndexes {
					data: 0,
					gas: 0,
					value: 0,
				};
				let mut post = BTreeMap::new();
				post.insert(test.fork, vec![post_single]);
				failed_multi.insert(
					test_name,
					TestMulti {
						info: test.info,
						env: test.env,
						post,
						pre: test.pre,
						transaction: TestMultiTransaction {
							data: vec![HexBytes(test.transaction.data)],
							gas_limit: vec![test.transaction.gas_limit],
							gas_price: Some(test.transaction.gas_price),
							max_fee_per_gas: None,
							max_priority_fee_per_gas: test.transaction.gas_priority_fee,
							nonce: test.transaction.nonce,
							secret_key: test.transaction.secret_key,
							sender: test.transaction.sender,
							to: test.transaction.to,
							value: vec![test.transaction.value],
							access_lists: Some(vec![Some(test.transaction.access_list)]),
						},
					},
				);

				serde_json::to_writer_pretty(
					BufWriter::new(File::create(failed_file)?),
					&failed_multi,
				)?;
			}
		}

		return Err(TestError::StateMismatch.into());
	}

	Ok(())
}
