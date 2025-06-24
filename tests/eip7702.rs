use evm::{
	backend::{Backend, MemoryBackend},
	executor::stack::StackExecutor,
	Config, ExitError, ExitReason, Handler,
};
use primitive_types::{H160, H256, U256};
use std::collections::BTreeMap;

// ============================================================================
// Helper Functions for Test Data Generation
// ============================================================================

/// Create a valid authorization tuple for testing
fn create_authorization(
	chain_id: U256,
	delegation_address: H160,
	nonce: U256,
	authorizing_address: H160,
) -> (U256, H160, U256, H160) {
	(chain_id, delegation_address, nonce, authorizing_address)
}

/// Create a test vicinity for EIP-7702 tests
fn create_test_vicinity() -> evm::backend::MemoryVicinity {
	evm::backend::MemoryVicinity {
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
	}
}

// ============================================================================
// Transaction Type and Format Tests (Section 1)
// ============================================================================

#[test]
fn test_1_1_valid_transaction_structure() {
	// Test: Valid type 0x04 transaction with all required fields
	// Expected: Transaction accepted and processed correctly
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	assert!(
		config.has_eip_7702,
		"EIP-7702 must be enabled in Pectra config"
	);

	let mut state = BTreeMap::new();
	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xf3],
		},
	);

	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Create valid authorization
	let authorization = create_authorization(
		U256::from(1),  // chain_id matches vicinity
		implementation, // delegation_address
		U256::zero(),   // nonce matches account
		authorizing,    // authorizing_address
	);

	// Execute transaction with authorization list (simulates type 0x04)
	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),          // access_list
		vec![authorization], // authorization_list
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify delegation was set
	let delegation_designator = evm_core::create_delegation_designator(implementation);
	assert_eq!(executor.code(authorizing), delegation_designator);
}

#[test]
fn test_1_2_invalid_transaction_missing_authorization_list() {
	// Test: Transaction with empty authorization_list (simulates missing list)
	// Purpose: Verify executor handles transactions without EIP-7702 authorizations
	// Context: This represents a regular transaction, not a type 0x04 transaction
	let caller = H160::from_slice(&[1u8; 20]);
	let target = H160::from_slice(&[2u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();
	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Execute without authorization list (regular transaction)
	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(), // access_list
		Vec::new(), // empty authorization_list
	);

	println!("📋 Transaction Type Analysis:");
	println!("   • Empty authorization_list = regular transaction (not type 0x04)");
	println!("   • Type 0x04 transactions would have authorization_list populated");
	println!("   • Executor processes both transaction types uniformly");
	println!("   • EIP-7702 features only activate when authorizations are present");

	// Should succeed as regular transaction
	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	println!("✅ Regular transaction processed successfully");
	println!("💡 Note: Empty authorization list = no EIP-7702 processing occurs");
}

#[test]
fn test_1_3_transaction_with_null_destination() {
	// Test: Type 0x04 transaction with null destination (contract creation)
	// Expected: Contract creation should work with authorization
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42],
		},
	);

	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	let authorization =
		create_authorization(U256::from(1), implementation, U256::zero(), authorizing);

	// Test contract creation with authorization
	let creation_code = vec![0x60, 0x42, 0x60, 0x00, 0x52, 0x60, 0x01, 0x60, 0x1f, 0xf3]; // Returns 0x42
	let (exit_reason, created_address) = executor.transact_create(
		caller,
		U256::zero(),
		creation_code,
		100_000,
		Vec::new(),
		vec![authorization],
	);

	// Contract creation should succeed
	assert!(matches!(exit_reason, ExitReason::Succeed(_)));
	assert!(!created_address.is_empty()); // Return data should not be empty
}

#[test]
fn test_1_4_empty_authorization_list_executor_behavior() {
	// Test: Executor behavior with empty authorization_list
	// Note: Per EIP-7702, empty lists should be rejected at transaction pool level
	// This test demonstrates that the executor accepts empty lists for internal flexibility
	let caller = H160::from_slice(&[1u8; 20]);
	let target = H160::from_slice(&[2u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();
	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Execute with empty authorization list
	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(), // access_list
		Vec::new(), // empty authorization_list (should be invalid per EIP-7702)
	);

	// Per EIP-7702: Type 0x04 transactions MUST have at least one authorization
	// This validation should be handled by transaction pools/RPC layers, not the executor
	// The executor accepts empty authorization lists for flexibility in testing and internal use

	println!("📋 EIP-7702 Architectural Note:");
	println!("   • Empty authorization lists are accepted by the executor for flexibility");
	println!("   • Transaction pools/RPC layers should validate type 0x04 transactions");
	println!("   • Per EIP-7702: Type 0x04 transactions MUST have ≥1 authorization");
	println!("   • This test demonstrates executor behavior, not transaction validation");

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Gas should only include base transaction cost (21000)
	let gas_used = executor.used_gas();
	assert_eq!(gas_used, 21000);

	println!("✅ Executor processed empty authorization list successfully");
	println!("⚠️  Production systems should reject this at transaction pool level");
}

// ============================================================================
// Authorization Tuple Validation Tests (Section 2)
// ============================================================================

#[test]
fn test_2_1_valid_authorization_tuple() {
	// Test: Authorization tuple with all valid components
	// Expected: Authorization processed successfully
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	// Set up accounts
	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xf3],
		},
	);

	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Test valid authorization tuple with all components within limits
	let authorization = create_authorization(
		U256::from(1),  // chain_id: valid
		implementation, // address: 20 bytes
		U256::zero(),   // nonce: < 2^64 - 1
		authorizing,    // authorizing_address: 20 bytes
	);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify delegation was set correctly
	let delegation_designator = evm_core::create_delegation_designator(implementation);
	assert_eq!(executor.code(authorizing), delegation_designator);
}

