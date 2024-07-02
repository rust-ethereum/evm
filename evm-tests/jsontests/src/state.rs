use crate::utils::eip_4844;
use crate::utils::transaction::InvalidTxReason;
use ethjson::hash::Address;
use ethjson::spec::builtin::{AltBn128ConstOperations, AltBn128Pairing, PricingAt};
use ethjson::spec::{ForkSpec, Pricing};
use ethjson::test_helpers::state::PostStateResult;
use ethjson::uint::Uint;
use evm::backend::{ApplyBackend, MemoryAccount, MemoryBackend, MemoryVicinity};
use evm::executor::stack::{
	MemoryStackState, PrecompileFailure, PrecompileFn, PrecompileOutput, StackExecutor,
	StackSubstateMetadata,
};
use evm::utils::U64_MAX;
use evm::{Config, Context, ExitError, ExitReason, ExitSucceed};
use lazy_static::lazy_static;
use libsecp256k1::SecretKey;
use primitive_types::{H160, H256, U256};
use serde::Deserialize;
use sha3::{Digest, Keccak256};
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(Default, Debug, Clone)]
pub struct VerboseOutput {
	pub verbose: bool,
	pub verbose_failed: bool,
	pub very_verbose: bool,
	pub print_state: bool,
}

#[derive(Clone, Debug)]
pub struct FailedTestDetails {
	pub name: String,
	pub spec: ForkSpec,
	pub index: usize,
	pub expected_hash: H256,
	pub actual_hash: H256,
	pub state: BTreeMap<H160, MemoryAccount>,
}

#[derive(Clone, Debug)]
pub struct TestExecutionResult {
	pub total: u64,
	pub failed: u64,
	pub failed_tests: Vec<FailedTestDetails>,
}

impl TestExecutionResult {
	#[allow(clippy::new_without_default)]
	pub const fn new() -> Self {
		Self {
			total: 0,
			failed: 0,
			failed_tests: Vec::new(),
		}
	}

	pub fn merge(&mut self, src: Self) {
		self.failed_tests.extend(src.failed_tests);
		self.total += src.total;
		self.failed += src.failed;
	}
}

#[derive(Deserialize, Debug)]
pub struct Test(ethjson::test_helpers::state::State);

impl Test {
	pub fn unwrap_to_pre_state(&self) -> BTreeMap<H160, MemoryAccount> {
		crate::utils::unwrap_to_state(&self.0.pre_state)
	}

	pub fn unwrap_caller(&self) -> H160 {
		let hash: H256 = self.0.transaction.secret.unwrap().into();
		let mut secret_key = [0; 32];
		secret_key.copy_from_slice(hash.as_bytes());
		let secret = SecretKey::parse(&secret_key);
		let public = libsecp256k1::PublicKey::from_secret_key(&secret.unwrap());
		let mut res = [0u8; 64];
		res.copy_from_slice(&public.serialize()[1..65]);

		H160::from(H256::from_slice(Keccak256::digest(res).as_slice()))
	}

	pub fn unwrap_to_vicinity(
		&self,
		spec: &ForkSpec,
		blob_gas_price: Option<u128>,
	) -> Result<MemoryVicinity, InvalidTxReason> {
		let block_base_fee_per_gas = self.0.env.block_base_fee_per_gas.0;
		let tx = &self.0.transaction;
		// Validation for EIP-1559 that was introduced in London hard fork
		let gas_price = if *spec >= ForkSpec::London {
			tx.gas_price.or(tx.max_fee_per_gas).unwrap_or_default().0
		} else {
			if tx.max_fee_per_gas.is_some() {
				return Err(InvalidTxReason::GasPriseEip1559);
			}
			tx.gas_price.expect("expect gas price").0
		};

		// EIP-1559: priority fee must be lower than gas_price
		if let Some(max_priority_fee_per_gas) = tx.max_priority_fee_per_gas {
			if max_priority_fee_per_gas.0 > gas_price {
				return Err(InvalidTxReason::PriorityFeeTooLarge);
			}
		}
		let effective_gas_price = self.0.transaction.max_priority_fee_per_gas.map_or(
			gas_price,
			|max_priority_fee_per_gas| {
				gas_price.min(max_priority_fee_per_gas.0 + block_base_fee_per_gas)
			},
		);

		// gas price cannot be lower than base fee
		if gas_price < block_base_fee_per_gas {
			return Err(InvalidTxReason::GasPriceLessThenBlockBaseFee);
		}

		let block_randomness = if spec.is_eth2() {
			self.0.env.random.map(|r| {
				// Convert between U256 and H256. U256 is in little-endian but since H256 is just
				// a string-like byte array, it's big endian (MSB is the first element of the array).
				//
				// Byte order here is important because this opcode has the same value as DIFFICULTY
				// (0x44), and so for older forks of Ethereum, the threshold value of 2^64 is used to
				// distinguish between the two: if it's below, the value corresponds to the DIFFICULTY
				// opcode, otherwise to the PREVRANDAO opcode.
				crate::utils::u256_to_h256(r.0)
			})
		} else {
			None
		};
		let blob_hashes = tx.blob_versioned_hashes.clone();

		Ok(MemoryVicinity {
			gas_price,
			effective_gas_price,
			origin: self.unwrap_caller(),
			block_hashes: Vec::new(),
			block_number: self.0.env.number.into(),
			block_coinbase: self.0.env.author.into(),
			block_timestamp: self.0.env.timestamp.into(),
			block_difficulty: self.0.env.difficulty.into(),
			block_gas_limit: self.0.env.gas_limit.into(),
			chain_id: U256::one(),
			block_base_fee_per_gas,
			block_randomness,
			blob_gas_price,
			blob_hashes,
		})
	}
}

