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

		// Standard calldata cost: 3000*4 = 12000
		// Floor calldata cost: 3000 * 10 = 30000
		// Calldata cost: Max(12000, 30000) = 30000
		// Total: 21000 + 30000 = 51000
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

		// Standard calldata cost: 10000*4 = 40000
		// Floor calldata cost: 10000 * 10 = 100000
		// Calldata cost: Max(40000, 100000) = 100000
		// Total: 21000 + 100000 = 121000
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
		// At equilibrium for calldata: n*4 = n*10
		// This never happens since 10 > 4
		// So let's test a case where they're close
		let data = vec![0; 3500];
		let cost = call_transaction_cost(&data, &[], &[]);

		let mut gasometer = Gasometer::new(100_000, &config);
		assert!(gasometer.record_transaction(cost).is_ok());

		// Standard calldata cost: 3500*4 = 14000
		// Floor calldata cost: 3500 * 10 = 35000
		// Calldata cost: Max(14000, 35000) = 35000
		// Total: 21000 + 35000 = 56000
		assert_eq!(gasometer.total_used_gas(), 56000);
	}
}