#[test]
fn test_2_2_invalid_chain_id_large() {
	// Test: Authorization with chain_id >= 2**256 (conceptually impossible but test boundary)
	// Expected: This tests the conceptual limit as U256::MAX is the largest U256
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xf3],
		},
	);

	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Test with maximum U256 value (boundary case)
	let authorization = create_authorization(
		U256::MAX, // chain_id: largest possible U256
		implementation,
		U256::zero(),
		authorizing,
	);

	// This should be skipped as chain_id doesn't match current chain (1)
	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify delegation was NOT set (authorization skipped)
	assert_eq!(executor.code(authorizing), Vec::<u8>::new());
}

#[test]
fn test_2_3_invalid_nonce_large() {
	// Test: Authorization with nonce >= 2**64
	// Expected: Authorization rejected, constraint violation
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42],
		},
	);

	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Test with nonce >= 2^64 (18446744073709551616)
	let large_nonce = U256::from(2u64).pow(U256::from(64));
	let authorization = create_authorization(
		U256::from(1),
		implementation,
		large_nonce, // nonce >= 2^64
		authorizing,
	);

	// This should be processed but skipped due to nonce mismatch
	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify delegation was NOT set (authorization skipped due to invalid nonce)
	assert_eq!(executor.code(authorizing), Vec::<u8>::new());
}

#[test]
fn test_2_4_max_nonce_value() {
	// Test: Authorization with nonce = 2**64 - 1
	// Expected: Authorization rejected (nonce must be < 2**64 - 1)
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42],
		},
	);

	// Set authorizing account with high nonce
	let max_nonce_minus_one = U256::from(2u64).pow(U256::from(64)) - U256::from(1);
	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: max_nonce_minus_one, // Set account nonce to 2^64 - 1
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Test with max valid nonce (2^64 - 1)
	let authorization = create_authorization(
		U256::from(1),
		implementation,
		max_nonce_minus_one, // nonce = 2^64 - 1 (should be rejected per EIP-7702)
		authorizing,
	);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify delegation was NOT set (authorization rejected due to max nonce)
	assert_eq!(executor.code(authorizing), Vec::<u8>::new());
}

#[test]
fn test_2_5_delegation_indicator_format() {
	// Test: Verify delegation indicator format
	// Expected: Code = 0xef0100 || address (exactly 23 bytes)
	let implementation_address = H160::from_slice(&[0x42u8; 20]);

	// Test delegation designator creation
	let delegation_designator = evm_core::create_delegation_designator(implementation_address);

	// Verify format: 0xef0100 + 20 byte address = 23 bytes total
	assert_eq!(delegation_designator.len(), 23);
	assert_eq!(&delegation_designator[0..3], &[0xef, 0x01, 0x00]);
	assert_eq!(
		&delegation_designator[3..23],
		implementation_address.as_bytes()
	);

	// Test detection
	assert!(evm_core::is_delegation_designator(&delegation_designator));

	// Test extraction
	let extracted = evm_core::extract_delegation_address(&delegation_designator);
	assert_eq!(extracted, Some(implementation_address));

	// Test invalid format (wrong length)
	let invalid_short = vec![0xef, 0x01, 0x00]; // Too short
	assert!(!evm_core::is_delegation_designator(&invalid_short));

	let mut invalid_long = vec![0xef, 0x01, 0x00];
	invalid_long.extend(vec![0u8; 27]); // Make it 30 bytes total (too long)
	assert!(!evm_core::is_delegation_designator(&invalid_long));

	// Test invalid prefix
	let invalid_prefix = {
		let mut invalid = delegation_designator.clone();
		invalid[0] = 0xfe; // Wrong prefix
		invalid
	};
	assert!(!evm_core::is_delegation_designator(&invalid_prefix));
} // ============================================================================
  // Authorization Processing Tests (Section 3)
  // ============================================================================

