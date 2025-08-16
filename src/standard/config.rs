use evm_interpreter::runtime::RuntimeConfig;

/// Runtime configuration.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Config {
	/// Runtime config.
	pub runtime: RuntimeConfig,
	/// Disallow empty contract creation.
	pub eip2_no_empty_contract: bool,
	/// Increase contract creation transaction cost.
	pub eip2_create_transaction_increase: bool,
	/// EIP-1884: trie repricing.
	pub eip1884_trie_repricing: bool,
	/// EIP-1283.
	pub eip2200_sstore_gas_metering: bool,
	/// EIP-1706.
	pub eip2200_sstore_revert_under_stipend: bool,
	/// EIP-2929
	pub eip2929_increase_state_access_gas: bool,
	/// EIP-3529
	pub eip3529_decrease_clears_refund: bool,
	/// EIP-3541
	pub eip3541_disallow_executable_format: bool,
	/// Gas increases of EIP150.
	pub eip150_gas_increase: bool,
	/// Whether to throw out of gas error when
	/// CALL/CALLCODE/DELEGATECALL requires more than maximum amount
	/// of gas.
	pub eip150_no_err_on_call_with_more_gas: bool,
	/// Take l64 for callcreate after gas.
	pub eip150_call_l64_after_gas: bool,
	/// Whether create transactions and create opcode increases nonce by one.
	pub eip161_create_increase_nonce: bool,
	/// EIP170.
	pub eip170_create_contract_limit: bool,
	/// EIP-3860, maximum size limit of init_code.
	pub eip3860_max_initcode_size: bool,
	/// Has delegate call.
	pub eip7_delegate_call: bool,
	/// Has create2.
	pub eip1014_create2: bool,
	/// Has revert.
	pub eip140_revert: bool,
	/// EIP160.
	pub eip160_exp_increase: bool,
	/// Has return data.
	pub eip211_return_data: bool,
	/// Static call.
	pub eip214_static_call: bool,
	/// Has bitwise shifting.
	pub eip145_bitwise_shifting: bool,
	/// Has chain ID.
	pub eip1344_chain_id: bool,
	/// Has self balance.
	pub eip1884_self_balance: bool,
	/// Has ext code hash.
	pub eip1052_ext_code_hash: bool,
	/// Has ext block fee. See [EIP-3198](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-3198.md)
	pub eip3198_base_fee: bool,
	/// Has PUSH0 opcode. See [EIP-3855](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-3855.md)
	pub eip3855_push0: bool,
	/// Enables transient storage. See [EIP-1153](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1153.md)
	pub eip1153_transient_storage: bool,
	/// Enables MCOPY instruction. See [EIP-5656](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-5656.md)
	pub eip5656_mcopy: bool,
	/// Uses EIP-1559 (Base fee is burned when this flag is enabled) [EIP-1559](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1559.md)
	pub eip1559_fee_market: bool,
	/// Call data gas cost reduction.
	pub eip2028_transaction_calldata_decrease: bool,
	/// EIP-198: Modexp precompile.
	pub eip198_modexp_precompile: bool,
	/// EIP-196: EC ADD/MUL precompile.
	pub eip196_ec_add_mul_precompile: bool,
	/// EIP-197: EC Pairing precompile.
	pub eip197_ec_pairing_precompile: bool,
	/// EIP-152: Blake2F precompile.
	pub eip152_blake_2f_precompile: bool,
	/// EIP-1108: Reduce EC ADD/MUL/Pairing costs.
	pub eip1108_ec_add_mul_pairing_decrease: bool,
	/// EIP-2565.
	pub eip2565_lower_modexp: bool,
	/// EIP-2930: Optional access list.
	pub eip2930_access_list: bool,
	/// EIP-7516: Blob base fee per gas.
	pub eip7516_blob_base_fee: bool,
}

