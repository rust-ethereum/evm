use crate::error::{Error, TestError};
use crate::in_memory::{InMemoryAccount, InMemoryBackend, InMemoryEnvironment, InMemoryLayer};
use crate::types::*;
use evm::standard::{Config, Etable, EtableResolver, Gasometer, Invoker, TransactArgs};
use evm::utils::u256_to_h256;
use evm::Capture;
use evm_precompile::StandardPrecompileSet;
use primitive_types::U256;
use std::collections::{BTreeMap, BTreeSet};

pub fn run_test(_filename: &str, _test_name: &str, test: Test, debug: bool) -> Result<(), Error> {
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
					original_storage: storage.clone(),
					storage,
				},
			)
		})
		.collect::<BTreeMap<_, _>>();

	let etable = Etable::runtime();
	let precompiles = StandardPrecompileSet::new(&config);
	let resolver = EtableResolver::new(&config, &precompiles, &etable);
	let invoker = Invoker::<_, Gasometer, _, _, _>::new(&config, &resolver);
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

	let mut run_backend = InMemoryBackend {
		environment: env,
		layers: vec![InMemoryLayer {
			state,
			logs: Vec::new(),
			suicides: Vec::new(),
			hots: {
				let mut hots = BTreeSet::new();
				for i in 1..10 {
					hots.insert((u256_to_h256(U256::from(i)).into(), None));
				}
				hots
			},
		}],
	};
	let mut step_backend = run_backend.clone();

	// Run
	let run_result = evm::transact(args.clone(), Some(4), &mut run_backend, &invoker);
	run_backend.layers[0].clear_pending();

	// Step
	if debug {
		let _step_result = evm::HeapTransact::new(args, &invoker, &mut step_backend).and_then(
			|mut stepper| loop {
				{
					if let Some(machine) = stepper.last_machine() {
						println!(
							"pc: {}, opcode: {:?}, gas: 0x{:x}",
							machine.machine.position(),
							machine.machine.peek_opcode(),
							machine.gasometer.gas(),
						);
					}
				}
				if let Err(Capture::Exit(result)) = stepper.step() {
					break result;
				}
			},
		);
		step_backend.layers[0].clear_pending();
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
			for (address, account) in &run_backend.layers[0].state {
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