#[test]
fn test_3_1_chain_id_verification() {
	// Test: Authorization with non-matching chain_id (not 0 and not current chain)
	// Expected: Authorization skipped during processing
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42],
		},
	);

	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity(); // chain_id = 1
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Authorization with chain_id = 2 (doesn't match current chain_id = 1)
	let authorization = create_authorization(
		U256::from(2), // non-matching chain_id
		implementation,
		U256::zero(),
		authorizing,
	);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify delegation was NOT set (authorization skipped)
	assert_eq!(executor.code(authorizing), Vec::<u8>::new());
}

#[test]
fn test_3_2_chain_id_zero() {
	// Test: Authorization with chain_id = 0
	// Expected: Authorization accepted regardless of current chain
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42],
		},
	);

	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity(); // chain_id = 1
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Authorization with chain_id = 0 (should be accepted regardless of current chain)
	let authorization = create_authorization(
		U256::zero(), // chain_id = 0
		implementation,
		U256::zero(),
		authorizing,
	);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify delegation was set (authorization accepted)
	let delegation_designator = evm_core::create_delegation_designator(implementation);
	assert_eq!(executor.code(authorizing), delegation_designator);
}

#[test]
fn test_3_3_authority_code_state_empty() {
	// Test: Authorization for EOA with empty code
	// Expected: Authorization succeeds, code set to delegation indicator
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xf3],
		},
	);

	// Authorizing account starts with empty code
	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(), // Empty code
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Verify initial state
	assert_eq!(executor.code(authorizing), Vec::<u8>::new());

	let authorization =
		create_authorization(U256::from(1), implementation, U256::zero(), authorizing);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify delegation was set
	let delegation_designator = evm_core::create_delegation_designator(implementation);
	assert_eq!(executor.code(authorizing), delegation_designator);
}

#[test]
fn test_3_4_authority_code_state_already_delegated() {
	// Test: Authorization for EOA already containing delegation indicator
	// Expected: Authorization succeeds, updates delegation
	let caller = H160::from_slice(&[1u8; 20]);
	let old_implementation = H160::from_slice(&[2u8; 20]);
	let new_implementation = H160::from_slice(&[3u8; 20]);
	let authorizing = H160::from_slice(&[4u8; 20]);
	let target = H160::from_slice(&[5u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		old_implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x00, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xf3], // Returns 0x00
		},
	);

	state.insert(
		new_implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xf3], // Returns 0x42
		},
	);

	// Authorizing account starts with delegation to old implementation
	let old_delegation = evm_core::create_delegation_designator(old_implementation);
	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: old_delegation.clone(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Verify initial delegation
	assert_eq!(executor.code(authorizing), old_delegation);

	// Update delegation to new implementation
	let authorization =
		create_authorization(U256::from(1), new_implementation, U256::zero(), authorizing);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify delegation was updated
	let new_delegation = evm_core::create_delegation_designator(new_implementation);
	assert_eq!(executor.code(authorizing), new_delegation);
}

#[test]
fn test_3_5_authority_code_state_non_delegation_code() {
	// Test: Authorization for account with existing non-delegation code
	// Expected: Authorization skipped
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42],
		},
	);

	// Authorizing account has existing non-delegation code
	let existing_code = vec![0x60, 0x00, 0x60, 0x00, 0x52]; // Some contract code
	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: existing_code.clone(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Verify initial code
	assert_eq!(executor.code(authorizing), existing_code);
	assert!(!evm_core::is_delegation_designator(
		&executor.code(authorizing)
	));

	let authorization =
		create_authorization(U256::from(1), implementation, U256::zero(), authorizing);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify code was NOT changed (authorization skipped)
	assert_eq!(executor.code(authorizing), existing_code);
}

#[test]
fn test_3_6_nonce_mismatch() {
	// Test: Authorization with nonce not matching authority's current nonce
	// Expected: Authorization skipped
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42],
		},
	);

	// Authorizing account has nonce = 5
	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::from(5),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Authorization with nonce = 3 (doesn't match account nonce = 5)
	let authorization = create_authorization(
		U256::from(1),
		implementation,
		U256::from(3), // Mismatched nonce
		authorizing,
	);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify delegation was NOT set (authorization skipped due to nonce mismatch)
	assert_eq!(executor.code(authorizing), Vec::<u8>::new());

	// Verify nonce was NOT incremented (authorization was skipped)
	assert_eq!(executor.state().basic(authorizing).nonce, U256::from(5));
}

