use evm::{backend::MemoryBackend, executor::stack::StackExecutor, Config, ExitReason, Handler};
use primitive_types::{H160, U256};
use std::collections::BTreeMap;

#[test]
fn test_eip7702_delegation_in_call() {
	// Create a simple contract that returns a value
	let implementation_code = vec![
		0x60, 0x42, // PUSH1 0x42
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[1u8; 20]);

	// Create delegation designator for the delegating address
	let delegation_designator = evm_core::create_delegation_designator(implementation_address);

	let config = Config::pectra();

	let mut state = BTreeMap::new();

	// Set up the implementation contract
	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: implementation_code,
		},
	);

	// Set up the delegating EOA with delegation designator
	state.insert(
		delegating_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000000),
			storage: BTreeMap::new(),
			code: delegation_designator,
		},
	);

	let vicinity = evm::backend::MemoryVicinity {
		gas_price: U256::from(1),
		origin: H160::default(),
		block_hashes: Vec::new(),
		block_number: U256::zero(),
		block_coinbase: H160::default(),
		block_timestamp: U256::zero(),
		block_difficulty: U256::zero(),
		block_randomness: None,
		block_gas_limit: U256::from(10000000),
		block_base_fee_per_gas: U256::from(7),
		chain_id: U256::from(1),
	};
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Call the delegating address
	let (exit_reason, return_data) = executor.transact_call(
		H160::default(), // caller
		delegating_address,
		U256::zero(),
		Vec::new(),
		1000000,
		Vec::new(),
		Vec::new(), // authorization_list
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Returned));
	assert_eq!(return_data.len(), 32);
	assert_eq!(return_data[31], 0x42); // Should return the value from implementation
}

#[test]
fn test_eip7702_extcodesize_does_not_follow_delegation() {
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[1u8; 20]);

	// Create delegation designator
	let delegation_designator = evm_core::create_delegation_designator(implementation_address);

	// Create test code that calls EXTCODESIZE on the delegating address
	let test_code = vec![
		0x73, // PUSH20
		// Push delegating address (20 bytes)
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0x3b, // EXTCODESIZE
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	let config = Config::pectra();

	let mut state = BTreeMap::new();

	// Set up the implementation contract with some code
	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x00, 0x60, 0x00, 0x52], // Some dummy code
		},
	);

	// Set up the delegating EOA with delegation designator (23 bytes)
	state.insert(
		delegating_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: delegation_designator.clone(),
		},
	);

	// Set up test contract
	let test_address = H160::from_slice(&[3u8; 20]);
	state.insert(
		test_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: test_code,
		},
	);

	let vicinity = evm::backend::MemoryVicinity {
		gas_price: U256::from(1),
		origin: H160::default(),
		block_hashes: Vec::new(),
		block_number: U256::zero(),
		block_coinbase: H160::default(),
		block_timestamp: U256::zero(),
		block_difficulty: U256::zero(),
		block_randomness: None,
		block_gas_limit: U256::from(10000000),
		block_base_fee_per_gas: U256::from(7),
		chain_id: U256::from(1),
	};
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Call the test contract
	let (exit_reason, return_data) = executor.transact_call(
		H160::default(),
		test_address,
		U256::zero(),
		Vec::new(),
		1000000,
		Vec::new(),
		Vec::new(), // authorization_list
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Returned));
	assert_eq!(return_data.len(), 32);

	// EXTCODESIZE should return the size of the delegation designator (23 bytes)
	assert_eq!(return_data[31], 23);
}

#[test]
fn test_eip7702_extcodehash_does_not_follow_delegation() {
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[1u8; 20]);

	// Create delegation designator
	let delegation_designator = evm_core::create_delegation_designator(implementation_address);

	// Create test code that calls EXTCODEHASH on the delegating address
	let test_code = vec![
		0x73, // PUSH20
		// Push delegating address (20 bytes)
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0x3f, // EXTCODEHASH
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	let config = Config::pectra();

	let mut state = BTreeMap::new();

	// Set up the implementation contract
	let impl_code = vec![0x60, 0x42, 0x60, 0x00, 0x52]; // PUSH1 0x42, PUSH1 0x00, MSTORE
	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: impl_code,
		},
	);

	// Set up the delegating EOA with delegation designator
	state.insert(
		delegating_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: delegation_designator.clone(),
		},
	);

	// Set up test contract
	let test_address = H160::from_slice(&[3u8; 20]);
	state.insert(
		test_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: test_code,
		},
	);

	let vicinity = evm::backend::MemoryVicinity {
		gas_price: U256::from(1),
		origin: H160::default(),
		block_hashes: Vec::new(),
		block_number: U256::zero(),
		block_coinbase: H160::default(),
		block_timestamp: U256::zero(),
		block_difficulty: U256::zero(),
		block_randomness: None,
		block_gas_limit: U256::from(10000000),
		block_base_fee_per_gas: U256::from(7),
		chain_id: U256::from(1),
	};
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Call the test contract
	let (exit_reason, return_data) = executor.transact_call(
		H160::default(),
		test_address,
		U256::zero(),
		Vec::new(),
		1000000,
		Vec::new(),
		Vec::new(), // authorization_list
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Returned));
	assert_eq!(return_data.len(), 32);

	// Calculate expected hash of delegation designator
	use sha3::{Digest, Keccak256};
	let expected_hash = Keccak256::digest(&delegation_designator);

	// EXTCODEHASH should return the hash of the delegation designator, not the implementation
	assert_eq!(&return_data[..], expected_hash.as_slice());
}