lazy_static! {
	static ref ISTANBUL_BUILTINS: BTreeMap<H160, ethcore_builtin::Builtin> = istanbul_builtins();
}

lazy_static! {
	static ref BERLIN_BUILTINS: BTreeMap<H160, ethcore_builtin::Builtin> = berlin_builtins();
}

lazy_static! {
	static ref CANCUN_BUILTINS: BTreeMap<H160, ethcore_builtin::Builtin> = cancun_builtins();
}

macro_rules! precompile_entry {
	($map:expr, $builtins:expr, $index:expr) => {
		let x: PrecompileFn =
			|input: &[u8], gas_limit: Option<u64>, _context: &Context, _is_static: bool| {
				let builtin = $builtins.get(&H160::from_low_u64_be($index)).unwrap();
				Self::exec_as_precompile(builtin, input, gas_limit)
			};
		$map.insert(H160::from_low_u64_be($index), x);
	};
}

pub struct JsonPrecompile;

impl JsonPrecompile {
	pub fn precompile(spec: &ForkSpec) -> Option<BTreeMap<H160, PrecompileFn>> {
		match spec {
			ForkSpec::Istanbul => {
				let mut map = BTreeMap::new();
				precompile_entry!(map, ISTANBUL_BUILTINS, 1);
				precompile_entry!(map, ISTANBUL_BUILTINS, 2);
				precompile_entry!(map, ISTANBUL_BUILTINS, 3);
				precompile_entry!(map, ISTANBUL_BUILTINS, 4);
				precompile_entry!(map, ISTANBUL_BUILTINS, 5);
				precompile_entry!(map, ISTANBUL_BUILTINS, 6);
				precompile_entry!(map, ISTANBUL_BUILTINS, 7);
				precompile_entry!(map, ISTANBUL_BUILTINS, 8);
				precompile_entry!(map, ISTANBUL_BUILTINS, 9);
				Some(map)
			}
			ForkSpec::Berlin => {
				let mut map = BTreeMap::new();
				precompile_entry!(map, BERLIN_BUILTINS, 1);
				precompile_entry!(map, BERLIN_BUILTINS, 2);
				precompile_entry!(map, BERLIN_BUILTINS, 3);
				precompile_entry!(map, BERLIN_BUILTINS, 4);
				precompile_entry!(map, BERLIN_BUILTINS, 5);
				precompile_entry!(map, BERLIN_BUILTINS, 6);
				precompile_entry!(map, BERLIN_BUILTINS, 7);
				precompile_entry!(map, BERLIN_BUILTINS, 8);
				precompile_entry!(map, BERLIN_BUILTINS, 9);
				Some(map)
			}
			// precompiles for London and Berlin are the same
			ForkSpec::London => Self::precompile(&ForkSpec::Berlin),
			// precompiles for Merge and Berlin are the same
			ForkSpec::Merge => Self::precompile(&ForkSpec::Berlin),
			// precompiles for Paris and Berlin are the same
			ForkSpec::Paris => Self::precompile(&ForkSpec::Berlin),
			// precompiles for Shanghai and Berlin are the same
			ForkSpec::Shanghai => Self::precompile(&ForkSpec::Berlin),
			ForkSpec::Cancun => {
				let mut map = BTreeMap::new();
				precompile_entry!(map, CANCUN_BUILTINS, 1);
				precompile_entry!(map, CANCUN_BUILTINS, 2);
				precompile_entry!(map, CANCUN_BUILTINS, 3);
				precompile_entry!(map, CANCUN_BUILTINS, 4);
				precompile_entry!(map, CANCUN_BUILTINS, 5);
				precompile_entry!(map, CANCUN_BUILTINS, 6);
				precompile_entry!(map, CANCUN_BUILTINS, 7);
				precompile_entry!(map, CANCUN_BUILTINS, 8);
				precompile_entry!(map, CANCUN_BUILTINS, 9);
				precompile_entry!(map, CANCUN_BUILTINS, 0xA);
				Some(map)
			}
			_ => None,
		}
	}

	fn exec_as_precompile(
		builtin: &ethcore_builtin::Builtin,
		input: &[u8],
		gas_limit: Option<u64>,
	) -> Result<(PrecompileOutput, u64), PrecompileFailure> {
		let cost = builtin.cost(input, 0);

		if let Some(target_gas) = gas_limit {
			if cost > U64_MAX || target_gas < cost.as_u64() {
				return Err(PrecompileFailure::Error {
					exit_status: ExitError::OutOfGas,
				});
			}
		}

		let mut output = Vec::new();
		match builtin.execute(input, &mut parity_bytes::BytesRef::Flexible(&mut output)) {
			Ok(()) => Ok((
				PrecompileOutput {
					exit_status: ExitSucceed::Stopped,
					output,
				},
				cost.as_u64(),
			)),
			Err(e) => Err(PrecompileFailure::Error {
				exit_status: ExitError::Other(e.into()),
			}),
		}
	}
}