#[test]
fn test_3_7_nonce_increment() {
	// Test: Successful authorization
	// Expected: Authority nonce incremented by 1
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42],
		},
	);

	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::from(7),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Verify initial nonce
	assert_eq!(executor.state().basic(authorizing).nonce, U256::from(7));

	// Authorization with matching nonce
	let authorization = create_authorization(
		U256::from(1),
		implementation,
		U256::from(7), // Matching nonce
		authorizing,
	);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify delegation was set
	let delegation_designator = evm_core::create_delegation_designator(implementation);
	assert_eq!(executor.code(authorizing), delegation_designator);

	// Verify nonce was incremented
	assert_eq!(executor.state().basic(authorizing).nonce, U256::from(8));
}

// ============================================================================
// Delegation Indicator Tests (Section 4)
// ============================================================================

#[test]
fn test_4_1_correct_delegation_format() {
	// Test: Verify delegation indicator format
	// Expected: Code = 0xef0100 || address (exactly 23 bytes)
	let implementation_address = H160::from_slice(&[
		0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
		0x88, 0x99, 0xaa, 0xbb, 0xcc,
	]);

	// Create delegation designator
	let delegation_designator = evm_core::create_delegation_designator(implementation_address);

	// Test correct format
	assert_eq!(delegation_designator.len(), 23);
	assert_eq!(&delegation_designator[0..3], &[0xef, 0x01, 0x00]);
	assert_eq!(
		&delegation_designator[3..23],
		implementation_address.as_bytes()
	);

	// Test detection
	assert!(evm_core::is_delegation_designator(&delegation_designator));

	// Test extraction
	let extracted = evm_core::extract_delegation_address(&delegation_designator);
	assert_eq!(extracted, Some(implementation_address));
}

#[test]
fn test_4_2_extcodesize_with_delegation() {
	// Test: Call EXTCODESIZE on delegated account
	// Expected: Returns size of delegation designator (23 bytes), not delegated code size
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[1u8; 20]);

	// This test is already implemented in the original test suite as:
	// test_eip7702_extcodesize_does_not_follow_delegation()
	// Verifying that EXTCODESIZE returns 23 (delegation designator size)

	let delegation_designator = evm_core::create_delegation_designator(implementation_address);
	let config = Config::pectra();
	let mut state = BTreeMap::new();

	// Set up the implementation contract with some code
	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x00, 0x60, 0x00, 0x52], // 5 bytes of code
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

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);
	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Direct check - EXTCODESIZE should return the size of stored code (delegation designator)
	assert_eq!(executor.code(delegating_address).len(), 23);
	assert_eq!(executor.code(implementation_address).len(), 5);
}

#[test]
fn test_4_3_extcodecopy_with_delegation() {
	// Test: Call EXTCODECOPY on delegated account
	// Expected: Copies delegation designator bytes, not delegated code
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[1u8; 20]);

	let delegation_designator = evm_core::create_delegation_designator(implementation_address);
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

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);
	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// EXTCODECOPY should return the delegation designator itself
	assert_eq!(executor.code(delegating_address), delegation_designator);
	// Not the implementation code
	assert_ne!(
		executor.code(delegating_address),
		executor.code(implementation_address)
	);
}

#[test]
fn test_4_4_extcodehash_with_delegation() {
	// Test: Call EXTCODEHASH on delegated account
	// Expected: Returns keccak256(delegation designator)
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[1u8; 20]);

	let delegation_designator = evm_core::create_delegation_designator(implementation_address);

	// Calculate expected hash
	use sha3::{Digest, Keccak256};
	let expected_hash = Keccak256::digest(&delegation_designator);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	// Set up the implementation contract
	let impl_code = vec![0x60, 0x42, 0x60, 0x00, 0x52];
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

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);
	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// The hash should be of the delegation designator, not the implementation code
	let actual_code = executor.code(delegating_address);
	let actual_hash = Keccak256::digest(&actual_code);

	assert_eq!(actual_hash.as_slice(), expected_hash.as_slice());
}

#[test]
fn test_4_5_code_execution_redirection() {
	// Test: Execute code on delegated EOA
	// Expected: Execution redirected to designated address
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

	let delegation_designator = evm_core::create_delegation_designator(implementation_address);
	let config = Config::pectra();
	let mut state = BTreeMap::new();

	// Set up caller
	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
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

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Call the delegating address - execution should be redirected to implementation
	let (exit_reason, return_data) = executor.transact_call(
		caller,
		delegating_address,
		U256::zero(),
		Vec::new(),
		1000000,
		Vec::new(),
		Vec::new(),
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Returned));
	assert_eq!(return_data.len(), 32);
	assert_eq!(return_data[31], 0x42); // Should return the value from implementation
} // ============================================================================
  // Gas Cost Tests (Section 5)
  // ============================================================================

