use std::{
	collections::{BTreeMap, BTreeSet},
	fs::{self, File},
	io::{BufReader, BufWriter},
};

use evm::{
	backend::{InMemoryAccount, InMemoryBackend, InMemoryEnvironment, OverlayedBackend},
	interpreter::{Capture, runtime::GasState, utils::u256_to_h256},
	standard::{Config, TransactArgs, TransactArgsCallCreate},
};
use evm_mainnet::with_mainnet_invoker;
use primitive_types::{H256, U256};

use crate::{
	error::{Error, TestError},
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

/// Don't do any config change.
pub fn empty_config_change(_config: &mut Config) {}

/// Disable EIP-7610.
pub fn disable_eip7610(config: &mut Config) {
	config.runtime.eip7610_create_check_storage = false;
}

/// Run tests for specific json file with debug flag
pub fn run_file(
	filename: &str,
	debug: bool,
	write_failed: Option<&str>,
	config_change: fn(&mut Config),
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
			match run_test(
				filename,
				&test_name,
				test.clone(),
				debug,
				write_failed,
				config_change,
			) {
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
	config_change: fn(&mut Config),
) -> Result<TestCompletionStatus, Error> {
	if fs::metadata(filename)?.is_dir() {
		let mut tests_status = TestCompletionStatus::default();

		for filename in fs::read_dir(filename)? {
			let filepath = filename?.path();
			let filename = filepath.to_str().ok_or(Error::NonUtf8Filename)?;

			if filename.ends_with(".json") {
				println!("RUN for: {filename}");
				tests_status += run_file(filename, debug, write_failed, config_change)?;
			}
		}
		tests_status.print_total_for_dir(filename);
		Ok(tests_status)
	} else {
		run_file(filename, debug, write_failed, config_change)
	}
}

/// Collect all test file names.
pub fn collect_test_files(folder: &str) -> Result<Vec<(String, String)>, Error> {
	let mut result = Vec::new();
	collect_test_files_to(folder, folder, &mut result)?;
	Ok(result)
}

fn collect_test_files_to(
	top_level: &str,
	folder: &str,
	result: &mut Vec<(String, String)>,
) -> Result<(), Error> {
	if fs::metadata(folder)?.is_dir() {
		for filename in fs::read_dir(folder)? {
			let filepath = filename?.path();
			let filename = filepath.to_str().ok_or(Error::NonUtf8Filename)?;

			collect_test_files_to(top_level, filename, result)?;
		}
	} else if folder.ends_with(".json") {
		let mut short_file_name = folder.to_string();
		if let Some(res) = short_file_name.strip_prefix(top_level) {
			short_file_name = res.to_string();
		}
		if let Some(res) = short_file_name.strip_suffix(".json") {
			short_file_name = res.to_string();
		}
		if let Some(res) = short_file_name.strip_prefix("/") {
			short_file_name = res.to_string();
		}
		short_file_name = short_file_name.replace("+", "_plus_");
		short_file_name = short_file_name.replace("^", "_pow_");
		short_file_name = short_file_name.replace("-", "_h_");
		let normalized_name = if short_file_name.is_empty() {
			"single".to_string()
		} else {
			short_file_name
		};
		result.push((normalized_name, folder.to_string()));
	}

	Ok(())
}

/// Run single test
pub fn run_test(
	_filename: &str,
	test_name: &str,
	test: TestData,
	debug: bool,
	write_failed: Option<&str>,
	config_change: fn(&mut Config),
) -> Result<(), Error> {
	let mut config = match test.fork {
		Fork::Frontier => Config::frontier(),
		Fork::Homestead => Config::homestead(),
		// Fork::EIP150 => Config::tangerine_whistle(),
		Fork::Istanbul => Config::istanbul(),
		_ => return Err(Error::UnsupportedFork),
	};
	config_change(&mut config);

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
			if test.env.current_number > U256::zero()
				&& let Some(previous_hash) = test.env.previous_hash
			{
				block_hashes.insert(test.env.current_number - U256::one(), previous_hash);
			}
			block_hashes
		},
		block_number: test.env.current_number,
		block_coinbase: test.env.current_coinbase,
		block_timestamp: test.env.current_timestamp,
		block_difficulty: test.env.current_difficulty,
		block_randomness: test.env.current_random,
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

	let args = if let Some(to) = test.transaction.to {
		TransactArgs {
			call_create: TransactArgsCallCreate::Call {
				address: to,
				data: test.transaction.data.clone(),
			},
			caller: test.transaction.sender,
			value: test
				.transaction
				.value
				.0
				.map_err(|()| TestError::UnexpectedDecoding)?,
			gas_limit: test.transaction.gas_limit,
			gas_price: test.transaction.gas_price,
			access_list: test
				.transaction
				.access_list
				.clone()
				.into_iter()
				.map(|access| (access.address, access.storage_keys))
				.collect(),
			config: &config,
		}
	} else {
		TransactArgs {
			call_create: TransactArgsCallCreate::Create {
				salt: None,
				init_code: test.transaction.data.clone(),
			},
			caller: test.transaction.sender,
			value: test
				.transaction
				.value
				.0
				.map_err(|()| TestError::UnexpectedDecoding)?,
			gas_limit: test.transaction.gas_limit,
			gas_price: test.transaction.gas_price,
			access_list: test
				.transaction
				.access_list
				.clone()
				.into_iter()
				.map(|access| (access.address, access.storage_keys))
				.collect(),
			config: &config,
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

	let mut run_backend =
		OverlayedBackend::new(&base_backend, initial_accessed.clone(), &config.runtime);
	let mut step_backend =
		OverlayedBackend::new(&base_backend, initial_accessed.clone(), &config.runtime);

	// Run
	let run_result = evm_mainnet::transact(args.clone(), &mut run_backend);
	let run_changeset = run_backend.deconstruct().1;
	let mut run_backend = base_backend.clone();
	run_backend.apply_overlayed(&run_changeset);

	// Step
	if debug {
		with_mainnet_invoker!(|invoker| {
			let _step_result = evm::HeapTransact::new(args, &invoker, &mut step_backend).and_then(
				|mut stepper| loop {
					{
						if let Some(machine) = stepper.last_interpreter() {
							println!(
								"pc: {}, opcode: {:?}, gas: 0x{:x}, stack: {:?}",
								machine.position(),
								machine.peek_opcode(),
								machine.as_ref().state.gas(),
								machine
									.as_ref()
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
		});
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