impl Config {
	/// Frontier hard fork configuration.
	pub const fn frontier() -> Config {
		Config {
			runtime: RuntimeConfig::frontier(),
			eip2_no_empty_contract: false,
			eip2_create_transaction_increase: false,
			eip2200_sstore_gas_metering: false,
			eip2200_sstore_revert_under_stipend: false,
			eip2929_increase_state_access_gas: false,
			eip3529_decrease_clears_refund: false,
			eip3541_disallow_executable_format: false,
			eip150_no_err_on_call_with_more_gas: false,
			eip161_create_increase_nonce: false,
			eip150_call_l64_after_gas: false,
			eip170_create_contract_limit: false,
			eip3860_max_initcode_size: false,
			eip7_delegate_call: false,
			eip1014_create2: false,
			eip140_revert: false,
			eip211_return_data: false,
			eip145_bitwise_shifting: false,
			eip1344_chain_id: false,
			eip1884_self_balance: false,
			eip1052_ext_code_hash: false,
			eip3198_base_fee: false,
			eip3855_push0: false,
			eip1153_transient_storage: false,
			eip5656_mcopy: false,
			eip1559_fee_market: false,
			eip198_modexp_precompile: false,
			eip196_ec_add_mul_precompile: false,
			eip197_ec_pairing_precompile: false,
			eip152_blake_2f_precompile: false,
			eip150_gas_increase: false,
			eip160_exp_increase: false,
			eip1884_trie_repricing: false,
			eip214_static_call: false,
			eip1108_ec_add_mul_pairing_decrease: false,
			eip2028_transaction_calldata_decrease: false,
			eip2565_lower_modexp: false,
			eip2930_access_list: false,
			eip7516_blob_base_fee: false,
		}
	}

	/// Homestead
	pub const fn homestead() -> Config {
		let mut config = Self::frontier();
		config.eip2_no_empty_contract = true;
		config.eip2_create_transaction_increase = true;
		config.eip7_delegate_call = true;
		config
	}

	/// Tangerine whistle
	pub const fn tangerine_whistle() -> Config {
		let mut config = Self::homestead();
		config.eip150_gas_increase = true;
		config.eip150_no_err_on_call_with_more_gas = true;
		config.eip150_call_l64_after_gas = true;
		config
	}

	/// Spurious dragon
	pub const fn spurious_dragon() -> Config {
		let mut config = Self::tangerine_whistle();
		config.runtime.eip161_empty_check = true;
		config.eip161_create_increase_nonce = true;
		config.eip160_exp_increase = true;
		config.eip170_create_contract_limit = true;
		config
	}

	/// Byzantium
	pub const fn byzantium() -> Config {
		let mut config = Self::spurious_dragon();
		config.eip140_revert = true;
		config.eip196_ec_add_mul_precompile = true;
		config.eip197_ec_pairing_precompile = true;
		config.eip198_modexp_precompile = true;
		config.eip211_return_data = true;
		config.eip214_static_call = true;
		config
	}

	/// Petersburg
	pub const fn petersburg() -> Config {
		let mut config = Self::byzantium();
		config.eip145_bitwise_shifting = true;
		config.eip1014_create2 = true;
		config.eip1052_ext_code_hash = true;
		config
	}

	/// Istanbul hard fork configuration.
	pub const fn istanbul() -> Config {
		let mut config = Self::petersburg();
		config.eip152_blake_2f_precompile = true;
		config.eip1108_ec_add_mul_pairing_decrease = true;
		config.eip1344_chain_id = true;
		config.eip1884_trie_repricing = true;
		config.eip1884_self_balance = true;
		config.eip2028_transaction_calldata_decrease = true;
		config.eip2200_sstore_gas_metering = true;
		config.eip2200_sstore_revert_under_stipend = true;
		config
	}

	/// Berlin
	pub const fn berlin() -> Config {
		let mut config = Self::istanbul();
		config.eip2565_lower_modexp = true;
		config.eip2929_increase_state_access_gas = true;
		config.eip2930_access_list = true;
		config
	}

	/// London
	pub const fn london() -> Config {
		let mut config = Self::berlin();
		config.eip1559_fee_market = true;
		config.eip3198_base_fee = true;
		config.eip3529_decrease_clears_refund = true;
		config.eip3541_disallow_executable_format = true;
		config
	}

	/// Shanghai
	pub const fn shanghai() -> Config {
		let mut config = Self::london();
		config.runtime.eip3651_warm_coinbase_address = true;
		config.eip3855_push0 = true;
		config.eip3860_max_initcode_size = true;
		config
	}

	/// Cancun
	pub const fn cancun() -> Config {
		let mut config = Self::shanghai();
		config.eip1153_transient_storage = true;
		config.eip5656_mcopy = true;
		config.runtime.eip6780_suicide_only_in_same_tx = true;
		// TODO: EIP-7516.
		config
	}

	/// Gas paid for extcode.
	pub fn gas_ext_code(&self) -> u64 {
		if self.eip150_gas_increase { 700 } else { 20 }
	}

	/// Gas paid for extcodehash.
	pub fn gas_ext_code_hash(&self) -> u64 {
		if self.eip1884_trie_repricing {
			700
		} else {
			400
		}
	}

