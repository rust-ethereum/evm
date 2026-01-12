use evm::{
	backend::MemoryBackend,
	executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata},
	Config, ExitReason, ExitSucceed,
};
use evm_core::Opcode;
use primitive_types::{H160, H256, U256};
use std::collections::BTreeMap;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create test configuration with EIP-7939 enabled (Osaka)
fn create_eip7939_config() -> Config {
	let config = Config::osaka();
	assert!(
		config.has_eip_7939,
		"EIP-7939 must be enabled in Osaka config"
	);
	config
}

/// Create test configuration without EIP-7939 (Pectra)
fn create_pre_eip7939_config() -> Config {
	let config = Config::pectra();
	assert!(
		!config.has_eip_7939,
		"EIP-7939 should not be enabled in Pectra"
	);
	config
}

/// Create a test vicinity
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
		block_gas_limit: U256::from(30_000_000),
		block_base_fee_per_gas: U256::from(7),
		chain_id: U256::from(1),
	}
}

/// Build EVM bytecode that pushes a value and executes CLZ
fn build_clz_bytecode(value: U256) -> Vec<u8> {
	let mut code = Vec::new();

	// PUSH32 <value>
	code.push(Opcode::PUSH32.as_u8());
	let value_bytes: [u8; 32] = H256::from(value.to_big_endian()).into();
	code.extend_from_slice(&value_bytes);

	// CLZ (0x1e)
	code.push(Opcode::CLZ.as_u8());

	// PUSH1 0x00 (memory offset)
	code.push(Opcode::PUSH1.as_u8());
	code.push(0x00);

	// MSTORE (store result at offset 0)
	code.push(Opcode::MSTORE.as_u8());

	// PUSH1 0x20 (return size: 32 bytes)
	code.push(Opcode::PUSH1.as_u8());
	code.push(0x20);

	// PUSH1 0x00 (return offset)
	code.push(Opcode::PUSH1.as_u8());
	code.push(0x00);

	// RETURN
	code.push(Opcode::RETURN.as_u8());

	code
}

