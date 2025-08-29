#[cfg(test)]
mod eip7623_tests {
	use evm::Config;
	use evm_gasometer::{call_transaction_cost, create_transaction_cost, Gasometer};
	use primitive_types::{H160, H256};

	#[test]
	fn test_eip7623_call_transaction_standard_wins() {
		// Test case where standard cost is higher than floor cost
		let config = Config::pectra();

		// Small calldata: 10 bytes (5 zero, 5 non-zero)
		let data = vec![0, 0, 0, 0, 0, 1, 2, 3, 4, 5];
		let cost = call_transaction_cost(&data, &[], &[]);

		let mut gasometer = Gasometer::new(1_000_000, &config);
		assert!(gasometer.record_transaction(cost).is_ok());

		// Standard cost: 21000 + 5*4 + 5*16 = 21000 + 20 + 80 = 21100
		// Floor cost: 10 * 10 = 100
		// Max(21100, 100) = 21100
		assert_eq!(gasometer.total_used_gas(), 21100);
	}

	#[test]
	fn test_eip7623_call_transaction_floor_wins() {
		// Test case where floor cost is higher than standard cost
		let config = Config::pectra();

		// Large calldata: 3000 bytes of zeros
		let data = vec![0; 3000];
		let cost = call_transaction_cost(&data, &[], &[]);

		let mut gasometer = Gasometer::new(100_000, &config);
		assert!(gasometer.record_transaction(cost).is_ok());
		
		// Initially only standard cost is applied: 21000 + 3000*4 = 33000
		assert_eq!(gasometer.total_used_gas(), 33000);
		
		// Simulate some execution gas (e.g., 5000)
		assert!(gasometer.record_cost(5000).is_ok());
		
		// Apply post-execution adjustments
		assert!(gasometer.post_execution().is_ok());

		// After adjustment:
		// Standard: 3000*4 + 5000 = 17000
		// Floor: 3000 * 10 = 30000
		// Adjustment: 30000 - 17000 = 13000
		// Total: 33000 + 5000 + 13000 = 51000
		assert_eq!(gasometer.total_used_gas(), 51000);
	}

	#[test]
	fn test_eip7623_call_transaction_floor_wins_large() {
		// Test case where floor cost definitely wins
		let config = Config::pectra();

		// Very large calldata: 10000 bytes of zeros
		let data = vec![0; 10000];
		let cost = call_transaction_cost(&data, &[], &[]);

		let mut gasometer = Gasometer::new(200_000, &config);
		assert!(gasometer.record_transaction(cost).is_ok());
		
		// Initially only standard cost: 21000 + 10000*4 = 61000
		assert_eq!(gasometer.total_used_gas(), 61000);
		
		// Simulate some execution gas
		assert!(gasometer.record_cost(5000).is_ok());
		
		// Apply EIP-7623 adjustment
		assert!(gasometer.post_execution().is_ok());

		// After adjustment:
		// Standard: 10000*4 + 5000 = 45000
		// Floor: 10000 * 10 = 100000
		// Adjustment: 100000 - 45000 = 55000
		// Total: 61000 + 5000 + 55000 = 121000
		assert_eq!(gasometer.total_used_gas(), 121000);
	}

	#[test]
	fn test_eip7623_create_transaction() {
		// Test create transaction with EIP-7623
		let config = Config::pectra();

		// Initcode with mixed data
		let data = vec![0x60, 0x80, 0x60, 0x40, 0x52, 0, 0, 0, 0, 0]; // 5 non-zero, 5 zero
		let cost = create_transaction_cost(&data, &[], &[]);

		let mut gasometer = Gasometer::new(100_000, &config);
		assert!(gasometer.record_transaction(cost).is_ok());

		// Standard cost: 53000 + 5*4 + 5*16 = 53000 + 20 + 80 = 53100
		// Floor cost: 10 * 10 = 100
		// Max(53100, 100) = 53100
		assert_eq!(gasometer.total_used_gas(), 53102);
	}

	#[test]
	fn test_eip7623_disabled() {
		// Test that when EIP-7623 is disabled, only standard cost applies
		let config = Config::london();

		// Large calldata that would trigger floor cost if enabled
		let data = vec![0; 10000];
		let cost = call_transaction_cost(&data, &[], &[]);

		let mut gasometer = Gasometer::new(100_000, &config);
		assert!(gasometer.record_transaction(cost).is_ok());

		// Standard cost only: 21000 + 10000*4 = 61000
		assert_eq!(gasometer.total_used_gas(), 61000);
	}

	#[test]
	fn test_eip7623_mixed_calldata() {
		// Test with mixed zero and non-zero bytes
		let config = Config::pectra();

		// 1000 zeros and 1000 non-zeros
		let mut data = vec![0; 1000];
		data.extend(vec![0xFF; 1000]);
		let cost = call_transaction_cost(&data, &[], &[]);

		let mut gasometer = Gasometer::new(100_000, &config);
		assert!(gasometer.record_transaction(cost).is_ok());

		// Standard cost: 21000 + 1000*4 + 1000*16 = 21000 + 4000 + 16000 = 41000
		// Floor cost: 2000 * 10 = 20000
		// Max(41000, 20000) = 41000
		assert_eq!(gasometer.total_used_gas(), 41000);
	}

	#[test]
	fn test_eip7623_with_access_list() {
		// Test transaction with access list
		let config = Config::pectra();

		// Small calldata with access list
		let data = vec![1, 2, 3, 4, 5];
		let access_list = vec![
			(H160::zero(), vec![H256::zero(), H256::from_low_u64_be(1)]),
			(H160::from_low_u64_be(1), vec![H256::zero()]),
		];
		let cost = call_transaction_cost(&data, &access_list, &[]);

		let mut gasometer = Gasometer::new(100_000, &config);
		assert!(gasometer.record_transaction(cost).is_ok());

		// Standard cost: 21000 + 0*4 + 5*16 + 2*2400 + 3*1900 = 21000 + 80 + 4800 + 5700 = 31580
		// Floor cost: 5 * 10 = 50
		// Max(31580, 50) = 31580
		assert_eq!(gasometer.total_used_gas(), 31580);
	}

	#[test]
	fn test_eip7623_exact_boundary() {
		// Test the exact boundary where floor cost equals standard cost
		let config = Config::pectra();

		// For zero bytes: standard = 4, floor = 10
		// With execution gas, we can reach equilibrium
		let data = vec![0; 3500];
		let cost = call_transaction_cost(&data, &[], &[]);

		let mut gasometer = Gasometer::new(100_000, &config);
		assert!(gasometer.record_transaction(cost).is_ok());
		
		// Initially: 21000 + 3500*4 = 35000
		assert_eq!(gasometer.total_used_gas(), 35000);
		
		// Add execution gas to reach equilibrium
		// Floor: 3500 * 10 = 35000
		// Standard calldata: 3500 * 4 = 14000
		// Need execution gas: 35000 - 14000 = 21000
		assert!(gasometer.record_cost(21000).is_ok());
		
		// Apply EIP-7623 adjustment
		assert!(gasometer.post_execution().is_ok());

		// After adjustment:
		// Standard: 14000 + 21000 = 35000
		// Floor: 35000
		// No adjustment needed (they're equal)
		// Total: 35000 + 21000 = 56000
		assert_eq!(gasometer.total_used_gas(), 56000);
	}
}
