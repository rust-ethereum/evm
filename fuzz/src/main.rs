#![no_main]

use arbitrary::Arbitrary;
use evm::{
	backend::{MemoryAccount, MemoryBackend, MemoryVicinity},
	executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata},
	Config,
};
use primitive_types::{H160, U256};
use std::{collections::BTreeMap, str::FromStr};

const NB_ACCOUNTS: u8 = 5;
const BALANCES: U256 = U256::MAX;

// Fuzzer input
#[derive(Arbitrary, Debug)]
struct EvmInput<'a> {
	transact_from: u8,
	transact_to: u8,
	transact_value: u64,
	smart_contract_code: &'a [u8],
	call_input: &'a [u8],
}

// Fuzzing harness macro, will create the targets for honggfuzz, libfuzzer and afl++
ziggy::fuzz!(|evminput: EvmInput| {
	// If we're not fuzzing, we print out information about the input
	#[cfg(not(fuzzing))]
	{
		println!("Running input:");
		println!("{evminput:?}");
	}

	let config = Config::frontier();

	let vicinity = MemoryVicinity {
		gas_price: U256::one(), // Should this be zero instead?
		origin: H160::default(),
		block_hashes: Vec::new(),
		block_number: Default::default(),
		block_coinbase: Default::default(),
		block_timestamp: Default::default(),
		block_difficulty: Default::default(),
		block_gas_limit: Default::default(),
		chain_id: U256::one(),
		block_base_fee_per_gas: U256::one(), // Should this be zero instead?
	};

	// Create initial state with NB_ACCOUNTS accounts
	let mut state = BTreeMap::new();
	for i in 1..NB_ACCOUNTS + 1 {
		state.insert(
			H160::from_str(&format!("0x{}000000000000000000000000000000000000000", i)).unwrap(),
			MemoryAccount {
				nonce: U256::one(),
				balance: BALANCES,
				storage: BTreeMap::new(),
				code: Vec::new(),
			},
		);
	}

	// Create one smart contract
	state.insert(
		H160::from_str(&format!(
			"0x{}000000000000000000000000000000000000000",
			NB_ACCOUNTS + 2
		))
		.unwrap(),
		MemoryAccount {
			nonce: U256::one(),
			balance: BALANCES,
			storage: BTreeMap::new(),
			code: evminput.smart_contract_code.to_vec(),
		},
	);

	let backend = MemoryBackend::new(&vicinity, state);
	let metadata = StackSubstateMetadata::new(u64::MAX, &config);
	let state = MemoryStackState::new(metadata, &backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);
	let from = H160::from_str(&format!(
		"0x{}000000000000000000000000000000000000000",
		evminput.transact_from % NB_ACCOUNTS + 2
	))
	.unwrap();
	let to = H160::from_str(&format!(
		"0x{}000000000000000000000000000000000000000",
		evminput.transact_to % NB_ACCOUNTS + 2
	))
	.unwrap();

	// Run the transaction
	let _reason = executor.transact_call(
		from,
		to,
		evminput.transact_value.into(),
		evminput.call_input.to_vec(),
		10_000_000_000,
		Vec::new(),
	);
});
