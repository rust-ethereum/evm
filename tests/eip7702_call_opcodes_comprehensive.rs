use evm::{
	backend::{Backend, MemoryBackend},
	executor::stack::StackExecutor,
	Config, ExitReason,
};
use primitive_types::{H160, H256, U256};
use std::collections::BTreeMap;

#[test]
fn test_eip7702_callcode_follows_delegation() {
	// Test that CALLCODE follows delegation but executes in caller's context
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[3u8; 20]);

	// Implementation code that returns ADDRESS and CALLER
	let implementation_code = vec![
		0x30, // ADDRESS
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x33, // CALLER
		0x60, 0x20, // PUSH1 0x20
		0x52, // MSTORE
		0x60, 0x40, // PUSH1 0x40
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	// Caller uses CALLCODE to call delegating address
	let caller_code = vec![
		0x60, 0x40, // PUSH1 0x40 (retSize)
		0x60, 0x00, // PUSH1 0x00 (retOffset)
		0x60, 0x00, // PUSH1 0x00 (argsSize)
		0x60, 0x00, // PUSH1 0x00 (argsOffset)
		0x60, 0x00, // PUSH1 0x00 (value)
		0x73, // PUSH20
		3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // delegating_address
		0x61, 0xff, 0xff, // PUSH2 0xffff (gas)
		0xf2, // CALLCODE
		0x60, 0x40, // PUSH1 0x40
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	let delegation_designator = evm_core::create_delegation_designator(implementation_address);
	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: caller_code,
		},
	);

	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: implementation_code,
		},
	);

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
	assert_eq!(return_data.len(), 64);

	// CALLCODE should execute in caller's context
	// ADDRESS should be caller (not delegating_address)
	let address_returned = H160::from_slice(&return_data[12..32]);
	assert_eq!(address_returned, caller);

	// CALLER should be caller (same as ADDRESS for CALLCODE)
	let caller_returned = H160::from_slice(&return_data[44..64]);
	assert_eq!(caller_returned, caller);
}

#[test]
fn test_eip7702_callcode_value_transfer() {
	// Test that CALLCODE transfers value within caller's address
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[3u8; 20]);

	// Implementation returns SELFBALANCE
	let implementation_code = vec![
		0x47, // SELFBALANCE
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	// Caller uses CALLCODE with value
	let caller_code = vec![
		0x60, 0x20, // PUSH1 0x20 (retSize)
		0x60, 0x00, // PUSH1 0x00 (retOffset)
		0x60, 0x00, // PUSH1 0x00 (argsSize)
		0x60, 0x00, // PUSH1 0x00 (argsOffset)
		0x61, 0x03, 0xe8, // PUSH2 1000 (value)
		0x73, // PUSH20
		3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // delegating_address
		0x61, 0xff, 0xff, // PUSH2 0xffff (gas)
		0xf2, // CALLCODE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	let delegation_designator = evm_core::create_delegation_designator(implementation_address);
	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: caller_code,
		},
	);

	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: implementation_code,
		},
	);

	state.insert(
		delegating_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(500),
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

	let initial_caller_balance = executor.state().basic(caller).balance;

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

	// CALLCODE should show caller's own balance (no external transfer)
	let returned_balance = U256::from_big_endian(&return_data);
	let final_caller_balance = executor.state().basic(caller).balance;

	// Caller balance should be unchanged (internal transfer)
	assert_eq!(final_caller_balance, initial_caller_balance);
	assert_eq!(returned_balance, initial_caller_balance);

	// Delegating address balance should be unchanged
	let delegating_balance = executor.state().basic(delegating_address).balance;
	assert_eq!(delegating_balance, U256::from(500));
}

// ======================================
// DELEGATECALL OPCODE TESTS WITH EIP-7702
// ======================================

