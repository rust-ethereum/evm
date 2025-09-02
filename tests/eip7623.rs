use evm::{
	backend::MemoryBackend,
	executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata},
	gasometer::{call_transaction_cost, create_transaction_cost, Gasometer, TransactionCost},
	Config, ExitError, ExitReason,
};
use primitive_types::{H160, U256};
use std::collections::BTreeMap;

// ============================================================================
// Constants from EIP-7623
// ============================================================================

const TOTAL_COST_FLOOR_PER_TOKEN: u64 = 10;
const INITCODE_WORD_COST: u64 = 2;

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculate tokens in calldata as per EIP-7623 specification
fn calculate_tokens_in_calldata(zero_bytes: usize, non_zero_bytes: usize) -> u64 {
	zero_bytes as u64 + (non_zero_bytes as u64 * 4)
}

/// Create test configuration with EIP-7623 enabled
fn create_eip7623_config() -> Config {
	let config = Config::pectra();
	assert!(
		config.has_eip_7623,
		"EIP-7623 must be enabled in Pectra config"
	);
	assert_eq!(
		config.gas_calldata_zero_floor, 10,
		"Zero byte floor cost should be 10"
	);
	assert_eq!(
		config.gas_calldata_non_zero_floor, 40,
		"Non-zero byte floor cost should be 40"
	);
	config
}