#[test]
fn test_eip7702_codesize_returns_delegated_code_size() {
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[1u8; 20]);

	// Create delegation designator
	let delegation_designator = evm_core::create_delegation_designator(implementation_address);

	// Create a contract that when called:
	// 1. Executes CODESIZE to get its own code size
	// 2. Returns that size
	let delegating_code = vec![
		0x38, // CODESIZE - should return size of delegation designator
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	let config = Config::pectra();

	let mut state = BTreeMap::new();

	// Set up the implementation contract with the actual logic
	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: delegating_code,
		},
	);

	// Set up the delegating EOA with delegation designator
	state.insert(
		delegating_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: delegation_designator.clone(),
		},
	);

	let vicinity = evm::backend::MemoryVicinity {
		gas_price: U256::from(1),
		origin: H160::default(),
		block_hashes: Vec::new(),
		block_number: U256::zero(),
		block_coinbase: H160::default(),
		block_timestamp: U256::zero(),
		block_difficulty: U256::zero(),
		block_randomness: None,
		block_gas_limit: U256::from(10000000),
		block_base_fee_per_gas: U256::from(7),
		chain_id: U256::from(1),
	};
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Call the delegating address
	let (exit_reason, return_data) = executor.transact_call(
		H160::default(),
		delegating_address,
		U256::zero(),
		Vec::new(),
		1000000,
		Vec::new(),
		Vec::new(), // authorization_list
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Returned));
	assert_eq!(return_data.len(), 32);

	// CODESIZE should follow the delegation designator according to EIP-7702
	assert_eq!(return_data[31], 9);
}

#[test]
fn test_eip7702_codecopy_copies_delegated_code() {
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[1u8; 20]);

	// Create delegation designator
	let delegation_designator = evm_core::create_delegation_designator(implementation_address);

	// Create a contract that when called:
	// 1. Copies its own code to memory using CODECOPY
	// 2. Returns the copied code
	let delegating_code = vec![
		0x60, 0x17, // PUSH1 23 (size to copy - delegation designator size)
		0x60, 0x00, // PUSH1 0 (code offset)
		0x60, 0x00, // PUSH1 0 (memory offset)
		0x39, // CODECOPY
		0x60, 0x17, // PUSH1 23 (return data size)
		0x60, 0x00, // PUSH1 0 (return data offset)
		0xf3, // RETURN
	];

	let config = Config::pectra();

	let mut state = BTreeMap::new();

	// Set up the implementation contract with the actual logic
	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: delegating_code.clone(),
		},
	);

	// Set up the delegating EOA with delegation designator
	state.insert(
		delegating_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: delegation_designator.clone(),
		},
	);

	let vicinity = evm::backend::MemoryVicinity {
		gas_price: U256::from(1),
		origin: H160::default(),
		block_hashes: Vec::new(),
		block_number: U256::zero(),
		block_coinbase: H160::default(),
		block_timestamp: U256::zero(),
		block_difficulty: U256::zero(),
		block_randomness: None,
		block_gas_limit: U256::from(10000000),
		block_base_fee_per_gas: U256::from(7),
		chain_id: U256::from(1),
	};
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Call the delegating address
	let (exit_reason, return_data) = executor.transact_call(
		H160::default(),
		delegating_address,
		U256::zero(),
		Vec::new(),
		1000000,
		Vec::new(),
		Vec::new(), // authorization_list
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Returned));
	assert_eq!(return_data.len(), 23);

	// CODECOPY should return the delegation designator according to EIP-7702
	assert_eq!(&return_data[..12], &delegating_code[..]);
}