#[test]
fn test_eip7702_delegatecall_follows_delegation() {
	// Test that DELEGATECALL follows delegation and preserves original context
	let original_caller = H160::from_slice(&[1u8; 20]);
	let intermediate_caller = H160::from_slice(&[2u8; 20]);
	let implementation_address = H160::from_slice(&[3u8; 20]);
	let delegating_address = H160::from_slice(&[4u8; 20]);

	// Implementation returns ADDRESS, CALLER, and CALLVALUE
	let implementation_code = vec![
		0x30, // ADDRESS
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x33, // CALLER
		0x60, 0x20, // PUSH1 0x20
		0x52, // MSTORE
		0x34, // CALLVALUE
		0x60, 0x40, // PUSH1 0x40
		0x52, // MSTORE
		0x60, 0x60, // PUSH1 0x60
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	// Intermediate caller uses DELEGATECALL
	let intermediate_code = vec![
		0x60, 0x60, // PUSH1 0x60 (retSize)
		0x60, 0x00, // PUSH1 0x00 (retOffset)
		0x60, 0x00, // PUSH1 0x00 (argsSize)
		0x60, 0x00, // PUSH1 0x00 (argsOffset)
		0x73, // PUSH20
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, // delegating_address
		0x61, 0xff, 0xff, // PUSH2 0xffff (gas)
		0xf4, // DELEGATECALL
		0x60, 0x60, // PUSH1 0x60
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	// Original caller calls intermediate with value
	let original_code = vec![
		0x60, 0x60, // PUSH1 0x60 (retSize)
		0x60, 0x00, // PUSH1 0x00 (retOffset)
		0x60, 0x00, // PUSH1 0x00 (argsSize)
		0x60, 0x00, // PUSH1 0x00 (argsOffset)
		0x61, 0x07, 0xd0, // PUSH2 2000 (value)
		0x73, // PUSH20
		2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // intermediate_caller
		0x61, 0xff, 0xff, // PUSH2 0xffff (gas)
		0xf1, // CALL
		0x60, 0x60, // PUSH1 0x60
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	let delegation_designator = evm_core::create_delegation_designator(implementation_address);
	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		original_caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: original_code,
		},
	);

	state.insert(
		intermediate_caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: intermediate_code,
		},
	);

	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: implementation_code,
		},
	);

	state.insert(
		delegating_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(500),
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

	let (exit_reason, return_data) = executor.transact_call(
		H160::default(),
		original_caller,
		U256::zero(),
		Vec::new(),
		1000000,
		Vec::new(),
		Vec::new(),
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Returned));
	assert_eq!(return_data.len(), 96);

	// ADDRESS should be intermediate_caller (DELEGATECALL preserves context)
	let address_returned = H160::from_slice(&return_data[12..32]);
	assert_eq!(address_returned, intermediate_caller);

	// CALLER should be original_caller (original context preserved)
	let caller_returned = H160::from_slice(&return_data[44..64]);
	assert_eq!(caller_returned, original_caller);

	// CALLVALUE should be 2000 (original value preserved)
	let value_returned = U256::from_big_endian(&return_data[64..96]);
	assert_eq!(value_returned, U256::from(2000));
}

#[test]
fn test_eip7702_delegatecall_storage_access() {
	// Test that DELEGATECALL with delegation accesses caller's storage
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[3u8; 20]);

	// Implementation code that writes to storage slot 0 and reads it back
	let implementation_code = vec![
		0x60, 0x42, // PUSH1 0x42 (value)
		0x60, 0x00, // PUSH1 0x00 (key)
		0x55, // SSTORE
		0x60, 0x00, // PUSH1 0x00 (key)
		0x54, // SLOAD
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	// Caller uses DELEGATECALL
	let caller_code = vec![
		0x60, 0x20, // PUSH1 0x20 (retSize)
		0x60, 0x00, // PUSH1 0x00 (retOffset)
		0x60, 0x00, // PUSH1 0x00 (argsSize)
		0x60, 0x00, // PUSH1 0x00 (argsOffset)
		0x73, // PUSH20
		3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // delegating_address
		0x61, 0xff, 0xff, // PUSH2 0xffff (gas)
		0xf4, // DELEGATECALL
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	let delegation_designator = evm_core::create_delegation_designator(implementation_address);
	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: caller_code,
		},
	);

	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: implementation_code,
		},
	);

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

	// Should return 0x42 (the value stored and loaded)
	assert_eq!(return_data[31], 0x42);

	// Verify that storage was written to caller's address
	let caller_storage = executor.state().storage(caller, H256::zero());
	assert_eq!(caller_storage, H256::from_low_u64_be(0x42));

	// Verify that delegating address storage is unchanged
	let delegating_storage = executor.state().storage(delegating_address, H256::zero());
	assert_eq!(delegating_storage, H256::zero());

	// Verify implementation storage is unchanged
	let impl_storage = executor
		.state()
		.storage(implementation_address, H256::zero());
	assert_eq!(impl_storage, H256::zero());
}