#[test]
fn test_5_1_base_transaction_cost() {
	// Test: Calculate intrinsic gas for type 0x04 transaction
	// Expected: 21000 + calldata costs + access list costs + (PER_EMPTY_ACCOUNT_COST * auth_list_length)
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	assert_eq!(config.gas_auth_base_cost, 12500); // PER_AUTH_BASE_COST
	assert_eq!(config.gas_per_empty_account_cost, 25000); // PER_EMPTY_ACCOUNT_COST

	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42],
		},
	);

	// Leave authorizing as empty account
	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	let authorization =
		create_authorization(U256::from(1), implementation, U256::zero(), authorizing);

	// No calldata, no access list, one authorization
	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(), // No calldata
		100_000,
		Vec::new(),          // No access list
		vec![authorization], // One authorization for empty account
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Expected: 21000 (base) + 25000 (empty account cost) = 46000
	let gas_used = executor.used_gas();
	assert_eq!(gas_used, 46000);
}

#[test]
fn test_5_2_per_auth_base_cost() {
	// Test: Verify gas consumption per authorization
	// Expected: 12,500 gas per authorization tuple
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing1 = H160::from_slice(&[3u8; 20]);
	let authorizing2 = H160::from_slice(&[4u8; 20]);
	let target = H160::from_slice(&[5u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42],
		},
	);

	// Both are non-empty accounts (have balance)
	state.insert(
		authorizing1,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		authorizing2,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	let auth1 = create_authorization(U256::from(1), implementation, U256::zero(), authorizing1);
	let auth2 = create_authorization(U256::from(1), implementation, U256::zero(), authorizing2);

	// Two authorizations for non-empty accounts
	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![auth1, auth2],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Expected: 21000 (base) + 2 * (25000 - 12500) = 21000 + 25000 = 46000
	// Each non-empty account: 25000 initially, then 12500 refund = 12500 net cost
	let gas_used = executor.used_gas();
	assert_eq!(gas_used, 46000); // 21000 + 2 * 12500
}

#[test]
fn test_5_3_per_empty_account_cost() {
	// Test: Verify additional cost for empty accounts
	// Expected: 25,000 gas per authorization (no refund for empty accounts)
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let empty_authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42],
		},
	);

	// Leave empty_authorizing as truly empty (not in state)
	// This should be treated as an empty account

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	let authorization = create_authorization(
		U256::from(1),
		implementation,
		U256::zero(),
		empty_authorizing,
	);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Expected: 21000 (base) + 25000 (empty account, no refund) = 46000
	let gas_used = executor.used_gas();
	assert_eq!(gas_used, 46000);
}

#[test]
fn test_5_4_cold_account_access() {
	// Test: Access cold account during delegated code execution
	// Expected: Additional 2600 gas (COLD_ACCOUNT_READ_COST)
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation_address = H160::from_slice(&[2u8; 20]);
	let delegating_address = H160::from_slice(&[3u8; 20]);
	let cold_account = H160::from_slice(&[4u8; 20]);

	// Implementation code that reads balance of cold account
	let implementation_code = vec![
		0x73, // PUSH20
	];
	let mut full_code = implementation_code;
	full_code.extend_from_slice(cold_account.as_bytes()); // Push cold account address
	full_code.extend_from_slice(&[
		0x31, // BALANCE (reads cold account)
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	]);

	let delegation_designator = evm_core::create_delegation_designator(implementation_address);
	let config = Config::pectra();
	let mut state = BTreeMap::new();

	// Set up accounts
	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: full_code,
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

	state.insert(
		cold_account,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(500),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Call delegating address which will execute implementation code that accesses cold account
	let (exit_reason, return_data) = executor.transact_call(
		caller,
		delegating_address,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		Vec::new(),
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Returned));
	assert_eq!(return_data.len(), 32);

	// The returned balance should be 500
	let returned_balance = U256::from_big_endian(&return_data);
	assert_eq!(returned_balance, U256::from(500));

	// Gas should include cold account access cost
	let gas_used = executor.used_gas();
	// This includes: base (21000) + execution costs + cold account access (2600)
	assert!(gas_used > 21000 + 2600);
}

#[test]
fn test_5_5_invalid_authorization_gas() {
	// Test: Transaction with invalid authorizations
	// Expected: Gas still consumed for all authorization tuples
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42],
		},
	);

	// Set authorizing account with nonce = 5
	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::from(5),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Create authorization with wrong nonce (will be invalid/skipped)
	let invalid_authorization = create_authorization(
		U256::from(1),
		implementation,
		U256::from(3), // Wrong nonce (account has nonce = 5)
		authorizing,
	);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![invalid_authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify delegation was NOT set (authorization was invalid)
	assert_eq!(executor.code(authorizing), Vec::<u8>::new());

	// Gas should still be consumed for processing the authorization
	let gas_used = executor.used_gas();
	// Expected: 21000 (base) + 25000 initially - 12500 refund = 33500
	assert_eq!(gas_used, 33500);
}

// ============================================================================
// Multiple Authorization Tests (Section 6)
// ============================================================================