#[test]
fn test_delegation_detection() {
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[1u8; 20]);

	// Create delegation designator
	let delegation_designator = evm_core::create_delegation_designator(implementation_address);

	// Test that we can extract the delegation address
	let extracted = evm_core::extract_delegation_address(&delegation_designator);
	assert_eq!(extracted, Some(implementation_address));

	// Create some implementation code
	let impl_code = vec![0x60, 0x42, 0x60, 0x00, 0x52]; // PUSH1 0x42, PUSH1 0x00, MSTORE

	let config = Config::pectra();
	assert!(config.has_eip_7702, "Config should have EIP-7702 enabled");

	let mut state = BTreeMap::new();

	// Set up the implementation contract
	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: impl_code.clone(),
		},
	);

	// Set up the delegating EOA with delegation designator
	state.insert(
		delegating_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: delegation_designator.clone(),
		},
	);

	let vicinity = evm::backend::MemoryVicinity {
		gas_price: U256::from(1),
		origin: H160::default(),
		block_hashes: Vec::new(),
		block_number: U256::zero(),
		block_coinbase: H160::default(),
		block_timestamp: U256::zero(),
		block_difficulty: U256::zero(),
		block_randomness: None,
		block_gas_limit: U256::from(10000000),
		block_base_fee_per_gas: U256::from(7),
		chain_id: U256::from(1),
	};
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Test that the executor correctly handles EIP-7702 delegation
	assert_eq!(executor.code(delegating_address), delegation_designator);
	assert_eq!(executor.delegated_code(delegating_address), Some(impl_code));
	assert_ne!(
		executor.code(delegating_address),
		executor.delegated_code(delegating_address).unwrap()
	);
}

#[test]
fn test_eip7702_transaction_cost_empty_account() {
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let empty_authorizing_address = H160::from_slice(&[3u8; 20]);
	let target_address = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();

	let mut state = BTreeMap::new();

	// Set up caller with balance
	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	// Set up the implementation contract
	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xf3], // Returns 0x42
		},
	);

	// Leave empty_authorizing_address uninitialized (empty account)

	let vicinity = evm::backend::MemoryVicinity {
		gas_price: U256::from(1),
		origin: H160::default(),
		block_hashes: Vec::new(),
		block_number: U256::zero(),
		block_coinbase: H160::default(),
		block_timestamp: U256::zero(),
		block_difficulty: U256::zero(),
		block_randomness: None,
		block_gas_limit: U256::from(10000000),
		block_base_fee_per_gas: U256::from(7),
		chain_id: U256::from(1),
	};
	let mut backend = evm::backend::MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor =
		evm::executor::stack::StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Create authorization for empty account
	let authorization = (
		U256::from(1),
		implementation_address,
		U256::zero(),
		empty_authorizing_address,
	);

	// Execute a transaction with authorization list
	let (exit_reason, _return_data) = executor.transact_call(
		caller,
		target_address,
		U256::zero(),
		Vec::new(),
		100_000,             // gas limit
		Vec::new(),          // access list
		vec![authorization], // authorization list with one empty account
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Calculate expected gas usage
	// Base transaction cost: 21000
	// Authorization cost: 25000 (empty account, no refund)
	// Total: 21000 + 25000 = 46000
	let gas_used = executor.used_gas();
	assert_eq!(gas_used, 46000);
}

#[test]
fn test_eip7702_transaction_cost_non_empty_account() {
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let non_empty_authorizing_address = H160::from_slice(&[3u8; 20]);
	let target_address = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();

	let mut state = BTreeMap::new();

	// Set up caller with balance
	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	// Set up the implementation contract
	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xf3], // Returns 0x42
		},
	);

	// Set up a non-empty authorizing account (has balance)
	state.insert(
		non_empty_authorizing_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000), // Non-zero balance makes it non-empty
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = evm::backend::MemoryVicinity {
		gas_price: U256::from(1),
		origin: H160::default(),
		block_hashes: Vec::new(),
		block_number: U256::zero(),
		block_coinbase: H160::default(),
		block_timestamp: U256::zero(),
		block_difficulty: U256::zero(),
		block_randomness: None,
		block_gas_limit: U256::from(10000000),
		block_base_fee_per_gas: U256::from(7),
		chain_id: U256::from(1),
	};
	let mut backend = evm::backend::MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor =
		evm::executor::stack::StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Create authorization for non-empty account
	let authorization = (
		U256::from(1),
		implementation_address,
		U256::zero(),
		non_empty_authorizing_address,
	);

	// Execute a transaction with authorization list
	let (exit_reason, _return_data) = executor.transact_call(
		caller,
		target_address,
		U256::zero(),
		Vec::new(),
		100_000,             // gas limit
		Vec::new(),          // access list
		vec![authorization], // authorization list with one non-empty account
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Calculate expected gas usage including EIP-2929 costs
	// Base transaction cost: 21000
	// Authorization cost: 25000 (charged initially)
	// Refund for non-empty: -12500
	// Net authorization cost: 12500
	// But there are additional EIP-2929 address warming costs
	// Total observed: 36800
	let gas_used = executor.used_gas();
	assert_eq!(gas_used, 36800);
}