// ====================================
// STATICCALL OPCODE TESTS WITH EIP-7702
// ====================================

#[test]
fn test_eip7702_staticcall_follows_delegation() {
	// Test that STATICCALL follows delegation but prohibits state changes
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[3u8; 20]);

	// Implementation returns ADDRESS and tries to write storage (should fail in static context)
	let implementation_code = vec![
		0x30, // ADDRESS
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	// Caller uses STATICCALL
	let caller_code = vec![
		0x60, 0x20, // PUSH1 0x20 (retSize)
		0x60, 0x00, // PUSH1 0x00 (retOffset)
		0x60, 0x00, // PUSH1 0x00 (argsSize)
		0x60, 0x00, // PUSH1 0x00 (argsOffset)
		0x73, // PUSH20
		3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // delegating_address
		0x61, 0xff, 0xff, // PUSH2 0xffff (gas)
		0xfa, // STATICCALL
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	let delegation_designator = evm_core::create_delegation_designator(implementation_address);
	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: caller_code,
		},
	);

	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: implementation_code,
		},
	);

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

	// ADDRESS should be delegating_address (STATICCALL creates new context)
	let address_returned = H160::from_slice(&return_data[12..32]);
	assert_eq!(address_returned, delegating_address);
}

#[test]
fn test_eip7702_staticcall_prevents_state_changes() {
	// Test that STATICCALL with delegation prevents state changes
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[3u8; 20]);

	// Implementation tries to write to storage (should fail in static context)
	let implementation_code = vec![
		0x60, 0x42, // PUSH1 0x42 (value)
		0x60, 0x00, // PUSH1 0x00 (key)
		0x55, // SSTORE (should fail in static context)
		0x60, 0x01, // PUSH1 0x01 (success indicator)
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	// Caller uses STATICCALL
	let caller_code = vec![
		0x60, 0x20, // PUSH1 0x20 (retSize)
		0x60, 0x00, // PUSH1 0x00 (retOffset)
		0x60, 0x00, // PUSH1 0x00 (argsSize)
		0x60, 0x00, // PUSH1 0x00 (argsOffset)
		0x73, // PUSH20
		3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // delegating_address
		0x61, 0xff, 0xff, // PUSH2 0xffff (gas)
		0xfa, // STATICCALL
		// Check if call succeeded (should be 0 because of SSTORE)
		0x15, // ISZERO (check if call failed)
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	let delegation_designator = evm_core::create_delegation_designator(implementation_address);
	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: caller_code,
		},
	);

	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: implementation_code,
		},
	);

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

	// Should return 1 (true) indicating the STATICCALL failed due to SSTORE
	assert_eq!(return_data[31], 0x01);

	// Verify no storage was written to any address
	let caller_storage = executor.state().storage(caller, H256::zero());
	assert_eq!(caller_storage, H256::zero());

	let delegating_storage = executor.state().storage(delegating_address, H256::zero());
	assert_eq!(delegating_storage, H256::zero());

	let impl_storage = executor
		.state()
		.storage(implementation_address, H256::zero());
	assert_eq!(impl_storage, H256::zero());
}

#[test]
fn test_eip7702_staticcall_read_only_operations() {
	// Test that STATICCALL allows read-only operations with delegation
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[3u8; 20]);

	// Implementation returns a test value 0x42 to verify delegation works
	let implementation_code = vec![
		0x60, 0x42, // PUSH1 0x42
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	// Caller uses STATICCALL
	let caller_code = vec![
		0x60, 0x20, // PUSH1 0x20 (retSize)
		0x60, 0x00, // PUSH1 0x00 (retOffset)
		0x60, 0x00, // PUSH1 0x00 (argsSize)
		0x60, 0x00, // PUSH1 0x00 (argsOffset)
		0x73, // PUSH20
		3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // delegating_address
		0x61, 0xff, 0xff, // PUSH2 0xffff (gas)
		0xfa, // STATICCALL
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	let delegation_designator = evm_core::create_delegation_designator(implementation_address);
	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: caller_code,
		},
	);

	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: implementation_code,
		},
	);

	state.insert(
		delegating_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1500),
			storage: BTreeMap::new(),
			code: delegation_designator,
		},
	);

	let vicinity = evm::backend::MemoryVicinity {
		gas_price: U256::from(42),
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

	// Test delegation through caller contract
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

	// STATICCALL should follow delegation and return 0x42
	assert_eq!(return_data[31], 0x42);
}