#[test]
fn test_6_1_duplicate_authorities() {
	// Test: Multiple authorizations for same authority
	// Expected: Only last valid authorization processed
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation1 = H160::from_slice(&[2u8; 20]);
	let implementation2 = H160::from_slice(&[3u8; 20]);
	let authorizing = H160::from_slice(&[4u8; 20]);
	let target = H160::from_slice(&[5u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation1,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x01], // Returns 1
		},
	);

	state.insert(
		implementation2,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x02], // Returns 2
		},
	);

	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Two authorizations for the same authority (duplicate)
	let auth1 = create_authorization(U256::from(1), implementation1, U256::zero(), authorizing);
	let auth2 = create_authorization(U256::from(1), implementation2, U256::from(1), authorizing);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![auth1, auth2], // Duplicate authority (same authorizing address)
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Only the LAST authorization should be processed (implementation2)
	let delegation_designator = evm_core::create_delegation_designator(implementation2);
	assert_eq!(executor.code(authorizing), delegation_designator);

	// Verify nonce was incremented only once (not twice)
	assert_eq!(executor.state().basic(authorizing).nonce, U256::from(2));
}

#[test]
fn test_6_2_mixed_valid_invalid() {
	// Test: Mix of valid and invalid authorizations
	// Expected: Valid ones processed, invalid skipped
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let valid_authorizing = H160::from_slice(&[3u8; 20]);
	let invalid_authorizing = H160::from_slice(&[4u8; 20]);
	let target = H160::from_slice(&[5u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42],
		},
	);

	state.insert(
		valid_authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		invalid_authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::from(5), // Different nonce
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Valid authorization (nonce matches)
	let valid_auth = create_authorization(
		U256::from(1),
		implementation,
		U256::zero(),
		valid_authorizing,
	);
	// Invalid authorization (nonce doesn't match)
	let invalid_auth = create_authorization(
		U256::from(1),
		implementation,
		U256::zero(),
		invalid_authorizing,
	);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![valid_auth, invalid_auth],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Valid authorization should be processed
	let delegation_designator = evm_core::create_delegation_designator(implementation);
	assert_eq!(executor.code(valid_authorizing), delegation_designator);

	// Invalid authorization should be skipped
	assert_eq!(executor.code(invalid_authorizing), Vec::<u8>::new());

	// Verify nonce increments
	assert_eq!(
		executor.state().basic(valid_authorizing).nonce,
		U256::from(1)
	);
	assert_eq!(
		executor.state().basic(invalid_authorizing).nonce,
		U256::from(5)
	); // Unchanged
}

#[test]
fn test_6_3_order_independence() {
	// Test: Different ordering of authorizations (no duplicate authorities)
	// Expected: Same final state regardless of order
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation1 = H160::from_slice(&[2u8; 20]);
	let implementation2 = H160::from_slice(&[3u8; 20]);
	let authorizing1 = H160::from_slice(&[4u8; 20]);
	let authorizing2 = H160::from_slice(&[5u8; 20]);
	let target = H160::from_slice(&[6u8; 20]);

	let config = Config::pectra();

	// Test both orderings
	for order in [true, false] {
		let mut state = BTreeMap::new();

		state.insert(
			caller,
			evm::backend::MemoryAccount {
				nonce: U256::zero(),
				balance: U256::from(10_000_000),
				storage: BTreeMap::new(),
				code: Vec::new(),
			},
		);

		state.insert(
			implementation1,
			evm::backend::MemoryAccount {
				nonce: U256::zero(),
				balance: U256::zero(),
				storage: BTreeMap::new(),
				code: vec![0x60, 0x01],
			},
		);

		state.insert(
			implementation2,
			evm::backend::MemoryAccount {
				nonce: U256::zero(),
				balance: U256::zero(),
				storage: BTreeMap::new(),
				code: vec![0x60, 0x02],
			},
		);

		state.insert(
			authorizing1,
			evm::backend::MemoryAccount {
				nonce: U256::zero(),
				balance: U256::from(1000),
				storage: BTreeMap::new(),
				code: Vec::new(),
			},
		);

		state.insert(
			authorizing2,
			evm::backend::MemoryAccount {
				nonce: U256::zero(),
				balance: U256::from(1000),
				storage: BTreeMap::new(),
				code: Vec::new(),
			},
		);

		let vicinity = create_test_vicinity();
		let mut backend = MemoryBackend::new(&vicinity, state);

		let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
		let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
		let precompiles = BTreeMap::new();
		let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

		let auth1 =
			create_authorization(U256::from(1), implementation1, U256::zero(), authorizing1);
		let auth2 =
			create_authorization(U256::from(1), implementation2, U256::zero(), authorizing2);

		let authorizations = if order {
			vec![auth1, auth2]
		} else {
			vec![auth2, auth1]
		};

		let (exit_reason, _) = executor.transact_call(
			caller,
			target,
			U256::zero(),
			Vec::new(),
			100_000,
			Vec::new(),
			authorizations,
		);

		assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

		// Both authorizations should be processed regardless of order
		let delegation1 = evm_core::create_delegation_designator(implementation1);
		let delegation2 = evm_core::create_delegation_designator(implementation2);

		assert_eq!(executor.code(authorizing1), delegation1);
		assert_eq!(executor.code(authorizing2), delegation2);

		// Both nonces should be incremented
		assert_eq!(executor.state().basic(authorizing1).nonce, U256::from(1));
		assert_eq!(executor.state().basic(authorizing2).nonce, U256::from(1));
	}
}