fn istanbul_builtins() -> BTreeMap<H160, ethcore_builtin::Builtin> {
	use ethjson::spec::builtin::{BuiltinCompat, Linear, Modexp, PricingCompat};

	let builtins: BTreeMap<Address, BuiltinCompat> = BTreeMap::from([
		(
			Address(H160::from_low_u64_be(1)),
			BuiltinCompat {
				name: "ecrecover".to_string(),
				pricing: PricingCompat::Single(Pricing::Linear(Linear {
					base: 3000,
					word: 0,
				})),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(2)),
			BuiltinCompat {
				name: "sha256".to_string(),
				pricing: PricingCompat::Single(Pricing::Linear(Linear { base: 60, word: 12 })),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(3)),
			BuiltinCompat {
				name: "ripemd160".to_string(),
				pricing: PricingCompat::Single(Pricing::Linear(Linear {
					base: 600,
					word: 120,
				})),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(4)),
			BuiltinCompat {
				name: "identity".to_string(),
				pricing: PricingCompat::Single(Pricing::Linear(Linear { base: 15, word: 3 })),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(5)),
			BuiltinCompat {
				name: "modexp".to_string(),
				pricing: PricingCompat::Single(Pricing::Modexp(Modexp {
					divisor: 20,
					is_eip_2565: false,
				})),
				activate_at: Some(Uint(U256::zero())),
			},
		),
		(
			Address(H160::from_low_u64_be(6)),
			BuiltinCompat {
				name: "alt_bn128_add".to_string(),
				pricing: PricingCompat::Multi(BTreeMap::from([(
					Uint(U256::zero()),
					PricingAt {
						info: Some("EIP 1108 transition".to_string()),
						price: Pricing::AltBn128ConstOperations(AltBn128ConstOperations {
							price: 150,
						}),
					},
				)])),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(7)),
			BuiltinCompat {
				name: "alt_bn128_mul".to_string(),
				pricing: PricingCompat::Multi(BTreeMap::from([(
					Uint(U256::zero()),
					PricingAt {
						info: Some("EIP 1108 transition".to_string()),
						price: Pricing::AltBn128ConstOperations(AltBn128ConstOperations {
							price: 6000,
						}),
					},
				)])),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(8)),
			BuiltinCompat {
				name: "alt_bn128_pairing".to_string(),
				pricing: PricingCompat::Multi(BTreeMap::from([(
					Uint(U256::zero()),
					PricingAt {
						info: Some("EIP 1108 transition".to_string()),
						price: Pricing::AltBn128Pairing(AltBn128Pairing {
							base: 45000,
							pair: 34000,
						}),
					},
				)])),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(9)),
			BuiltinCompat {
				name: "blake2_f".to_string(),
				pricing: PricingCompat::Single(Pricing::Blake2F { gas_per_round: 1 }),
				activate_at: Some(Uint(U256::zero())),
			},
		),
	]);
	builtins
		.into_iter()
		.map(|(address, builtin)| {
			(
				address.into(),
				ethjson::spec::Builtin::from(builtin).try_into().unwrap(),
			)
		})
		.collect()
}

fn berlin_builtins() -> BTreeMap<H160, ethcore_builtin::Builtin> {
	use ethjson::spec::builtin::{BuiltinCompat, Linear, Modexp, PricingCompat};

	let builtins: BTreeMap<Address, BuiltinCompat> = BTreeMap::from([
		(
			Address(H160::from_low_u64_be(1)),
			BuiltinCompat {
				name: "ecrecover".to_string(),
				pricing: PricingCompat::Single(Pricing::Linear(Linear {
					base: 3000,
					word: 0,
				})),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(2)),
			BuiltinCompat {
				name: "sha256".to_string(),
				pricing: PricingCompat::Single(Pricing::Linear(Linear { base: 60, word: 12 })),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(3)),
			BuiltinCompat {
				name: "ripemd160".to_string(),
				pricing: PricingCompat::Single(Pricing::Linear(Linear {
					base: 600,
					word: 120,
				})),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(4)),
			BuiltinCompat {
				name: "identity".to_string(),
				pricing: PricingCompat::Single(Pricing::Linear(Linear { base: 15, word: 3 })),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(5)),
			BuiltinCompat {
				name: "modexp".to_string(),
				pricing: PricingCompat::Single(Pricing::Modexp(Modexp {
					divisor: 3,
					is_eip_2565: true,
				})),
				activate_at: Some(Uint(U256::zero())),
			},
		),
		(
			Address(H160::from_low_u64_be(6)),
			BuiltinCompat {
				name: "alt_bn128_add".to_string(),
				pricing: PricingCompat::Multi(BTreeMap::from([(
					Uint(U256::zero()),
					PricingAt {
						info: Some("EIP 1108 transition".to_string()),
						price: Pricing::AltBn128ConstOperations(AltBn128ConstOperations {
							price: 150,
						}),
					},
				)])),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(7)),
			BuiltinCompat {
				name: "alt_bn128_mul".to_string(),
				pricing: PricingCompat::Multi(BTreeMap::from([(
					Uint(U256::zero()),
					PricingAt {
						info: Some("EIP 1108 transition".to_string()),
						price: Pricing::AltBn128ConstOperations(AltBn128ConstOperations {
							price: 6000,
						}),
					},
				)])),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(8)),
			BuiltinCompat {
				name: "alt_bn128_pairing".to_string(),
				pricing: PricingCompat::Multi(BTreeMap::from([(
					Uint(U256::zero()),
					PricingAt {
						info: Some("EIP 1108 transition".to_string()),
						price: Pricing::AltBn128Pairing(AltBn128Pairing {
							base: 45000,
							pair: 34000,
						}),
					},
				)])),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(9)),
			BuiltinCompat {
				name: "blake2_f".to_string(),
				pricing: PricingCompat::Single(Pricing::Blake2F { gas_per_round: 1 }),
				activate_at: Some(Uint(U256::zero())),
			},
		),
	]);
	builtins
		.into_iter()
		.map(|(address, builtin)| {
			(
				address.into(),
				ethjson::spec::Builtin::from(builtin).try_into().unwrap(),
			)
		})
		.collect()
}