/// Create test configuration without EIP-7623
fn create_pre_eip7623_config() -> Config {
	let config = Config::cancun();
	assert!(
		!config.has_eip_7623,
		"EIP-7623 should not be enabled in Cancun"
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

/// Create test calldata with specified zero and non-zero byte counts
fn create_test_calldata(zero_bytes: usize, non_zero_bytes: usize) -> Vec<u8> {
	let mut data = Vec::new();
	data.extend(vec![0u8; zero_bytes]);
	data.extend(vec![0xffu8; non_zero_bytes]);
	data
}

// ============================================================================
// Section 1: Basic Gas Cost Calculation Tests
// ============================================================================

#[cfg(test)]
mod basic_gas_cost_tests {
	use super::*;

	#[test]
	fn test_tokens_calculation() {
		// Test the tokens_in_calldata calculation
		assert_eq!(calculate_tokens_in_calldata(0, 0), 0);
		assert_eq!(calculate_tokens_in_calldata(10, 0), 10);
		assert_eq!(calculate_tokens_in_calldata(0, 10), 40);
		assert_eq!(calculate_tokens_in_calldata(10, 10), 50);
		assert_eq!(calculate_tokens_in_calldata(100, 100), 500);
	}

	#[test]
	fn test_floor_cost_calculation() {
		let _config = create_eip7623_config();

		// Test floor cost calculations
		// Floor cost = TOTAL_COST_FLOOR_PER_TOKEN * tokens_in_calldata

		// Empty calldata: 0 tokens * 10 = 0
		let tokens = calculate_tokens_in_calldata(0, 0);
		assert_eq!(tokens * TOTAL_COST_FLOOR_PER_TOKEN, 0);

		// 10 zero bytes: 10 tokens * 10 = 100
		let tokens = calculate_tokens_in_calldata(10, 0);
		assert_eq!(tokens * TOTAL_COST_FLOOR_PER_TOKEN, 100);

		// 10 non-zero bytes: 40 tokens * 10 = 400
		let tokens = calculate_tokens_in_calldata(0, 10);
		assert_eq!(tokens * TOTAL_COST_FLOOR_PER_TOKEN, 400);

		// Mixed: 10 zero + 10 non-zero = 50 tokens * 10 = 500
		let tokens = calculate_tokens_in_calldata(10, 10);
		assert_eq!(tokens * TOTAL_COST_FLOOR_PER_TOKEN, 500);
	}

	#[test]
	fn test_standard_cost_calculation() {
		let config = create_eip7623_config();

		// Standard cost = gas_transaction_zero_data * zero_bytes + gas_transaction_non_zero_data * non_zero_bytes
		// For EIP-7623 config: zero = 4, non_zero = 16

		// Empty calldata: 0
		assert_eq!(
			0 * config.gas_transaction_zero_data + 0 * config.gas_transaction_non_zero_data,
			0
		);

		// 10 zero bytes: 10 * 4 = 40
		assert_eq!(
			10 * config.gas_transaction_zero_data + 0 * config.gas_transaction_non_zero_data,
			40
		);

		// 10 non-zero bytes: 10 * 16 = 160
		assert_eq!(
			0 * config.gas_transaction_zero_data + 10 * config.gas_transaction_non_zero_data,
			160
		);

		// Mixed: 10 * 4 + 10 * 16 = 200
		assert_eq!(
			10 * config.gas_transaction_zero_data + 10 * config.gas_transaction_non_zero_data,
			200
		);
	}

	#[test]
	fn test_max_formula() {
		// Test the max() formula from EIP-7623
		let _config = create_eip7623_config();

		// Case 1: Standard cost is higher
		let standard_cost = 1000u64;
		let floor_cost = 500u64;
		assert_eq!(std::cmp::max(standard_cost, floor_cost), 1000);

		// Case 2: Floor cost is higher
		let standard_cost = 500u64;
		let floor_cost = 1000u64;
		assert_eq!(std::cmp::max(standard_cost, floor_cost), 1000);

		// Case 3: Equal costs
		let standard_cost = 1000u64;
		let floor_cost = 1000u64;
		assert_eq!(std::cmp::max(standard_cost, floor_cost), 1000);
	}
}

// ============================================================================
// Section 2: Transaction Cost Tests
// ============================================================================

#[cfg(test)]
mod transaction_cost_tests {
	use super::*;

	#[test]
	fn test_call_transaction_with_empty_calldata() {
		let _config = create_eip7623_config();
		let data = vec![];
		let access_list = vec![];
		let authorization_list = vec![];

		let cost = call_transaction_cost(&data, &access_list, &authorization_list);

		if let TransactionCost::Call {
			zero_data_len,
			non_zero_data_len,
			..
		} = cost
		{
			assert_eq!(zero_data_len, 0);
			assert_eq!(non_zero_data_len, 0);
		} else {
			panic!("Expected Call transaction cost");
		}
	}

	#[test]
	fn test_call_transaction_with_zero_bytes() {
		let _config = create_eip7623_config();
		let data = vec![0u8; 100];
		let access_list = vec![];
		let authorization_list = vec![];

		let cost = call_transaction_cost(&data, &access_list, &authorization_list);

		if let TransactionCost::Call {
			zero_data_len,
			non_zero_data_len,
			..
		} = cost
		{
			assert_eq!(zero_data_len, 100);
			assert_eq!(non_zero_data_len, 0);
		} else {
			panic!("Expected Call transaction cost");
		}
	}

	#[test]
	fn test_call_transaction_with_non_zero_bytes() {
		let _config = create_eip7623_config();
		let data = vec![0xffu8; 100];
		let access_list = vec![];
		let authorization_list = vec![];

		let cost = call_transaction_cost(&data, &access_list, &authorization_list);

		if let TransactionCost::Call {
			zero_data_len,
			non_zero_data_len,
			..
		} = cost
		{
			assert_eq!(zero_data_len, 0);
			assert_eq!(non_zero_data_len, 100);
		} else {
			panic!("Expected Call transaction cost");
		}
	}

	#[test]
	fn test_call_transaction_with_mixed_bytes() {
		let _config = create_eip7623_config();
		let mut data = vec![0u8; 50];
		data.extend(vec![0xffu8; 50]);
		let access_list = vec![];
		let authorization_list = vec![];

		let cost = call_transaction_cost(&data, &access_list, &authorization_list);

		if let TransactionCost::Call {
			zero_data_len,
			non_zero_data_len,
			..
		} = cost
		{
			assert_eq!(zero_data_len, 50);
			assert_eq!(non_zero_data_len, 50);
		} else {
			panic!("Expected Call transaction cost");
		}
	}

	#[test]
	fn test_create_transaction_cost() {
		let _config = create_eip7623_config();
		let data = create_test_calldata(10, 10);
		let access_list = vec![];
		let authorization_list = vec![];

		let cost = create_transaction_cost(&data, &access_list, &authorization_list);

		if let TransactionCost::Create {
			zero_data_len,
			non_zero_data_len,
			initcode_cost,
			..
		} = cost
		{
			assert_eq!(zero_data_len, 10);
			assert_eq!(non_zero_data_len, 10);
			// Initcode cost = INITCODE_WORD_COST * words(initcode)
			// words = (len + 31) / 32
			let words = (data.len() + 31) / 32;
			assert_eq!(initcode_cost, INITCODE_WORD_COST * words as u64);
		} else {
			panic!("Expected Create transaction cost");
		}
	}
}

// ============================================================================
// Section 3: Gasometer Integration Tests
// ============================================================================

#[cfg(test)]
mod gasometer_tests {
	use super::*;

	#[test]
	fn test_gasometer_with_eip7623_enabled() {
		let config = create_eip7623_config();
		let gas_limit = 100_000;
		let mut gasometer = Gasometer::new(gas_limit, &config);

		// Record a simple call transaction
		let data = create_test_calldata(10, 10);
		let cost = call_transaction_cost(&data, &vec![], &vec![]);

		let result = gasometer.record_transaction(cost);
		assert!(result.is_ok(), "Should successfully record transaction");

		// Verify gas consumption follows EIP-7623 rules
		let used_gas = gasometer.total_used_gas();
		assert!(used_gas > 0, "Should consume gas");
	}

	#[test]
	fn test_gasometer_with_insufficient_gas_limit() {
		let config = create_eip7623_config();
		// Set gas limit below the floor requirement
		let gas_limit = 21_000; // Just base cost, no room for calldata
		let mut gasometer = Gasometer::new(gas_limit, &config);

		// Create calldata that requires floor cost
		let data = create_test_calldata(0, 100); // 400 tokens * 10 = 4000 floor cost
		let cost = call_transaction_cost(&data, &vec![], &vec![]);

		let result = gasometer.record_transaction(cost);
		assert!(
			matches!(result, Err(ExitError::OutOfGas)),
			"Should fail with OutOfGas"
		);
	}

	#[test]
	fn test_gasometer_comparison_with_and_without_eip7623() {
		// Test with EIP-7623 disabled
		let config_without = create_pre_eip7623_config();
		let gas_limit = 100_000;
		let mut gasometer_without = Gasometer::new(gas_limit, &config_without);

		let data = create_test_calldata(0, 1000); // Large calldata
		let cost = call_transaction_cost(&data, &vec![], &vec![]);

		gasometer_without.record_transaction(cost.clone()).unwrap();
		let used_without = gasometer_without.total_used_gas();

		// Test with EIP-7623 enabled
		let config_with = create_eip7623_config();
		let mut gasometer_with = Gasometer::new(gas_limit, &config_with);

		gasometer_with.record_transaction(cost).unwrap();
		gasometer_with.post_execution().unwrap();
		let used_with = gasometer_with.total_used_gas();

		// With large calldata, EIP-7623 should charge more due to floor cost
		assert!(
			used_with > used_without,
			"EIP-7623 should not reduce gas cost"
		);
	}
}

// ============================================================================
// Section 4: Contract Creation Tests
// ============================================================================

#[cfg(test)]
mod contract_creation_tests {
	use super::*;

	#[test]
	fn test_contract_creation_with_initcode() {
		let config = create_eip7623_config();
		let gas_limit = 500_000;
		let mut gasometer = Gasometer::new(gas_limit, &config);

		// Create initcode (contract bytecode)
		let initcode = vec![0x60, 0x80, 0x60, 0x40, 0x52]; // Simple initcode
		let cost = create_transaction_cost(&initcode, &vec![], &vec![]);

		let result = gasometer.record_transaction(cost);
		assert!(
			result.is_ok(),
			"Should successfully record contract creation"
		);

		// Verify initcode cost is included
		let used_gas = gasometer.total_used_gas();
		assert!(
			used_gas >= config.gas_transaction_create,
			"Should include base creation cost"
		);
	}

	#[test]
	fn test_contract_creation_floor_cost() {
		let config = create_eip7623_config();
		let gas_limit = 1_000_000;
		let mut gasometer = Gasometer::new(gas_limit, &config);

		// Create large initcode that triggers floor cost
		let initcode = vec![0xffu8; 10000]; // Large non-zero initcode
		let cost = create_transaction_cost(&initcode, &vec![], &vec![]);

		if let TransactionCost::Create {
			zero_data_len,
			non_zero_data_len,
			initcode_cost: _,
			..
		} = cost
		{
			let tokens = calculate_tokens_in_calldata(zero_data_len, non_zero_data_len);
			let floor_cost = tokens * TOTAL_COST_FLOOR_PER_TOKEN;

			// Record transaction and apply post-execution adjustments
			gasometer.record_transaction(cost).unwrap();
			gasometer.post_execution().unwrap();

			let used_gas = gasometer.total_used_gas();

			// Gas should be at least the floor cost
			assert!(
				used_gas >= floor_cost + config.gas_transaction_call,
				"Should apply floor cost for large initcode"
			);
		}
	}
}

// ============================================================================
// Section 5: Edge Cases and Boundary Tests
// ============================================================================

#[cfg(test)]
mod edge_case_tests {
	use super::*;

	#[test]
	fn test_maximum_calldata_size() {
		let _config = create_eip7623_config();
		// Maximum theoretical calldata that could fit in a block pre-EIP-7623
		// was about 1.79 MB, post-EIP-7623 it's reduced to ~0.72 MB

		// Test with 1 MB of calldata
		let data = vec![0xffu8; 1_000_000];
		let cost = call_transaction_cost(&data, &vec![], &vec![]);

		if let TransactionCost::Call {
			zero_data_len,
			non_zero_data_len,
			..
		} = cost
		{
			let tokens = calculate_tokens_in_calldata(zero_data_len, non_zero_data_len);
			let floor_cost = tokens * TOTAL_COST_FLOOR_PER_TOKEN;

			// With 1MB of non-zero bytes: 1,000,000 * 4 = 4,000,000 tokens
			// Floor cost: 4,000,000 * 10 = 40,000,000 gas
			assert_eq!(
				floor_cost, 40_000_000,
				"Floor cost for 1MB should be 40M gas"
			);
		}
	}

	#[test]
	fn test_zero_length_calldata() {
		let config = create_eip7623_config();
		let gas_limit = 50_000;
		let mut gasometer = Gasometer::new(gas_limit, &config);

		let data = vec![];
		let cost = call_transaction_cost(&data, &vec![], &vec![]);

		let result = gasometer.record_transaction(cost);
		assert!(result.is_ok(), "Empty calldata should be valid");

		let used_gas = gasometer.total_used_gas();
		assert_eq!(
			used_gas, config.gas_transaction_call,
			"Should only charge base cost for empty calldata"
		);
	}

	#[test]
	fn test_single_byte_calldata() {
		let config = create_eip7623_config();
		let gas_limit = 50_000;

		// Test with single zero byte
		let mut gasometer = Gasometer::new(gas_limit, &config);
		let data = vec![0x00];
		let cost = call_transaction_cost(&data, &vec![], &vec![]);
		gasometer.record_transaction(cost).unwrap();
		gasometer.post_execution().unwrap();
		let zero_byte_gas = gasometer.total_used_gas();

		// Test with single non-zero byte
		let mut gasometer = Gasometer::new(gas_limit, &config);
		let data = vec![0xff];
		let cost = call_transaction_cost(&data, &vec![], &vec![]);
		gasometer.record_transaction(cost).unwrap();
		gasometer.post_execution().unwrap();
		let non_zero_byte_gas = gasometer.total_used_gas();

		// Non-zero byte should cost more
		assert!(
			non_zero_byte_gas > zero_byte_gas,
			"Non-zero byte should cost more than zero byte"
		);
	}

	#[test]
	fn test_alternating_byte_pattern() {
		let config = create_eip7623_config();
		let gas_limit = 100_000;
		let mut gasometer = Gasometer::new(gas_limit, &config);

		// Create alternating pattern: 0x00, 0xff, 0x00, 0xff, ...
		let mut data = Vec::new();
		for _ in 0..100 {
			data.push(0x00);
			data.push(0xff);
		}

		let cost = call_transaction_cost(&data, &vec![], &vec![]);

		if let TransactionCost::Call {
			zero_data_len,
			non_zero_data_len,
			..
		} = cost
		{
			assert_eq!(zero_data_len, 100, "Should have 100 zero bytes");
			assert_eq!(non_zero_data_len, 100, "Should have 100 non-zero bytes");
		}

		gasometer.record_transaction(cost).unwrap();
		gasometer.post_execution().unwrap();

		let used_gas = gasometer.total_used_gas();
		let tokens = calculate_tokens_in_calldata(100, 100);
		let floor_cost = tokens * TOTAL_COST_FLOOR_PER_TOKEN;

		// Should use floor cost if it's higher than standard cost
		assert!(
			used_gas >= config.gas_transaction_call + floor_cost,
			"Should apply floor cost for mixed byte pattern"
		);
	}
}

// ============================================================================
// Section 6: Snapshot Tests for Gas Calculations
// ============================================================================

#[cfg(test)]
mod snapshot_tests {
	use super::*;

	#[test]
	fn test_snapshot_empty_calldata() {
		let config = create_eip7623_config();
		let gas_limit = 100_000;
		let mut gasometer = Gasometer::new(gas_limit, &config);

		let data = vec![];
		let cost = call_transaction_cost(&data, &vec![], &vec![]);

		gasometer.record_transaction(cost).unwrap();
		gasometer.post_execution().unwrap();

		let used_gas = gasometer.total_used_gas();

		// Snapshot: Empty calldata should use exactly base cost (21000)
		assert_eq!(used_gas, 21_000, "Empty calldata gas mismatch");
	}

	#[test]
	fn test_snapshot_small_calldata() {
		let config = create_eip7623_config();
		let gas_limit = 100_000;
		let mut gasometer = Gasometer::new(gas_limit, &config);

		// 10 zero bytes, 10 non-zero bytes
		let data = create_test_calldata(10, 10);
		let cost = call_transaction_cost(&data, &vec![], &vec![]);

		gasometer.record_transaction(cost).unwrap();
		gasometer.post_execution().unwrap();

		let used_gas = gasometer.total_used_gas();

		// Calculation:
		// Base cost: 21000
		// Standard calldata cost: 10*4 + 10*16 = 200
		// Total standard: 21200
		// Floor cost: (10 + 10*4) * 10 = 500
		// Total floor: 21000 + 500 = 21500
		// Should use max(21200, 21500) = 21500
		assert_eq!(used_gas, 21_500, "Small calldata gas mismatch");
	}

	#[test]
	fn test_snapshot_medium_calldata() {
		let config = create_eip7623_config();
		let gas_limit = 200_000;
		let mut gasometer = Gasometer::new(gas_limit, &config);

		// 100 non-zero bytes
		let data = vec![0xffu8; 100];
		let cost = call_transaction_cost(&data, &vec![], &vec![]);

		gasometer.record_transaction(cost).unwrap();
		gasometer.post_execution().unwrap();

		let used_gas = gasometer.total_used_gas();

		// Calculation:
		// Base cost: 21000
		// Standard calldata cost: 100*16 = 1600
		// Total standard: 22600
		// Floor cost: 100*4*10 = 4000
		// Total floor: 21000 + 4000 = 25000
		// Should use max(22600, 25000) = 25000
		assert_eq!(used_gas, 25_000, "Medium calldata gas mismatch");
	}

	#[test]
	fn test_snapshot_large_calldata() {
		let config = create_eip7623_config();
		let gas_limit = 500_000;
		let mut gasometer = Gasometer::new(gas_limit, &config);

		// 1000 non-zero bytes
		let data = vec![0xffu8; 1000];
		let cost = call_transaction_cost(&data, &vec![], &vec![]);

		gasometer.record_transaction(cost).unwrap();
		gasometer.post_execution().unwrap();

		let used_gas = gasometer.total_used_gas();

		// Calculation:
		// Base cost: 21000
		// Standard calldata cost: 1000*16 = 16000
		// Total standard: 37000
		// Floor cost: 1000*4*10 = 40000
		// Total floor: 21000 + 40000 = 61000
		// Should use max(37000, 61000) = 61000
		assert_eq!(used_gas, 61_000, "Large calldata gas mismatch");
	}

	#[test]
	fn test_snapshot_mixed_calldata() {
		let config = create_eip7623_config();
		let gas_limit = 100_000;
		let mut gasometer = Gasometer::new(gas_limit, &config);

		// 50 zero bytes, 50 non-zero bytes
		let mut data = vec![0u8; 50];
		data.extend(vec![0xffu8; 50]);
		let cost = call_transaction_cost(&data, &vec![], &vec![]);

		gasometer.record_transaction(cost).unwrap();
		gasometer.post_execution().unwrap();

		let used_gas = gasometer.total_used_gas();

		// Calculation:
		// Base cost: 21000
		// Standard calldata cost: 50*4 + 50*16 = 1000
		// Total standard: 22000
		// Tokens: 50 + 50*4 = 250
		// Floor cost: 250*10 = 2500
		// Total floor: 21000 + 2500 = 23500
		// Should use max(22000, 23500) = 23500
		assert_eq!(used_gas, 23_500, "Mixed calldata gas mismatch");
	}

	#[test]
	fn test_snapshot_contract_creation() {
		let config = create_eip7623_config();
		let gas_limit = 200_000;
		let mut gasometer = Gasometer::new(gas_limit, &config);

		// Simple initcode: 20 bytes (mix of zero and non-zero)
		let initcode = vec![
			0x60, 0x00, // PUSH1 0x00
			0x60, 0x00, // PUSH1 0x00
			0x60, 0x00, // PUSH1 0x00
			0x60, 0x00, // PUSH1 0x00
			0x60, 0x00, // PUSH1 0x00
			0x60, 0x00, // PUSH1 0x00
			0x60, 0x00, // PUSH1 0x00
			0x60, 0x00, // PUSH1 0x00
			0x60, 0x00, // PUSH1 0x00
			0xf3, 0x00, // RETURN + padding
		];

		let cost = create_transaction_cost(&initcode, &vec![], &vec![]);

		gasometer.record_transaction(cost).unwrap();
		gasometer.post_execution().unwrap();

		let used_gas = gasometer.total_used_gas();

		// Precise calculation based on the actual initcode content
		let zero_count = initcode.iter().filter(|&&b| b == 0).count();
		let non_zero_count = initcode.len() - zero_count;

		// Verify our test data: should be 10 zero bytes (0x00) and 10 non-zero bytes (0x60, 0xf3)
		assert_eq!(zero_count, 10, "Expected 10 zero bytes in test initcode");
		assert_eq!(
			non_zero_count, 10,
			"Expected 10 non-zero bytes in test initcode"
		);

		// Standard costs:
		// Base: 53000 (gas_transaction_create)
		// Standard calldata: 10*4 + 10*16 = 200
		// Initcode words: ((20 + 31) / 32) * 2 = 1 * 2 = 2
		// Total standard: 53000 + 200 + 2 = 53202

		// EIP-7623 floor comparison:
		// Standard calldata + execution + contract_creation = 200 + 0 + 2 = 202
		// Floor calldata only = 50 * 10 = 500
		// Since floor (500) > standard (202), should add difference
		// Final cost = 53000 (base) + 500 (floor calldata) + 2 (initcode) = 53502

		// Actually, let's verify what we observe vs what we expect
		let tokens = zero_count as u64 + (non_zero_count as u64 * 4); // 10 + 10*4 = 50
		let standard_calldata = (zero_count as u64 * 4) + (non_zero_count as u64 * 16); // 200
		let floor_calldata = tokens * 10; // 500
		let initcode_cost = 2; // (20+31)/32 * 2 = 2
		let base_cost = config.gas_transaction_create; // 53000

		// Standard path: 200 (calldata) + 2 (execution) + 32002 (contract creation) = 32204
		// Floor path: 500 (floor calldata only)
		// max(32204, 500) = 32204, so no adjustment
		// Total: 53000 (base) + 200 (calldata) + 2 (initcode) = 53202
		let expected_gas = 53_202;
		assert_eq!(
			used_gas,
			expected_gas,
			"Contract creation gas mismatch: expected {}, got {} \
			(zero_count: {}, non_zero_count: {}, standard_calldata: {}, floor_calldata: {}, \
			base_cost: {}, initcode_cost: {})",
			expected_gas,
			used_gas,
			zero_count,
			non_zero_count,
			standard_calldata,
			floor_calldata,
			base_cost,
			initcode_cost
		);

		// Verify components are included
		assert!(
			used_gas >= config.gas_transaction_create,
			"Should include base creation cost"
		);
	}

	#[test]
	fn test_contract_creation_post_execution_investigation() {
		// This test investigates why contract creation doesn't seem to apply floor cost
		let config = create_eip7623_config();
		let gas_limit = 200_000;

		// Use the same initcode as the snapshot test
		let initcode = vec![
			0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0x60, 0x00,
			0x60, 0x00, 0x60, 0x00, 0xf3, 0x00,
		];
		let cost = create_transaction_cost(&initcode, &vec![], &vec![]);

		// Test without post_execution
		let mut gasometer_before = Gasometer::new(gas_limit, &config);
		gasometer_before.record_transaction(cost.clone()).unwrap();
		let gas_before_post = gasometer_before.total_used_gas();

		// Test with post_execution
		let mut gasometer_after = Gasometer::new(gas_limit, &config);
		gasometer_after.record_transaction(cost).unwrap();
		gasometer_after.post_execution().unwrap();
		let gas_after_post = gasometer_after.total_used_gas();

		println!(
			"Contract creation gas before post_execution: {}",
			gas_before_post
		);
		println!(
			"Contract creation gas after post_execution: {}",
			gas_after_post
		);
		println!(
			"Difference: {}",
			gas_after_post as i64 - gas_before_post as i64
		);

		// For comparison, test a regular call with similar calldata
		let call_data = initcode.clone(); // Same bytes as initcode
		let call_cost = call_transaction_cost(&call_data, &vec![], &vec![]);

		let mut call_gasometer = Gasometer::new(gas_limit, &config);
		call_gasometer
			.record_transaction(call_cost.clone())
			.unwrap();
		let call_gas_before = call_gasometer.total_used_gas();

		call_gasometer.post_execution().unwrap();
		let call_gas_after = call_gasometer.total_used_gas();

		println!("Call gas before post_execution: {}", call_gas_before);
		println!("Call gas after post_execution: {}", call_gas_after);
		println!(
			"Call difference: {}",
			call_gas_after as i64 - call_gas_before as i64
		);

		// CORRECT BEHAVIOR:
		// 1. Contract creation shows NO difference before/after post_execution
		//    because the contract creation cost (32002) makes the standard path higher than floor
		// 2. Regular calls show positive difference when floor cost > standard cost
		// 3. This is the correct EIP-7623 behavior as specified

		// Verify correct behavior with assertions
		assert_eq!(
			gas_after_post, gas_before_post,
			"Contract creation should NOT change with post_execution (contract creation cost dominates)"
		);
		assert!(
			call_gas_after > call_gas_before,
			"Regular calls should increase with post_execution due to floor cost"
		);

		// Calculate expected floor increase for call
		let zero_count = call_data.iter().filter(|&&b| b == 0).count();
		let non_zero_count = call_data.len() - zero_count;
		let tokens = zero_count as u64 + (non_zero_count as u64 * 4);
		let standard_calldata = (zero_count as u64 * 4) + (non_zero_count as u64 * 16);
		let floor_calldata = tokens * 10;
		let expected_increase = floor_calldata.saturating_sub(standard_calldata);

		assert_eq!(
			call_gas_after - call_gas_before,
			expected_increase,
			"Call gas increase should match floor - standard difference"
		);
	}

	#[test]
	fn test_snapshot_comparison_with_without_eip7623() {
		// Test the difference in gas consumption with and without EIP-7623
		let data = vec![0xffu8; 500]; // 500 non-zero bytes

		// Without EIP-7623
		let config_without = create_pre_eip7623_config();
		let mut gasometer_without = Gasometer::new(100_000, &config_without);
		let cost = call_transaction_cost(&data, &vec![], &vec![]);
		gasometer_without.record_transaction(cost.clone()).unwrap();
		let gas_without = gasometer_without.total_used_gas();

		// With EIP-7623
		let config_with = create_eip7623_config();
		let mut gasometer_with = Gasometer::new(100_000, &config_with);
		gasometer_with.record_transaction(cost).unwrap();
		gasometer_with.post_execution().unwrap();
		let gas_with = gasometer_with.total_used_gas();

		// Without EIP-7623:
		// Base: 21000 + 500*16 = 29000
		assert_eq!(gas_without, 29_000, "Gas without EIP-7623 mismatch");

		// With EIP-7623:
		// Standard: 21000 + 500*16 = 29000
		// Floor: 21000 + 500*4*10 = 41000
		// Should use max(29000, 41000) = 41000
		assert_eq!(gas_with, 41_000, "Gas with EIP-7623 mismatch");

		// EIP-7623 should increase cost for large calldata
		assert!(
			gas_with > gas_without,
			"EIP-7623 should increase gas for large calldata"
		);

		// The increase should be exactly the difference between floor and standard
		assert_eq!(
			gas_with - gas_without,
			12_000,
			"Gas increase should match floor - standard difference"
		);
	}
}

// ============================================================================
// Section 7: Integration Tests with Full Transaction Execution
// ============================================================================

#[cfg(test)]
mod integration_tests {
	use super::*;

	#[test]
	fn test_full_transaction_execution_with_eip7623() {
		let caller = H160::from_slice(&[1u8; 20]);
		let target = H160::from_slice(&[2u8; 20]);

		let config = create_eip7623_config();

		// Create initial state
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
			target,
			evm::backend::MemoryAccount {
				nonce: U256::zero(),
				balance: U256::zero(),
				storage: BTreeMap::new(),
				// Simple contract that stores a value
				code: vec![
					0x60, 0x01, // PUSH1 0x01
					0x60, 0x00, // PUSH1 0x00
					0x55, // SSTORE
				],
			},
		);

		let vicinity = create_test_vicinity();
		let mut backend = MemoryBackend::new(&vicinity, state);

		// Create large calldata to trigger floor cost
		let calldata = vec![0xffu8; 1000];
		let gas_limit = 500_000;

		let metadata = StackSubstateMetadata::new(gas_limit, &config);
		let state = MemoryStackState::new(metadata, &mut backend);
		let mut precompiles = ();
		let mut executor = StackExecutor::new_with_precompiles(state, &config, &mut precompiles);

		let authorization_list = vec![];
		let (exit_reason, _result) = executor.transact_call(
			caller,
			target,
			U256::zero(),
			calldata.clone(),
			gas_limit,
			vec![],
			authorization_list,
		);

		match exit_reason {
			ExitReason::Succeed(_) => {
				let gas_used = executor.used_gas();

				// Calculate expected minimum gas with floor cost
				let tokens = calculate_tokens_in_calldata(0, calldata.len());
				let floor_cost = tokens * TOTAL_COST_FLOOR_PER_TOKEN;

				assert!(
					gas_used >= floor_cost,
					"Gas used ({}) should be at least floor cost ({})",
					gas_used,
					floor_cost
				);
			}
			_ => panic!("Transaction should succeed, got {:?}", exit_reason),
		}
	}

	#[test]
	fn test_contract_deployment_with_eip7623() {
		let deployer = H160::from_slice(&[1u8; 20]);

		let config = create_eip7623_config();

		// Create initial state
		let mut state = BTreeMap::new();
		state.insert(
			deployer,
			evm::backend::MemoryAccount {
				nonce: U256::zero(),
				balance: U256::from(10_000_000),
				storage: BTreeMap::new(),
				code: Vec::new(),
			},
		);

		let vicinity = create_test_vicinity();
		let mut backend = MemoryBackend::new(&vicinity, state);

		// Contract initcode that deploys a simple storage contract
		let initcode = vec![
			// Constructor code
			0x60, 0x0a, // PUSH1 0x0a (size of runtime code)
			0x60, 0x0c, // PUSH1 0x0c (offset of runtime code)
			0x60, 0x00, // PUSH1 0x00 (destination in memory)
			0x39, // CODECOPY
			0x60, 0x0a, // PUSH1 0x0a (size to return)
			0x60, 0x00, // PUSH1 0x00 (offset to return)
			0xf3, // RETURN
			// Runtime code (10 bytes)
			0x60, 0x42, // PUSH1 0x42
			0x60, 0x00, // PUSH1 0x00
			0x55, // SSTORE
			0x60, 0x01, // PUSH1 0x01
			0x60, 0x00, // PUSH1 0x00
			0xf3, // RETURN
		];

		let gas_limit = 500_000;

		let metadata = StackSubstateMetadata::new(gas_limit, &config);
		let state = MemoryStackState::new(metadata, &mut backend);
		let mut precompiles = ();
		let mut executor = StackExecutor::new_with_precompiles(state, &config, &mut precompiles);

		let authorization_list = vec![];
		let (exit_reason, _result) = executor.transact_create(
			deployer,
			U256::zero(),
			initcode.clone(),
			gas_limit,
			vec![],
			authorization_list,
		);

		match exit_reason {
			ExitReason::Succeed(_) => {
				let gas_used = executor.used_gas();

				// Calculate expected costs
				let tokens = calculate_tokens_in_calldata(
					initcode.iter().filter(|&&b| b == 0).count(),
					initcode.iter().filter(|&&b| b != 0).count(),
				);
				let floor_cost = tokens * TOTAL_COST_FLOOR_PER_TOKEN;

				// Contract creation should include initcode cost
				let words = (initcode.len() + 31) / 32;
				let initcode_word_cost = INITCODE_WORD_COST * words as u64;

				println!(
					"Gas used: {}, Floor cost: {}, Initcode cost: {}",
					gas_used, floor_cost, initcode_word_cost
				);

				// Verify gas consumption includes necessary costs
				assert!(
					gas_used >= config.gas_transaction_create,
					"Should include base creation cost"
				);
			}
			_ => panic!("Contract deployment should succeed, got {:?}", exit_reason),
		}
	}

	#[test]
	fn test_comparison_regular_vs_large_calldata_transaction() {
		let caller = H160::from_slice(&[1u8; 20]);
		let target = H160::from_slice(&[2u8; 20]);

		let config = create_eip7623_config();

		// Test with small calldata (regular transaction)
		let small_calldata = vec![0x01, 0x02, 0x03, 0x04]; // 4 bytes

		// Test with large calldata (should trigger floor cost)
		let large_calldata = vec![0xffu8; 10000]; // 10KB

		// Create initial state
		let mut state = BTreeMap::new();
		state.insert(
			caller,
			evm::backend::MemoryAccount {
				nonce: U256::zero(),
				balance: U256::from(100_000_000),
				storage: BTreeMap::new(),
				code: Vec::new(),
			},
		);

		state.insert(
			target,
			evm::backend::MemoryAccount {
				nonce: U256::zero(),
				balance: U256::zero(),
				storage: BTreeMap::new(),
				code: vec![0x00], // STOP
			},
		);

		let vicinity = create_test_vicinity();

		// Execute small calldata transaction
		let mut backend = MemoryBackend::new(&vicinity, state.clone());
		let metadata = StackSubstateMetadata::new(1_000_000, &config);
		let state_small = MemoryStackState::new(metadata, &mut backend);
		let mut precompiles_small = ();
		let mut executor_small =
			StackExecutor::new_with_precompiles(state_small, &config, &mut precompiles_small);

		let (exit_small, _) = executor_small.transact_call(
			caller,
			target,
			U256::zero(),
			small_calldata.clone(),
			1_000_000,
			vec![],
			vec![],
		);

		assert!(matches!(exit_small, ExitReason::Succeed(_)));
		let gas_used_small = executor_small.used_gas();

		// Execute large calldata transaction
		let mut backend = MemoryBackend::new(&vicinity, state);
		let metadata = StackSubstateMetadata::new(1_000_000, &config);
		let state_large = MemoryStackState::new(metadata, &mut backend);
		let mut precompiles_large = ();
		let mut executor_large =
			StackExecutor::new_with_precompiles(state_large, &config, &mut precompiles_large);

		let (exit_large, _) = executor_large.transact_call(
			caller,
			target,
			U256::zero(),
			large_calldata.clone(),
			1_000_000,
			vec![],
			vec![],
		);

		assert!(matches!(exit_large, ExitReason::Succeed(_)));
		let gas_used_large = executor_large.used_gas();

		// Calculate expected floor costs
		let tokens_large = calculate_tokens_in_calldata(0, large_calldata.len());
		let floor_cost_large = tokens_large * TOTAL_COST_FLOOR_PER_TOKEN;

		println!("Small transaction gas: {}", gas_used_small);
		println!("Large transaction gas: {}", gas_used_large);
		println!("Expected floor cost for large: {}", floor_cost_large);

		// Large calldata should use significantly more gas due to floor cost
		// The ratio should be significant but not necessarily 100x
		// With 4 bytes vs 10,000 bytes, we expect at least 10x more gas
		assert!(
			gas_used_large > gas_used_small * 10,
			"Large calldata should use much more gas than small calldata: {} vs {}",
			gas_used_large,
			gas_used_small
		);

		// Verify floor cost is being applied
		assert!(
			gas_used_large >= floor_cost_large,
			"Large transaction should meet floor cost requirement"
		);
	}
}