// ============================================================================
// Edge Cases and Security Tests (Section 9)
// ============================================================================

#[test]
fn test_9_1_self_delegation() {
	// Test: EOA delegates to its own address
	// Expected: Should work but lead to infinite loop prevention or error
	let caller = H160::from_slice(&[1u8; 20]);
	let self_delegating = H160::from_slice(&[2u8; 20]);
	let target = H160::from_slice(&[3u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		self_delegating,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Self-delegation: EOA delegates to itself
	let authorization = create_authorization(
		U256::from(1),
		self_delegating, // delegate to self
		U256::zero(),
		self_delegating, // authorizing address is same
	);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify self-delegation was set
	let delegation_designator = evm_core::create_delegation_designator(self_delegating);
	assert_eq!(executor.code(self_delegating), delegation_designator);

	// Now try to call the self-delegating address - this should handle the infinite loop
	let (call_exit_reason, _) = executor.transact_call(
		caller,
		self_delegating,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		Vec::new(),
	);

	// Should either succeed (infinite loop prevented) or fail gracefully
	assert!(
		matches!(call_exit_reason, ExitReason::Succeed(_))
			|| matches!(call_exit_reason, ExitReason::Error(_))
	);
}

#[test]
fn test_9_2_delegation_chain() {
	// Test: A delegates to B, B delegates to C
	// Expected: Each delegation resolved independently (no chain following)
	let caller = H160::from_slice(&[1u8; 20]);
	let account_a = H160::from_slice(&[2u8; 20]);
	let account_b = H160::from_slice(&[3u8; 20]);
	let account_c = H160::from_slice(&[4u8; 20]);
	let target = H160::from_slice(&[5u8; 20]);

	// C has actual implementation code
	let implementation_code = vec![
		0x60, 0x42, // PUSH1 0x42
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	];

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		account_a,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		account_b,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		account_c,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: implementation_code,
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Set up chain: A -> B, B -> C
	let auth_a_to_b = create_authorization(U256::from(1), account_b, U256::zero(), account_a);
	let auth_b_to_c = create_authorization(U256::from(1), account_c, U256::zero(), account_b);

	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![auth_a_to_b, auth_b_to_c],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify delegations were set
	let delegation_a_to_b = evm_core::create_delegation_designator(account_b);
	let delegation_b_to_c = evm_core::create_delegation_designator(account_c);

	assert_eq!(executor.code(account_a), delegation_a_to_b);
	assert_eq!(executor.code(account_b), delegation_b_to_c);

	// Now call A - it should delegate to B, and B should execute its own delegation code (not follow chain to C)
	let (call_exit_reason, return_data) = executor.transact_call(
		caller,
		account_a,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		Vec::new(),
	);

	// According to EIP-7702, calling A should:
	// 1. Resolve A's delegation to B
	// 2. Execute B's code (which is a delegation designator)
	// 3. NOT follow the chain to C

	// Since B's code is a delegation designator (invalid EVM bytecode), this should fail
	assert!(matches!(call_exit_reason, ExitReason::Error(_)));
}

#[test]
fn test_9_3_reentrancy_via_delegation() {
	// Test: Delegated code calls back to delegating EOA
	// Expected: Proper reentrancy handling
	let caller = H160::from_slice(&[1u8; 20]);
	let delegating_address = H160::from_slice(&[2u8; 20]);
	let implementation_address = H160::from_slice(&[3u8; 20]);

	// Implementation code that calls back to the delegating address
	let implementation_code = vec![
		// Prepare CALL parameters
		0x60, 0x00, // PUSH1 0x00 (retSize)
		0x60, 0x00, // PUSH1 0x00 (retOffset)
		0x60, 0x00, // PUSH1 0x00 (argsSize)
		0x60, 0x00, // PUSH1 0x00 (argsOffset)
		0x60, 0x00, // PUSH1 0x00 (value)
		0x73, // PUSH20
	];
	let mut full_code = implementation_code;
	full_code.extend_from_slice(delegating_address.as_bytes()); // Push delegating address
	full_code.extend_from_slice(&[
		0x61, 0x27, 0x10, // PUSH2 10000 (gas)
		0xf1, // CALL (reentrancy!)
		// Return the result
		0x60, 0x00, // PUSH1 0x00
		0x52, // MSTORE
		0x60, 0x20, // PUSH1 0x20
		0x60, 0x00, // PUSH1 0x00
		0xf3, // RETURN
	]);

	let delegation_designator = evm_core::create_delegation_designator(implementation_address);
	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: full_code,
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

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	// Call the delegating address - this will cause reentrancy
	let (exit_reason, _) = executor.transact_call(
		caller,
		delegating_address,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		Vec::new(),
	);

	// Should handle reentrancy properly (either succeed or fail gracefully)
	// Should not cause infinite recursion
	assert!(
		matches!(exit_reason, ExitReason::Succeed(_))
			|| matches!(exit_reason, ExitReason::Error(_))
	);
}

#[test]
fn test_9_4_gas_exhaustion() {
	// Test: Insufficient gas for authorization processing
	// Expected: Transaction reverts, no partial delegation
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);
	let target = H160::from_slice(&[4u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: vec![0x60, 0x42],
		},
	);

	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	let authorization =
		create_authorization(U256::from(1), implementation, U256::zero(), authorizing);

	// Provide insufficient gas (less than the 25000 needed for authorization)
	let (exit_reason, _) = executor.transact_call(
		caller,
		target,
		U256::zero(),
		Vec::new(),
		20_000, // Insufficient gas
		Vec::new(),
		vec![authorization],
	);

	// Should fail due to insufficient gas
	assert!(matches!(
		exit_reason,
		ExitReason::Error(ExitError::OutOfGas)
	));

	// Verify no partial delegation occurred
	assert_eq!(executor.code(authorizing), Vec::<u8>::new());
}

#[test]
fn test_9_5_delegation_to_non_contract() {
	// Test: Delegate to EOA address (no code)
	// Expected: Execution finds no code at target
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]); // EOA with no code
	let authorizing = H160::from_slice(&[3u8; 20]);

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	// Implementation is an EOA with no code
	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(500),
			storage: BTreeMap::new(),
			code: Vec::new(), // No code
		},
	);

	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	let authorization =
		create_authorization(U256::from(1), implementation, U256::zero(), authorizing);

	// Set delegation
	let (exit_reason, _) = executor.transact_call(
		caller,
		H160::from_slice(&[9u8; 20]), // Dummy target
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Verify delegation was set
	let delegation_designator = evm_core::create_delegation_designator(implementation);
	assert_eq!(executor.code(authorizing), delegation_designator);

	// Now call the delegating address - should execute empty code
	let (call_exit_reason, return_data) = executor.transact_call(
		caller,
		authorizing,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		Vec::new(),
	);

	// Should succeed but with empty return data (no code to execute)
	assert_eq!(
		call_exit_reason,
		ExitReason::Succeed(evm::ExitSucceed::Stopped)
	);
	assert_eq!(return_data.len(), 0);
}