fn cancun_builtins() -> BTreeMap<H160, ethcore_builtin::Builtin> {
	use ethjson::spec::builtin::{BuiltinCompat, Linear, Modexp, PricingCompat};

	let builtins: BTreeMap<Address, BuiltinCompat> = BTreeMap::from([
		(
			Address(H160::from_low_u64_be(1)),
			BuiltinCompat {
				name: "ecrecover".to_string(),
				pricing: PricingCompat::Single(Pricing::Linear(Linear {
					base: 3000,
					word: 0,
				})),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(2)),
			BuiltinCompat {
				name: "sha256".to_string(),
				pricing: PricingCompat::Single(Pricing::Linear(Linear { base: 60, word: 12 })),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(3)),
			BuiltinCompat {
				name: "ripemd160".to_string(),
				pricing: PricingCompat::Single(Pricing::Linear(Linear {
					base: 600,
					word: 120,
				})),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(4)),
			BuiltinCompat {
				name: "identity".to_string(),
				pricing: PricingCompat::Single(Pricing::Linear(Linear { base: 15, word: 3 })),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(5)),
			BuiltinCompat {
				name: "modexp".to_string(),
				pricing: PricingCompat::Single(Pricing::Modexp(Modexp {
					divisor: 3,
					is_eip_2565: true,
				})),
				activate_at: Some(Uint(U256::zero())),
			},
		),
		(
			Address(H160::from_low_u64_be(6)),
			BuiltinCompat {
				name: "alt_bn128_add".to_string(),
				pricing: PricingCompat::Multi(BTreeMap::from([(
					Uint(U256::zero()),
					PricingAt {
						info: Some("EIP 1108 transition".to_string()),
						price: Pricing::AltBn128ConstOperations(AltBn128ConstOperations {
							price: 150,
						}),
					},
				)])),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(7)),
			BuiltinCompat {
				name: "alt_bn128_mul".to_string(),
				pricing: PricingCompat::Multi(BTreeMap::from([(
					Uint(U256::zero()),
					PricingAt {
						info: Some("EIP 1108 transition".to_string()),
						price: Pricing::AltBn128ConstOperations(AltBn128ConstOperations {
							price: 6000,
						}),
					},
				)])),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(8)),
			BuiltinCompat {
				name: "alt_bn128_pairing".to_string(),
				pricing: PricingCompat::Multi(BTreeMap::from([(
					Uint(U256::zero()),
					PricingAt {
						info: Some("EIP 1108 transition".to_string()),
						price: Pricing::AltBn128Pairing(AltBn128Pairing {
							base: 45000,
							pair: 34000,
						}),
					},
				)])),
				activate_at: None,
			},
		),
		(
			Address(H160::from_low_u64_be(9)),
			BuiltinCompat {
				name: "blake2_f".to_string(),
				pricing: PricingCompat::Single(Pricing::Blake2F { gas_per_round: 1 }),
				activate_at: Some(Uint(U256::zero())),
			},
		),
		(
			Address(H160::from_low_u64_be(0xA)),
			BuiltinCompat {
				name: "kzg".to_string(),
				pricing: PricingCompat::Single(Pricing::Linear(Linear {
					base: 50_000,
					word: 0,
				})),
				activate_at: None,
			},
		),
	]);
	builtins
		.into_iter()
		.map(|(address, builtin)| {
			(
				address.into(),
				ethjson::spec::Builtin::from(builtin).try_into().unwrap(),
			)
		})
		.collect()
}

pub fn test(
	verbose_output: VerboseOutput,
	name: &str,
	test: Test,
	specific_spec: Option<ForkSpec>,
) -> TestExecutionResult {
	use std::thread;

	const STACK_SIZE: usize = 16 * 1024 * 1024;

	let name = name.to_string();
	// Spawn thread with explicit stack size
	let child = thread::Builder::new()
		.stack_size(STACK_SIZE)
		.spawn(move || test_run(&verbose_output, &name, test, specific_spec))
		.unwrap();

	// Wait for thread to join
	child.join().unwrap()
}

/// Validate EIP-3607 - empty create caller
fn assert_empty_create_caller(expect_exception: &Option<String>, name: &str) {
	let exception = expect_exception
		.as_deref()
		.expect("expected evm-json-test exception");
	let check_exception = exception == "SenderNotEOA";
	assert!(
		check_exception,
		"expected EmptyCaller exception for test: {name}"
	);
}

/// Check call expected exception
fn assert_call_exit_exception(expect_exception: &Option<String>) {
	assert!(
		expect_exception.is_none(),
		"unexpected call exception: {expect_exception:?}"
	);
}

/// Check Exit Reason of EVM execution
fn check_create_exit_reason(
	reason: &ExitReason,
	expect_exception: &Option<String>,
	name: &str,
) -> bool {
	match reason {
		ExitReason::Error(err) => {
			if let Some(exception) = expect_exception.as_deref() {
				match err {
					ExitError::CreateContractLimit => {
						let check_result = exception == "TR_InitCodeLimitExceeded"
							|| exception == "TransactionException.INITCODE_SIZE_EXCEEDED";
						assert!(
							check_result,
							"unexpected exception {exception:?} for CreateContractLimit error for test: {name}"
						);
						return true;
					}
					ExitError::MaxNonce => {
						let check_result = exception == "TR_NonceHasMaxValue";
						assert!(check_result,
								"unexpected exception {exception:?} for MaxNonce error for test: {name}"
						);
						return true;
					}
					_ => {
						panic!("unexpected error: {err:?} for exception: {exception}")
					}
				}
			} else {
				return false;
			}
		}
		ExitReason::Fatal(err) => {
			panic!("Unexpected error: {err:?}")
		}
		_ => {
			assert!(
				expect_exception.is_none(),
				"Unexpected json-test error: {expect_exception:?}"
			);
		}
	}
	false
}