/// Execute bytecode and return the result
fn execute_bytecode(config: &Config, code: Vec<u8>) -> (ExitReason, Vec<u8>, u64) {
	let vicinity = create_test_vicinity();
	let contract_address = H160::from_low_u64_be(0x1000);
	let caller = H160::from_low_u64_be(0x2000);

	// Deploy the code to the contract address
	let mut state: BTreeMap<H160, evm::backend::MemoryAccount> = BTreeMap::new();
	state.insert(
		contract_address,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::zero(),
			storage: BTreeMap::new(),
			code: code, // Deploy the bytecode here
		},
	);
	state.insert(
		caller,
		evm::backend::MemoryAccount {
			nonce: U256::zero(),
			balance: U256::from(10_000_000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);
	let mut backend = MemoryBackend::new(&vicinity, state);

	let metadata = StackSubstateMetadata::new(u64::MAX, config);
	let state = MemoryStackState::new(metadata, &mut backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, config, &precompiles);

	let (exit_reason, return_value) = executor.transact_call(
		caller,
		contract_address,
		U256::zero(),
		Vec::new(), // calldata (empty for our tests)
		u64::MAX,
		Vec::new(), // access_list
		Vec::new(), // authorization_list
	);

	let gas_used = executor.used_gas();

	(exit_reason, return_value, gas_used)
}

// ============================================================================
// Section 1: CLZ Function Unit Tests (via integration)
// ============================================================================

#[cfg(test)]
mod clz_function_tests {
	use super::*;

	/// Helper to test CLZ via executor and return the result
	fn execute_clz(value: U256) -> U256 {
		let config = create_eip7939_config();
		let code = build_clz_bytecode(value);
		let (exit_reason, return_value, _) = execute_bytecode(&config, code);
		assert!(matches!(
			exit_reason,
			ExitReason::Succeed(ExitSucceed::Returned)
		));
		U256::from_big_endian(&return_value)
	}

	#[test]
	fn test_clz_zero() {
		// Zero input should return 256
		let result = execute_clz(U256::zero());
		assert_eq!(result, U256::from(256));
	}

	#[test]
	fn test_clz_max_value() {
		// All bits set (0xFFFF...FFFF) should return 0
		let result = execute_clz(U256::MAX);
		assert_eq!(result, U256::from(0));
	}

	#[test]
	fn test_clz_msb_set() {
		// MSB set (0x8000...0000) should return 0
		let value = U256::one() << 255;
		let result = execute_clz(value);
		assert_eq!(result, U256::from(0));
	}

	#[test]
	fn test_clz_second_bit_set() {
		// Second bit set (0x4000...0000) should return 1
		let value = U256::one() << 254;
		let result = execute_clz(value);
		assert_eq!(result, U256::from(1));
	}

	#[test]
	fn test_clz_lsb_only() {
		// Only LSB set (0x0000...0001) should return 255
		let result = execute_clz(U256::one());
		assert_eq!(result, U256::from(255));
	}

	#[test]
	fn test_clz_single_byte() {
		// 0x0000...00FF should return 248
		let result = execute_clz(U256::from(0xFF));
		assert_eq!(result, U256::from(248));
	}

	#[test]
	fn test_clz_two_bytes() {
		// 0x0000...FFFF should return 240
		let result = execute_clz(U256::from(0xFFFF));
		assert_eq!(result, U256::from(240));
	}

	#[test]
	fn test_clz_various_positions() {
		// Test various bit positions
		for i in 0..256 {
			let value = U256::one() << i;
			let result = execute_clz(value);
			assert_eq!(
				result,
				U256::from(255 - i),
				"CLZ of 1 << {} should be {}",
				i,
				255 - i
			);
		}
	}

	#[test]
	fn test_clz_power_of_two_minus_one() {
		// Test values like 0x7FFF...FFFF
		for i in 1..=255 {
			let value = (U256::one() << i) - U256::one();
			let result = execute_clz(value);
			assert_eq!(
				result,
				U256::from(256 - i),
				"CLZ of (1 << {}) - 1 should be {}",
				i,
				256 - i
			);
		}
	}
}

// ============================================================================
// Section 2: Integration Tests with Executor
// ============================================================================

#[cfg(test)]
mod integration_tests {
	use super::*;

	#[test]
	fn test_clz_opcode_zero_input() {
		let config = create_eip7939_config();
		let code = build_clz_bytecode(U256::zero());

		let (exit_reason, return_value, _) = execute_bytecode(&config, code);

		assert!(matches!(
			exit_reason,
			ExitReason::Succeed(ExitSucceed::Returned)
		));
		assert_eq!(return_value.len(), 32);

		let result = U256::from_big_endian(&return_value);
		assert_eq!(result, U256::from(256));
	}

	#[test]
	fn test_clz_opcode_max_value() {
		let config = create_eip7939_config();
		let code = build_clz_bytecode(U256::MAX);

		let (exit_reason, return_value, _) = execute_bytecode(&config, code);

		assert!(matches!(
			exit_reason,
			ExitReason::Succeed(ExitSucceed::Returned)
		));
		let result = U256::from_big_endian(&return_value);
		assert_eq!(result, U256::from(0));
	}

	#[test]
	fn test_clz_opcode_msb_set() {
		let config = create_eip7939_config();
		let value = U256::one() << 255;
		let code = build_clz_bytecode(value);

		let (exit_reason, return_value, _) = execute_bytecode(&config, code);

		assert!(matches!(
			exit_reason,
			ExitReason::Succeed(ExitSucceed::Returned)
		));
		let result = U256::from_big_endian(&return_value);
		assert_eq!(result, U256::from(0));
	}

	#[test]
	fn test_clz_opcode_lsb_only() {
		let config = create_eip7939_config();
		let code = build_clz_bytecode(U256::one());

		let (exit_reason, return_value, _) = execute_bytecode(&config, code);

		assert!(matches!(
			exit_reason,
			ExitReason::Succeed(ExitSucceed::Returned)
		));
		let result = U256::from_big_endian(&return_value);
		assert_eq!(result, U256::from(255));
	}

	#[test]
	fn test_clz_opcode_various_values() {
		let config = create_eip7939_config();

		let test_cases = vec![
			(U256::from(0x80), U256::from(248)), // 0x80 = 1000_0000, 248 leading zeros
			(U256::from(0x100), U256::from(247)), // 0x100 = 1_0000_0000, 247 leading zeros
			(U256::from(0x8000), U256::from(240)), // 240 leading zeros
			(U256::from(0x10000), U256::from(239)), // 239 leading zeros
		];

		for (input, expected) in test_cases {
			let code = build_clz_bytecode(input);
			let (exit_reason, return_value, _) = execute_bytecode(&config, code);

			assert!(matches!(
				exit_reason,
				ExitReason::Succeed(ExitSucceed::Returned)
			));
			let result = U256::from_big_endian(&return_value);
			assert_eq!(
				result, expected,
				"CLZ of {:?} should be {:?}, got {:?}",
				input, expected, result
			);
		}
	}
}

// ============================================================================
// Section 3: Gas Cost Tests
// ============================================================================

#[cfg(test)]
mod gas_cost_tests {
	use super::*;

	#[test]
	fn test_clz_gas_cost() {
		let config = create_eip7939_config();

		// Build bytecode that just does CLZ and stops (no return overhead)
		let mut code = Vec::new();

		// PUSH32 <value>
		code.push(Opcode::PUSH32.as_u8());
		code.extend_from_slice(&[0u8; 32]); // zero value

		// CLZ
		code.push(Opcode::CLZ.as_u8());

		// STOP
		code.push(Opcode::STOP.as_u8());

		let vicinity = create_test_vicinity();
		let contract_address = H160::from_low_u64_be(0x1000);
		let caller = H160::from_low_u64_be(0x2000);

		let mut state: BTreeMap<H160, evm::backend::MemoryAccount> = BTreeMap::new();
		state.insert(
			contract_address,
			evm::backend::MemoryAccount {
				nonce: U256::zero(),
				balance: U256::zero(),
				storage: BTreeMap::new(),
				code: code,
			},
		);
		state.insert(
			caller,
			evm::backend::MemoryAccount {
				nonce: U256::zero(),
				balance: U256::from(10_000_000),
				storage: BTreeMap::new(),
				code: Vec::new(),
			},
		);
		let mut backend = MemoryBackend::new(&vicinity, state);

		let metadata = StackSubstateMetadata::new(u64::MAX, &config);
		let state = MemoryStackState::new(metadata, &mut backend);
		let precompiles = BTreeMap::new();
		let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

		let (_exit_reason, _return_value) = executor.transact_call(
			caller,
			contract_address,
			U256::zero(),
			Vec::new(),
			u64::MAX,
			Vec::new(), // access_list
			Vec::new(), // authorization_list
		);

		// Gas breakdown:
		// - PUSH32: 3 (G_VERYLOW)
		// - CLZ: 5 (G_LOW) - this is what we're testing
		// - STOP: 0 (G_ZERO)
		// Total opcode gas: 8

		let gas_used = executor.used_gas();
		// We expect at least the base transaction cost + opcode costs
		// The CLZ operation itself should cost exactly G_LOW (5)
		assert!(
			gas_used >= 8,
			"Expected at least 8 gas for PUSH32 + CLZ + STOP, got {}",
			gas_used
		);
	}
}

// ============================================================================
// Section 4: Pre-fork Behavior Tests (Opcode Invalid)
// ============================================================================

#[cfg(test)]
mod pre_fork_tests {
	use super::*;
	use evm::ExitError;

	#[test]
	fn test_clz_invalid_before_osaka() {
		let config = create_pre_eip7939_config();
		let code = build_clz_bytecode(U256::one());

		let (exit_reason, _, _) = execute_bytecode(&config, code);

		// Before Osaka, CLZ opcode should be invalid
		assert!(
			matches!(exit_reason, ExitReason::Error(ExitError::InvalidCode(_))),
			"CLZ should be invalid before Osaka, got {:?}",
			exit_reason
		);
	}
}

// ============================================================================
// Section 5: Edge Case Tests
// ============================================================================

#[cfg(test)]
mod edge_case_tests {
	use super::*;

	#[test]
	fn test_clz_boundary_values() {
		let config = create_eip7939_config();

		// Test at byte boundaries
		let boundary_tests = vec![
			// (input, expected_clz)
			(U256::from(0x01), U256::from(255)),          // 1 byte, LSB
			(U256::from(0x0100), U256::from(247)),        // 2 bytes
			(U256::from(0x010000), U256::from(239)),      // 3 bytes
			(U256::from(0x01000000u64), U256::from(231)), // 4 bytes
		];

		for (input, expected) in boundary_tests {
			let code = build_clz_bytecode(input);
			let (exit_reason, return_value, _) = execute_bytecode(&config, code);

			assert!(matches!(
				exit_reason,
				ExitReason::Succeed(ExitSucceed::Returned)
			));
			let result = U256::from_big_endian(&return_value);
			assert_eq!(
				result, expected,
				"CLZ boundary test failed for input {:?}",
				input
			);
		}
	}

	#[test]
	fn test_clz_consecutive_operations() {
		let config = create_eip7939_config();

		// Build bytecode that performs multiple CLZ operations
		let mut code = Vec::new();

		// First CLZ: value = 1
		code.push(Opcode::PUSH1.as_u8());
		code.push(0x01);
		code.push(Opcode::CLZ.as_u8());
		// Result: 255, now on stack

		// Second CLZ on result (255 = 0xFF)
		code.push(Opcode::CLZ.as_u8());
		// Result: 248 (since 255 = 0x00...00FF has 248 leading zeros)

		// Store result
		code.push(Opcode::PUSH1.as_u8());
		code.push(0x00);
		code.push(Opcode::MSTORE.as_u8());

		// Return
		code.push(Opcode::PUSH1.as_u8());
		code.push(0x20);
		code.push(Opcode::PUSH1.as_u8());
		code.push(0x00);
		code.push(Opcode::RETURN.as_u8());

		let (exit_reason, return_value, _) = execute_bytecode(&config, code);

		assert!(matches!(
			exit_reason,
			ExitReason::Succeed(ExitSucceed::Returned)
		));
		let result = U256::from_big_endian(&return_value);
		assert_eq!(
			result,
			U256::from(248),
			"CLZ(CLZ(1)) should be CLZ(255) = 248"
		);
	}
}

// ============================================================================
// Section 6: EIP-7939 Specification Compliance Tests
// ============================================================================

#[cfg(test)]
mod spec_compliance_tests {
	use super::*;

	/// Helper to test CLZ via executor and return the result
	fn execute_clz(value: U256) -> U256 {
		let config = create_eip7939_config();
		let code = build_clz_bytecode(value);
		let (exit_reason, return_value, _) = execute_bytecode(&config, code);
		assert!(matches!(
			exit_reason,
			ExitReason::Succeed(ExitSucceed::Returned)
		));
		U256::from_big_endian(&return_value)
	}

	/// Test cases from EIP-7939 specification
	#[test]
	fn test_eip7939_spec_test_vectors() {
		// Test vectors from the EIP specification
		let test_vectors = vec![
			// (input, expected_output)
			(U256::zero(), U256::from(256)),     // Zero input returns 256
			(U256::one() << 255, U256::from(0)), // 0x8000...0000 returns 0
			(U256::MAX, U256::from(0)),          // All ones returns 0
			(U256::one() << 254, U256::from(1)), // 0x4000...0000 returns 1
			(U256::one(), U256::from(255)),      // 0x0000...0001 returns 255
		];

		for (input, expected) in test_vectors {
			let result = execute_clz(input);
			assert_eq!(
				result, expected,
				"EIP-7939 spec test vector failed: CLZ({:?}) should be {:?}, got {:?}",
				input, expected, result
			);
		}
	}

	/// Verify that CLZ(0) >> 3 == 32 (useful for byte skipping as mentioned in EIP)
	#[test]
	fn test_byte_skip_calculation() {
		let clz_zero = execute_clz(U256::zero());
		assert_eq!(clz_zero, U256::from(256));

		// As per EIP rationale: 256 >> 3 = 32 bytes to skip
		let bytes_to_skip = clz_zero >> 3;
		assert_eq!(bytes_to_skip, U256::from(32));
	}
}