	/// Gas paid for sstore set.
	pub fn gas_sstore_set(&self) -> u64 {
		20000
	}

	/// Gas paid for sstore reset.
	pub fn gas_sstore_reset(&self) -> u64 {
		if self.eip2929_increase_state_access_gas {
			2900
		} else {
			5000
		}
	}

	/// Gas paid for sstore refund.
	pub fn refund_sstore_clears(&self) -> i64 {
		if self.eip3529_decrease_clears_refund {
			4800
		} else {
			15000
		}
	}

	/// EIP-3529
	pub fn max_refund_quotient(&self) -> u64 {
		if self.eip3529_decrease_clears_refund {
			5
		} else {
			2
		}
	}

	/// Gas paid for BALANCE opcode.
	pub fn gas_balance(&self) -> u64 {
		if self.eip1884_trie_repricing {
			700
		} else if self.eip150_gas_increase {
			400
		} else {
			20
		}
	}

	/// Gas paid for SLOAD opcode.
	pub fn gas_sload(&self) -> u64 {
		if self.eip2929_increase_state_access_gas {
			100
		} else if self.eip2200_sstore_gas_metering {
			800
		} else if self.eip150_gas_increase {
			200
		} else {
			50
		}
	}

	/// Gas paid for cold SLOAD opcode.
	pub fn gas_sload_cold(&self) -> u64 {
		if self.eip2929_increase_state_access_gas {
			2100
		} else {
			0
		}
	}

	/// Gas paid for SUICIDE opcode.
	pub fn gas_suicide(&self) -> u64 {
		if self.eip150_gas_increase { 5000 } else { 0 }
	}

	/// Gas paid for SUICIDE opcode when it hits a new account.
	pub fn gas_suicide_new_account(&self) -> u64 {
		if self.eip150_gas_increase { 25000 } else { 0 }
	}

	/// Gas paid for CALL opcode.
	pub fn gas_call(&self) -> u64 {
		if self.eip150_gas_increase { 700 } else { 40 }
	}

	/// Gas paid for EXP opcode for every byte.
	pub fn gas_expbyte(&self) -> u64 {
		if self.eip160_exp_increase { 50 } else { 10 }
	}

	/// Gas paid for a contract creation transaction.
	pub fn gas_transaction_create(&self) -> u64 {
		if self.eip2_create_transaction_increase {
			53000
		} else {
			21000
		}
	}

	/// Gas paid for a message call transaction.
	pub fn gas_transaction_call(&self) -> u64 {
		21000
	}

	/// Gas paid for zero data in a transaction.
	pub fn gas_transaction_zero_data(&self) -> u64 {
		4
	}

	/// Gas paid for non-zero data in a transaction.
	pub fn gas_transaction_non_zero_data(&self) -> u64 {
		if self.eip2028_transaction_calldata_decrease {
			16
		} else {
			68
		}
	}

	/// Gas paid per address in transaction access list (see EIP-2930).
	pub fn gas_access_list_address(&self) -> u64 {
		if self.eip2930_access_list { 2400 } else { 0 }
	}

	/// Gas paid per storage key in transaction access list (see EIP-2930).
	pub fn gas_access_list_storage_key(&self) -> u64 {
		if self.eip2930_access_list { 1900 } else { 0 }
	}

	/// Gas paid for accessing cold account.
	pub fn gas_account_access_cold(&self) -> u64 {
		if self.eip2929_increase_state_access_gas {
			2600
		} else {
			0
		}
	}

	/// Gas paid for accessing ready storage.
	pub fn gas_storage_read_warm(&self) -> u64 {
		if self.eip2929_increase_state_access_gas {
			100
		} else {
			0
		}
	}

	/// Stack limit.
	pub fn stack_limit(&self) -> usize {
		1024
	}

	/// Memory limit.
	pub fn memory_limit(&self) -> usize {
		usize::MAX
	}

	/// Call stack limit.
	pub fn call_stack_limit(&self) -> usize {
		1024
	}

	/// Call stipend.
	pub fn call_stipend(&self) -> u64 {
		2300
	}

	/// Maximum size limit of init code.
	pub fn max_initcode_size(&self) -> Option<usize> {
		if self.eip3860_max_initcode_size {
			Some(0xc000)
		} else {
			None
		}
	}

	/// Create contract limit.
	pub fn create_contract_limit(&self) -> Option<usize> {
		if self.eip170_create_contract_limit {
			Some(0x6000)
		} else {
			None
		}
	}
}