/// Assert vicinity validation to ensure that test os expected validation error
#[allow(clippy::cognitive_complexity)]
fn assert_vicinity_validation(
	reason: &InvalidTxReason,
	states: &[PostStateResult],
	spec: &ForkSpec,
	name: &str,
) {
	match *spec {
		ForkSpec::Istanbul | ForkSpec::Berlin => match reason {
			InvalidTxReason::GasPriseEip1559 => {
				for (i, state) in states.iter().enumerate() {
					let expected = state
						.expect_exception
						.as_deref()
						.expect("expected error message for test: [{spec}] {name}:{i}");
					let is_checked = expected == "TR_TypeNotSupported";
					assert!(
						is_checked,
						"unexpected error message {expected:?} for: [{spec:?}] {name}:{i}",
					);
				}
			}
			_ => panic!("Unexpected validation reason: {reason:?} [{name}]"),
		},
		ForkSpec::London => {
			match reason {
				InvalidTxReason::PriorityFeeTooLarge => {
					for (i, state) in states.iter().enumerate() {
						let expected = state.expect_exception.as_deref().expect(
							"expected error message for test: {reason:?} [{spec}] {name}:{i}",
						);
						let is_checked = expected == "tipTooHigh" || expected == "TR_TipGtFeeCap";
						assert!(
							is_checked,
							"unexpected error message {expected:?} for: {reason:?} [{spec:?}] {name}:{i}",
						);
					}
				}
				InvalidTxReason::GasPriceLessThenBlockBaseFee => {
					for (i, state) in states.iter().enumerate() {
						let expected = state.expect_exception.as_deref().expect(
							"expected error message for test: {reason:?} [{spec}] {name}:{i}",
						);
						let is_checked =
							expected == "lowFeeCap" || expected == "TR_FeeCapLessThanBlocks";
						assert!(
							is_checked,
							"unexpected error message {expected:?} for: {reason:?} [{spec:?}] {name}:{i}",
						);
					}
				}
				_ => panic!("Unexpected validation reason: {reason:?} [{spec:?}] {name}"),
			}
		}
		ForkSpec::Paris => {
			match reason {
				InvalidTxReason::PriorityFeeTooLarge => {
					for (i, state) in states.iter().enumerate() {
						let expected = state.expect_exception.as_deref().expect(
							"expected error message for test: {reason:?} [{spec}] {name}:{i}",
						);
						let is_checked = expected == "TR_TipGtFeeCap";
						assert!(
							is_checked,
							"unexpected error message {expected:?} for: {reason:?} [{spec:?}] {name}:{i}",
						);
					}
				}
				InvalidTxReason::GasPriceLessThenBlockBaseFee => {
					for (i, state) in states.iter().enumerate() {
						let expected = state.expect_exception.as_deref().expect(
							"expected error message for test: {reason:?} [{spec}] {name}:{i}",
						);
						let is_checked = expected == "TR_FeeCapLessThanBlocks";
						assert!(
							is_checked,
							"unexpected error message {expected:?} for: {reason:?} [{spec:?}] {name}:{i}",
						);
					}
				}
				_ => panic!("Unexpected validation reason: {reason:?} [{spec:?}] {name}"),
			}
		}
		ForkSpec::Shanghai => {
			match reason {
				InvalidTxReason::PriorityFeeTooLarge => {
					for (i, state) in states.iter().enumerate() {
						let expected = state.expect_exception.as_deref().expect(
							"expected error message for test: {reason:?} [{spec}] {name}:{i}",
						);
						let is_checked = expected == "TR_TipGtFeeCap";
						assert!(
							is_checked,
							"unexpected error message {expected:?} for: {reason:?} [{spec:?}] {name}:{i}",
						);
					}
				}
				InvalidTxReason::GasPriceLessThenBlockBaseFee => {
					for (i, state) in states.iter().enumerate() {
						let expected = state.expect_exception.as_deref().expect(
							"expected error message for test: {reason:?} [{spec}] {name}:{i}",
						);
						let is_checked = expected == "TR_FeeCapLessThanBlocks";
						assert!(
							is_checked,
							"unexpected error message {expected:?} for: {reason:?} [{spec:?}] {name}:{i}",
						);
					}
				}
				_ => panic!("Unexpected validation reason: {reason:?} [{spec:?}] {name}"),
			}
		}
		ForkSpec::Cancun => {
			match reason {
				InvalidTxReason::PriorityFeeTooLarge => {
					for (i, state) in states.iter().enumerate() {
						let expected = state.expect_exception.as_deref().expect(
							"expected error message for test: {reason:?} [{spec}] {name}:{i}",
						);
						let is_checked = expected == "TR_TipGtFeeCap";
						assert!(
							is_checked,
							"unexpected error message {expected:?} for: {reason:?} [{spec:?}] {name}:{i}",
						);
					}
				}
				InvalidTxReason::GasPriceLessThenBlockBaseFee => {
					for (i, state) in states.iter().enumerate() {
						let expected = state.expect_exception.as_deref().expect(
							"expected error message for test: {reason:?} [{spec}] {name}:{i}",
						);
						let is_checked = expected == "TR_FeeCapLessThanBlocks"
							|| expected == "TransactionException.INSUFFICIENT_MAX_FEE_PER_GAS";
						assert!(
							is_checked,
							"unexpected error message {expected:?} for: {reason:?} [{spec:?}] {name}:{i}",
						);
					}
				}
				_ => panic!("Unexpected validation reason: {reason:?} [{spec:?}] {name}"),
			}
		}
		_ => panic!("Unexpected validation reason: {reason:?} [{spec:?}] {name}"),
	}
}

