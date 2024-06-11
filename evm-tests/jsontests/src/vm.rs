use crate::state::{TestExecutionResult, VerboseOutput};
use evm::backend::{ApplyBackend, MemoryAccount, MemoryBackend, MemoryVicinity};
use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};
use evm::Config;
use primitive_types::{H160, H256, U256};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Deserialize, Debug)]
pub struct Test(ethjson::vm::Vm);

impl Test {
	fn unwrap_to_pre_state(&self) -> BTreeMap<H160, MemoryAccount> {
		crate::utils::unwrap_to_state(&self.0.pre_state)
	}

	fn unwrap_to_vicinity(&self) -> MemoryVicinity {
		let block_randomness = self.0.env.random.map(|r| {
			// Convert between U256 and H256. U256 is in little-endian but since H256 is just
			// a string-like byte array, it's big endian (MSB is the first element of the array).
			//
			// Byte order here is important because this opcode has the same value as DIFFICULTY
			// (0x44), and so for older forks of Ethereum, the threshold value of 2^64 is used to
			// distinguish between the two: if it's below, the value corresponds to the DIFFICULTY
			// opcode, otherwise to the PREVRANDAO opcode.
			let mut buf = [0u8; 32];
			r.0.to_big_endian(&mut buf);
			H256(buf)
		});

		MemoryVicinity {
			gas_price: self.0.transaction.gas_price.into(),
			effective_gas_price: self.0.transaction.gas_price.into(),
			origin: self.0.transaction.origin.into(),
			block_hashes: Vec::new(),
			block_number: self.0.env.number.into(),
			block_coinbase: self.0.env.author.into(),
			block_timestamp: self.0.env.timestamp.into(),
			block_difficulty: self.0.env.difficulty.into(),
			block_gas_limit: self.0.env.gas_limit.into(),
			chain_id: U256::zero(),
			block_base_fee_per_gas: self.0.transaction.gas_price.into(),
			block_randomness,
			blob_gas_price: None,
			blob_hashes: Vec::new(),
		}
	}

	fn unwrap_to_code(&self) -> Rc<Vec<u8>> {
		Rc::new(self.0.transaction.code.clone().into())
	}

	fn unwrap_to_data(&self) -> Rc<Vec<u8>> {
		Rc::new(self.0.transaction.data.clone().into())
	}

	fn unwrap_to_context(&self) -> evm::Context {
		evm::Context {
			address: self.0.transaction.address.into(),
			caller: self.0.transaction.sender.into(),
			apparent_value: self.0.transaction.value.into(),
		}
	}

	fn unwrap_to_return_value(&self) -> Vec<u8> {
		self.0.output.clone().unwrap().into()
	}

	fn unwrap_to_gas_limit(&self) -> u64 {
		self.0.transaction.gas.into()
	}

	fn unwrap_to_post_gas(&self) -> u64 {
		self.0.gas_left.unwrap().into()
	}

	fn check_valid_state(&self, b: &BTreeMap<H160, MemoryAccount>) -> bool {
		let post_state = self.0.post_state.as_ref().unwrap();
		match &post_state.0 {
			ethjson::spec::HashOrMap::Map(m) => {
				&m.iter()
					.map(|(k, v)| ((*k).into(), crate::utils::unwrap_to_account(v)))
					.collect::<BTreeMap<_, _>>()
					== b
			}
			ethjson::spec::HashOrMap::Hash(h) => {
				let x = crate::utils::check_valid_hash(&(*h).into(), b);
				!x.0
			}
		}
	}
}

pub fn test(verbose_output: &VerboseOutput, name: &str, test: Test) -> TestExecutionResult {
	let mut result = TestExecutionResult::new();
	let mut failed = false;
	result.total = 1;
	if verbose_output.verbose {
		print!("Running test {} ... ", name);
		crate::utils::flush();
	}

	let original_state = test.unwrap_to_pre_state();
	let vicinity = test.unwrap_to_vicinity();
	let config = Config::frontier();
	let mut backend = MemoryBackend::new(&vicinity, original_state);
	let metadata = StackSubstateMetadata::new(test.unwrap_to_gas_limit(), &config);
	let state = MemoryStackState::new(metadata, &backend);
	let precompile = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompile);

	let code = test.unwrap_to_code();
	let data = test.unwrap_to_data();
	let context = test.unwrap_to_context();
	let mut runtime =
		evm::Runtime::new(code, data, context, config.stack_limit, config.memory_limit);

	let reason = executor.execute(&mut runtime);
	let gas = executor.gas();
	let (values, logs) = executor.into_state().deconstruct();
	backend.apply(values, logs, false);

	if test.0.output.is_none() {
		if verbose_output.verbose {
			print!("{:?} ", reason);
		}

		if reason.is_succeed() {
			failed = true;
			if verbose_output.verbose_failed {
				print!("[Failed: succeed for empty output: {:?}] ", reason);
			}
		}
		if !(test.0.post_state.is_none() && test.0.gas_left.is_none()) {
			failed = true;
			if verbose_output.verbose_failed {
				print!(
					"[Failed: not empty state and left gas for empty output: {:?}] ",
					reason
				);
			}
		}
	} else {
		let expected_post_gas = test.unwrap_to_post_gas();
		if verbose_output.verbose {
			print!("{:?} ", reason);
		}

		if runtime.machine().return_value() != test.unwrap_to_return_value() {
			failed = true;
			if verbose_output.verbose_failed {
				print!(
					"[Failed: wrong return value: {:?}] ",
					runtime.machine().return_value()
				);
			}
		}
		if !test.check_valid_state(backend.state()) {
			failed = true;
			if verbose_output.verbose_failed {
				print!("[Failed: invalid state] ");
			}
		}
		if gas != expected_post_gas {
			failed = true;
			if verbose_output.verbose_failed {
				print!("[Failed: unexpected gas: {:?}] ", gas);
			}
		}
	}

	if failed {
		result.failed += 1;
		if verbose_output.verbose || verbose_output.verbose_failed {
			println!("failed <-------");
		}
	} else if verbose_output.verbose {
		println!("succeed");
	}
	result
}
