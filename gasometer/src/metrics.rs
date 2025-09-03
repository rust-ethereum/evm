use evm_runtime::Config;

/// Tracks gas parameters for a Gasometer instance.
#[derive(Clone, Debug)]
pub struct GasMetrics {
	zero_bytes_in_calldata: usize,
	non_zero_bytes_in_calldata: usize,
	is_contract_creation: bool,
	// Cached values
	cached_standard_calldata_cost: Option<u64>,
	cached_floor_calldata_cost: Option<u64>,
	cached_base_cost: Option<u64>,
	cached_init_code_cost: Option<u64>,
	cached_contract_creation_cost: Option<u64>,
}

impl GasMetrics {
	pub fn new() -> Self {
		Self {
			zero_bytes_in_calldata: 0,
			non_zero_bytes_in_calldata: 0,
			is_contract_creation: false,
			cached_standard_calldata_cost: None,
			cached_floor_calldata_cost: None,
			cached_base_cost: None,
			cached_init_code_cost: None,
			cached_contract_creation_cost: None,
		}
	}

	fn invalidate_cache(&mut self) {
		self.cached_standard_calldata_cost = None;
		self.cached_floor_calldata_cost = None;
		self.cached_base_cost = None;
		self.cached_init_code_cost = None;
		self.cached_contract_creation_cost = None;
	}

	pub fn set_calldata_params(&mut self, zero_bytes: usize, non_zero_bytes: usize) {
		self.zero_bytes_in_calldata = zero_bytes;
		self.non_zero_bytes_in_calldata = non_zero_bytes;
		self.invalidate_cache();
	}

	pub fn set_contract_creation(&mut self, is_creation: bool) {
		self.is_contract_creation = is_creation;
		self.invalidate_cache();
	}

	pub fn standard_calldata_cost(&mut self, config: &Config) -> u64 {
		if let Some(cached) = self.cached_standard_calldata_cost {
			return cached;
		}

		let cost = (config.gas_transaction_zero_data * (self.zero_bytes_in_calldata as u64))
			+ (config.gas_transaction_non_zero_data * (self.non_zero_bytes_in_calldata as u64));
		self.cached_standard_calldata_cost = Some(cost);
		cost
	}

	pub fn floor_calldata_cost(&mut self, config: &Config) -> u64 {
		if let Some(cached) = self.cached_floor_calldata_cost {
			return cached;
		}

		let cost = (config.gas_calldata_zero_floor * (self.zero_bytes_in_calldata as u64))
			+ (config.gas_calldata_non_zero_floor * (self.non_zero_bytes_in_calldata as u64));
		self.cached_floor_calldata_cost = Some(cost);
		cost
	}

	fn base_cost(&mut self, config: &Config) -> u64 {
		if let Some(cached) = self.cached_base_cost {
			return cached;
		}

		let cost = if self.is_contract_creation {
			config.gas_transaction_create
		} else {
			config.gas_transaction_call
		};
		self.cached_base_cost = Some(cost);
		cost
	}

	pub fn init_code_cost(&mut self) -> u64 {
		if let Some(cached) = self.cached_init_code_cost {
			return cached;
		}

		let cost = if self.is_contract_creation {
			super::init_code_cost(
				self.zero_bytes_in_calldata as u64 + self.non_zero_bytes_in_calldata as u64,
			)
		} else {
			0
		};
		self.cached_init_code_cost = Some(cost);
		cost
	}

	pub fn contract_creation_cost(&mut self, config: &Config) -> u64 {
		if let Some(cached) = self.cached_contract_creation_cost {
			return cached;
		}

		let cost = if self.is_contract_creation {
			(config.gas_transaction_create - config.gas_transaction_call) + self.init_code_cost()
		} else {
			0
		};
		self.cached_contract_creation_cost = Some(cost);
		cost
	}

	/// Gas consumed during transaction execution, excluding base transaction costs,
	/// calldata costs, and contract creation costs. This value only represents
	/// the actual execution cost within post_execution() invocation.
	pub fn non_intrinsic_cost(&mut self, used_gas: u64, config: &Config) -> u64 {
		used_gas
			.saturating_sub(self.base_cost(config))
			.saturating_sub(self.init_code_cost())
			.saturating_sub(self.standard_calldata_cost(config))
	}
}