/// Check Exit Reason of EVM execution
fn check_validate_exit_reason(
	reason: &InvalidTxReason,
	expect_exception: &Option<String>,
	name: &str,
	spec: &ForkSpec,
) -> bool {
	expect_exception.as_deref().map_or_else(
		|| {
			panic!("unexpected validation error reason: {reason:?}");
		},
		|exception| {
			match reason {
				InvalidTxReason::OutOfFund => {
					let check_result = exception
						== "TransactionException.INSUFFICIENT_ACCOUNT_FUNDS"
						|| exception == "TR_NoFunds"
						|| exception == "TR_NoFundsX"
						|| exception == "TransactionException.INSUFFICIENT_MAX_FEE_PER_BLOB_GAS";
					assert!(
						check_result,
						"unexpected exception {exception:?} for OutOfFund for test: [{spec:?}] {name}"
					);
				}
				InvalidTxReason::GasLimitReached => {
					let check_result = exception == "TR_GasLimitReached";
					assert!(
						check_result,
						"unexpected exception {exception:?} for GasLimitReached for test: [{spec:?}] {name}"
					);
				}
				InvalidTxReason::IntrinsicGas => {
					let check_result = exception == "TR_NoFundsOrGas"
						|| exception == "TR_IntrinsicGas"
						|| exception == "TransactionException.INTRINSIC_GAS_TOO_LOW"
						|| exception == "IntrinsicGas";
					assert!(
						check_result,
						"unexpected exception {exception:?} for IntrinsicGas for test: [{spec:?}] {name}"
					);
				}
				InvalidTxReason::BlobVersionNotSupported => {
					let check_result = exception
						== "TransactionException.TYPE_3_TX_INVALID_BLOB_VERSIONED_HASH"
						|| exception == "TR_BLOBVERSION_INVALID";
					assert!(
						check_result,
						"unexpected exception {exception:?} for BlobVersionNotSupported for test: [{spec:?}] {name}"
					);
				}
				InvalidTxReason::BlobCreateTransaction => {
					let check_result = exception == "TR_BLOBCREATE";
					assert!(
						check_result,
						"unexpected exception {exception:?} for BlobCreateTransaction for test: [{spec:?}] {name}"
					);
				}
				InvalidTxReason::BlobGasPriceGreaterThanMax => {
					let check_result =
						exception == "TransactionException.INSUFFICIENT_MAX_FEE_PER_BLOB_GAS";
					assert!(
						check_result,
						"unexpected exception {exception:?} for BlobGasPriceGreaterThanMax for test: [{spec:?}] {name}"
					);
				}
				InvalidTxReason::TooManyBlobs => {
					let check_result = exception == "TR_BLOBLIST_OVERSIZE"
						|| exception == "TransactionException.TYPE_3_TX_BLOB_COUNT_EXCEEDED";
					assert!(
						check_result,
						"unexpected exception {exception:?} for TooManyBlobs for test: [{spec:?}] {name}"
					);
				}
				InvalidTxReason::EmptyBlobs => {
					let check_result = exception == "TransactionException.TYPE_3_TX_ZERO_BLOBS"
						|| exception == "TR_EMPTYBLOB";
					assert!(
						check_result,
						"unexpected exception {exception:?} for EmptyBlobs for test: [{spec:?}] {name}"
					);
				}
				InvalidTxReason::MaxFeePerBlobGasNotSupported => {
					let check_result =
						exception == "TransactionException.TYPE_3_TX_PRE_FORK|TransactionException.TYPE_3_TX_ZERO_BLOBS";
					assert!(
						check_result,
						"unexpected exception {exception:?} for MaxFeePerBlobGasNotSupported for test: [{spec:?}] {name}"
					);
				}
				InvalidTxReason::BlobVersionedHashesNotSupported => {
					let check_result = exception == "TransactionException.TYPE_3_TX_PRE_FORK";
					assert!(
						check_result,
						"unexpected exception {exception:?} for BlobVersionedHashesNotSupported for test: [{spec:?}] {name}"
					);
				}
				_ => {
					panic!(
						"unexpected exception {exception:?} for reason {reason:?} for test: [{spec:?}] {name}"
					);
				}
			}
			true
		},
	)
}