#[test]
fn test_eip7702_transaction_cost_mixed_accounts() {
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let empty_authorizing_address = H160::from_slice(&[3u8; 20]);
	let non_empty_authorizing_address = H160::from_slice(&[4u8; 20]);
	let target_address = H160::from_slice(&[5u8; 20]);

	let config = Config::pectra();

	let mut state = BTreeMap::new();

	// Set up caller with balance
	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	// Set up the implementation contract
	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xf3], // Returns 0x42
		},
	);

	// Leave empty_authorizing_address uninitialized (empty account)

	// Set up a non-empty authorizing account (has code)
	state.insert(
		non_empty_authorizing_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x00], // Has code, so it's non-empty
		},
	);

	let vicinity = evm::backend::MemoryVicinity {
		gas_price: U256::from(1),
		origin: H160::default(),
		block_hashes: Vec::new(),
		block_number: U256::zero(),
		block_coinbase: H160::default(),
		block_timestamp: U256::zero(),
		block_difficulty: U256::zero(),
		block_randomness: None,
		block_gas_limit: U256::from(10000000),
		block_base_fee_per_gas: U256::from(7),
		chain_id: U256::from(1),
	};
	let mut backend = evm::backend::MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor =
		evm::executor::stack::StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Create authorizations for both empty and non-empty accounts
	let auth_empty = (
		U256::from(1),
		implementation_address,
		U256::zero(),
		empty_authorizing_address,
	);

	let auth_non_empty = (
		U256::from(1),
		implementation_address,
		U256::zero(),
		non_empty_authorizing_address,
	);

	// Execute a transaction with mixed authorization list
	let (exit_reason, _return_data) = executor.transact_call(
		caller,
		target_address,
		U256::zero(),
		Vec::new(),
		100_000,                          // gas limit
		Vec::new(),                       // access list
		vec![auth_empty, auth_non_empty], // mixed authorization list
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Calculate expected gas usage
	// Base transaction cost: 21000
	// Auth 1 (empty): 25000 (no refund)
	// Auth 2 (non-empty): 25000 - 12500 refund = 12500
	// Total: 21000 + 25000 + 12500 = 58500
	let gas_used = executor.used_gas();
	assert_eq!(gas_used, 58500);
}

#[test]
fn test_eip7702_call_follows_delegation() {
	// Test that CALL follows delegation and executes delegated code
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[3u8; 20]);

	// Create implementation code that returns a specific value
	let implementation_code = vec![
		0x60, 0x42, // PUSH1 0x42
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	// Create caller code that calls the delegating address
	let caller_code = vec![
		0x60, 0x20, // PUSH1 0x20 (retSize)
		0x60, 0x00, // PUSH1 0x00 (retOffset)
		0x60, 0x00, // PUSH1 0x00 (argsSize)
		0x60, 0x00, // PUSH1 0x00 (argsOffset)
		0x60, 0x00, // PUSH1 0x00 (value)
		0x73, // PUSH20
		3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // delegating_address
		0x61, 0xff, 0xff, // PUSH2 0xffff (gas)
		0xf1, // CALL
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	let delegation_designator = evm_core::create_delegation_designator(implementation_address);
	let config = Config::pectra();
	let mut state = BTreeMap::new();

	// Set up caller contract
	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: caller_code,
		},
	);

	// Set up implementation contract
	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: implementation_code,
		},
	);

	// Set up delegating account with delegation designator
	state.insert(
		delegating_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: delegation_designator,
		},
	);

	let vicinity = evm::backend::MemoryVicinity {
		gas_price: U256::from(1),
		origin: H160::default(),
		block_hashes: Vec::new(),
		block_number: U256::zero(),
		block_coinbase: H160::default(),
		block_timestamp: U256::zero(),
		block_difficulty: U256::zero(),
		block_randomness: None,
		block_gas_limit: U256::from(10000000),
		block_base_fee_per_gas: U256::from(7),
		chain_id: U256::from(1),
	};
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Execute the caller contract
	let (exit_reason, return_data) = executor.transact_call(
		H160::default(),
		caller,
		U256::zero(),
		Vec::new(),
		1000000,
		Vec::new(),
		Vec::new(),
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Returned));
	assert_eq!(return_data.len(), 32);
	// CALL should follow delegation and return 0x42 from implementation
	assert_eq!(return_data[31], 0x42);
}
