use crate::error::{Error, TestError};
use crate::types::*;
use evm::backend::in_memory::{
	InMemoryAccount, InMemoryBackend, InMemoryEnvironment, InMemoryLayer,
};
use evm::standard::{Config, Etable, Invoker};
use evm::utils::u256_to_h256;
use primitive_types::U256;
use std::collections::{BTreeMap, BTreeSet};

pub fn run_test(_filename: &str, _test_name: &str, test: Test) -> Result<(), Error> {
	let config = match test.fork {
		Fork::Berlin => Config::berlin(),
		_ => return Err(Error::UnsupportedFork),
	};

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

	let mut backend = InMemoryBackend {
		environment: env,
		layers: vec![InMemoryLayer {
			state,
			logs: Vec::new(),
			suicides: Vec::new(),
			hots: BTreeSet::new(),
		}],
	};

	let etable = Etable::runtime();
	let invoker = Invoker::new(&config);
	let _result = invoker.transact_call(
		test.transaction.sender,
		test.transaction.to,
		test.transaction.value,
		test.transaction.data,
		test.transaction.gas_limit,
		test.transaction.gas_price,
		Vec::new(),
		&mut backend,
		&etable,
	);

	let state_root = crate::hash::state_root(&backend);

	if state_root != test.post.hash {
		return Err(TestError::StateMismatch.into());
	}

	Ok(())
}