#[allow(clippy::cognitive_complexity)]
fn test_run(
	verbose_output: &VerboseOutput,
	name: &str,
	test: Test,
	specific_spec: Option<ForkSpec>,
) -> TestExecutionResult {
	let mut tests_result = TestExecutionResult::new();
	let test_tx = &test.0.transaction;
	for (spec, states) in &test.0.post_states {
		// Run tests for specific SPEC (Hard fork)
		if let Some(s) = specific_spec.as_ref() {
			if s != spec {
				continue;
			}
		}
		let (gasometer_config, delete_empty) = match spec {
			ForkSpec::Istanbul => (Config::istanbul(), true),
			ForkSpec::Berlin => (Config::berlin(), true),
			ForkSpec::London => (Config::london(), true),
			ForkSpec::Merge => (Config::merge(), true),
			ForkSpec::Paris => (Config::merge(), true),
			ForkSpec::Shanghai => (Config::shanghai(), true),
			ForkSpec::Cancun => (Config::cancun(), true),
			_ => {
				continue;
			}
		};

		// EIP-4844
		let blob_gas_price =
			if let Some(current_excess_blob_gas) = test.0.env.current_excess_blob_gas {
				Some(eip_4844::calc_blob_gas_price(
					current_excess_blob_gas.0.as_u64(),
				))
			} else if let (Some(parent_blob_gas_used), Some(parent_excess_blob_gas)) = (
				test.0.env.parent_blob_gas_used,
				test.0.env.parent_excess_blob_gas,
			) {
				let excess_blob_gas = eip_4844::calc_excess_blob_gas(
					parent_blob_gas_used.0.as_u64(),
					parent_excess_blob_gas.0.as_u64(),
				);
				Some(eip_4844::calc_blob_gas_price(excess_blob_gas))
			} else {
				None
			};
		// EIP-4844
		let data_max_fee = if gasometer_config.has_shard_blob_transactions {
			let max_fee_per_blob_gas = test_tx.max_fee_per_blob_gas.unwrap_or_default().0;
			Some(eip_4844::calc_max_data_fee(
				max_fee_per_blob_gas,
				test_tx.blob_versioned_hashes.len(),
			))
		} else {
			None
		};
		let data_fee = if gasometer_config.has_shard_blob_transactions {
			Some(eip_4844::calc_data_fee(
				blob_gas_price.expect("expect blob_gas_price"),
				test_tx.blob_versioned_hashes.len(),
			))
		} else {
			None
		};

		let original_state = test.unwrap_to_pre_state();
		let vicinity = test.unwrap_to_vicinity(spec, blob_gas_price);
		if let Err(tx_err) = vicinity {
			tests_result.total += states.len() as u64;
			let h = states.first().unwrap().hash.0;
			// if vicinity could not be computed then the transaction was invalid so we simply
			// check the original state and move on
			let (is_valid_hash, actual_hash) = crate::utils::check_valid_hash(&h, &original_state);
			if !is_valid_hash {
				tests_result.failed_tests.push(FailedTestDetails {
					expected_hash: h,
					actual_hash,
					index: 0,
					name: String::from_str(name).unwrap(),
					spec: spec.clone(),
					state: original_state,
				});

				if verbose_output.verbose_failed {
					println!(" [{spec:?}] {name}: {tx_err:?} ... validation failed\t<----");
				}
				tests_result.failed += 1;
				continue;
			}
			assert_vicinity_validation(&tx_err, states, spec, name);
			// As it's expected validation error - skip the test run
			continue;
		}
		let vicinity = vicinity.unwrap();
		let caller = test.unwrap_caller();
		let caller_balance = original_state
			.get(&caller)
			.map_or_else(U256::zero, |acc| acc.balance);
		// EIP-3607
		let caller_code = original_state
			.get(&caller)
			.map_or_else(Vec::new, |acc| acc.code.clone());

		for (i, state) in states.iter().enumerate() {
			let transaction = test_tx.select(&state.indexes);
			let mut backend = MemoryBackend::new(&vicinity, original_state.clone());
			tests_result.total += 1;
			// Test case may be expected to fail with an unsupported tx type if the current fork is
			// older than Berlin (see EIP-2718). However, this is not implemented in sputnik itself and rather
			// in the code hosting sputnik. https://github.com/rust-blockchain/evm/pull/40
			let expect_tx_type_not_supported =
				matches!(
					spec,
					ForkSpec::EIP150
						| ForkSpec::EIP158 | ForkSpec::Frontier
						| ForkSpec::Homestead | ForkSpec::Byzantium
						| ForkSpec::Constantinople
						| ForkSpec::ConstantinopleFix
						| ForkSpec::Istanbul | ForkSpec::Berlin
				) && TxType::from_txbytes(&state.txbytes) != TxType::Legacy
					&& state.expect_exception.as_deref() == Some("TR_TypeNotSupported");
			if expect_tx_type_not_supported {
				continue;
			}

			let gas_limit: u64 = transaction.gas_limit.into();
			let data: Vec<u8> = transaction.data.clone().into();

			let valid_tx = crate::utils::transaction::validate(
				&transaction,
				test.0.env.gas_limit.0,
				caller_balance,
				&gasometer_config,
				test_tx,
				&vicinity,
				blob_gas_price,
				data_max_fee,
				spec,
			);
			if let Err(err) = &valid_tx {
				if check_validate_exit_reason(err, &state.expect_exception, name, spec) {
					continue;
				}
			}

			// We do not check overflow after TX validation
			let total_fee = if let Some(data_fee) = data_fee {
				vicinity.effective_gas_price * gas_limit + data_fee
			} else {
				vicinity.effective_gas_price * gas_limit
			};

			// Only execute valid transactions
			if valid_tx.is_ok() {
				let metadata =
					StackSubstateMetadata::new(transaction.gas_limit.into(), &gasometer_config);
				let executor_state = MemoryStackState::new(metadata, &backend);
				let precompile = JsonPrecompile::precompile(spec).unwrap();
				let mut executor = StackExecutor::new_with_precompiles(
					executor_state,
					&gasometer_config,
					&precompile,
				);
				executor.state_mut().withdraw(caller, total_fee).unwrap();

				let access_list = transaction
					.access_list
					.into_iter()
					.map(|(address, keys)| (address.0, keys.into_iter().map(|k| k.0).collect()))
					.collect();

				// EIP-3607: Reject transactions from senders with deployed code
				if caller_code.is_empty() {
					match transaction.to {
						ethjson::maybe::MaybeEmpty::Some(to) => {
							let value = transaction.value.into();

							// Exit reason for Call do not analyzed as it mostly do not expect exceptions
							let _reason = executor.transact_call(
								caller,
								to.into(),
								value,
								data,
								gas_limit,
								access_list,
							);
							assert_call_exit_exception(&state.expect_exception);
						}
						ethjson::maybe::MaybeEmpty::None => {
							let code = data;
							let value = transaction.value.into();

							let reason = executor.transact_create(
								caller,
								value,
								code,
								gas_limit,
								access_list,
							);
							if check_create_exit_reason(
								&reason.0,
								&state.expect_exception,
								&format!("{spec:?}-{name}-{i}"),
							) {
								continue;
							}
						}
					}
				} else {
					assert_empty_create_caller(&state.expect_exception, name);
				}

				if verbose_output.print_state {
					println!(
						"gas_limit: {gas_limit}\nused_gas: {:?}",
						executor.used_gas()
					);
				}

				let actual_fee = executor.fee(vicinity.effective_gas_price);
				// Forks after London burn miner rewards and thus have different gas fee
				// calculation (see EIP-1559)
				let miner_reward = if spec.is_eth2() {
					let coinbase_gas_price = vicinity
						.effective_gas_price
						.saturating_sub(vicinity.block_base_fee_per_gas);
					executor.fee(coinbase_gas_price)
				} else {
					actual_fee
				};

				executor
					.state_mut()
					.deposit(vicinity.block_coinbase, miner_reward);

				let amount_to_return_for_caller = data_fee.map_or_else(
					|| total_fee - actual_fee,
					|data_fee| total_fee - actual_fee - data_fee,
				);
				executor
					.state_mut()
					.deposit(caller, amount_to_return_for_caller);

				let (values, logs) = executor.into_state().deconstruct();

				backend.apply(values, logs, delete_empty);
				// It's special case for hard forks: London or before London
				// According to EIP-160 empty account should be removed. But in that particular test - original test state
				// contains account 0x03 (it's precompile), and when precompile 0x03 was called it exit with
				// OutOfGas result. And after exit of substate account not marked as touched, as exit reason
				// is not success. And it mean, that it don't appeared in Apply::Modify, then as untouched it
				// can't be removed by backend.apply event. In that particular case we should manage it manually.
				// NOTE: it's not realistic situation for real life flow.
				if *spec <= ForkSpec::London && delete_empty && name == "failed_tx_xcf416c53" {
					let state = backend.state_mut();
					state.retain(|addr, account| {
						// Check is account empty for precompile 0x03
						!(addr == &H160::from_low_u64_be(3)
							&& account.balance == U256::zero()
							&& account.nonce == U256::zero()
							&& account.code.is_empty())
					});
				}
			} else {
				if let Some(e) = state.expect_exception.as_ref() {
					panic!("unexpected exception: {e} for test {name}-{i}");
				}
				panic!("unexpected validation for test {name}-{i}")
			}
			let (is_valid_hash, actual_hash) =
				crate::utils::check_valid_hash(&state.hash.0, backend.state());
			if !is_valid_hash {
				let failed_res = FailedTestDetails {
					expected_hash: state.hash.0,
					actual_hash,
					index: i,
					name: String::from_str(name).unwrap(),
					spec: spec.clone(),
					state: backend.state().clone(),
				};
				tests_result.failed_tests.push(failed_res);
				tests_result.failed += 1;

				if verbose_output.verbose_failed {
					println!(" [{spec:?}] {name}:{i} ... failed\t<----");
				}
				if verbose_output.print_state {
					// Print detailed state data
					println!(
						"expected_hash:\t{:?}\nactual_hash:\t{actual_hash:?}",
						state.hash.0,
					);
					for (addr, acc) in backend.state().clone() {
						// Decode balance
						let mut write_buf = [0u8; 32];
						acc.balance.to_big_endian(&mut write_buf[..]);
						let balance = acc.balance.to_string();

						println!(
                            "{addr:?}: {{\n    balance: {balance}\n    code: {:?}\n    nonce: {}\n    storage: {:#?}\n}}",
                            hex::encode(acc.code),
                            acc.nonce,
                            acc.storage
                        );
					}
					if let Some(e) = state.expect_exception.as_ref() {
						println!("-> expect_exception: {e}");
					}
				}
			} else if verbose_output.very_verbose && !verbose_output.verbose_failed {
				println!(" [{spec:?}]  {name}:{i} ... passed");
			}
		}
	}
	tests_result
}

/// Denotes the type of transaction.
#[derive(Debug, PartialEq)]
enum TxType {
	/// All transactions before EIP-2718 are legacy.
	Legacy,
	/// https://eips.ethereum.org/EIPS/eip-2718
	AccessList,
	/// https://eips.ethereum.org/EIPS/eip-1559
	DynamicFee,
	/// https://eips.ethereum.org/EIPS/eip-4844
	ShardBlob,
}

impl TxType {
	/// Whether this is a legacy, access list, dynamic fee, etc transaction
	// Taken from geth's core/types/transaction.go/UnmarshalBinary, but we only detect the transaction
	// type rather than unmarshal the entire payload.
	const fn from_txbytes(txbytes: &[u8]) -> Self {
		match txbytes[0] {
			b if b > 0x7f => Self::Legacy,
			1 => Self::AccessList,
			2 => Self::DynamicFee,
			3 => Self::ShardBlob,
			_ => panic!(
				"Unknown tx type. \
You may need to update the TxType enum if Ethereum introduced new enveloped transaction types."
			),
		}
	}
}