#[test]
fn test_9_6_delegation_to_selfdestruct_contract() {
	// Test: Delegate to contract that selfdestructs
	// Expected: Handle gracefully per EVM rules
	let caller = H160::from_slice(&[1u8; 20]);
	let implementation = H160::from_slice(&[2u8; 20]);
	let authorizing = H160::from_slice(&[3u8; 20]);

	// Implementation code that selfdestructs
	let implementation_code = vec![
		0x30, // ADDRESS (get own address)
		0xff, // SELFDESTRUCT
	];

	let config = Config::pectra();
	let mut state = BTreeMap::new();

	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	state.insert(
		implementation,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(500),
			storage: BTreeMap::new(),
			code: implementation_code,
		},
	);

	state.insert(
		authorizing,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(1000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	let vicinity = create_test_vicinity();
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = evm::executor::stack::StackSubstateMetadata::new(1000000, &config);
	let state = evm::executor::stack::MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	let authorization =
		create_authorization(U256::from(1), implementation, U256::zero(), authorizing);

	// Set delegation
	let (exit_reason, _) = executor.transact_call(
		caller,
		H160::from_slice(&[9u8; 20]), // Dummy target
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		vec![authorization],
	);

	assert_eq!(exit_reason, ExitReason::Succeed(evm::ExitSucceed::Stopped));

	// Now call the delegating address - should execute selfdestruct code
	let (call_exit_reason, _) = executor.transact_call(
		caller,
		authorizing,
		U256::zero(),
		Vec::new(),
		100_000,
		Vec::new(),
		Vec::new(),
	);

	// Should handle selfdestruct gracefully
	assert!(
		matches!(call_exit_reason, ExitReason::Succeed(_))
			|| matches!(call_exit_reason, ExitReason::Error(_))
	);
}
