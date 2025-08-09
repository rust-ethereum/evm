use std::{collections::BTreeMap, str::FromStr};

use evm::interpreter::utils::u256_to_h256;
use hex::FromHex;
use primitive_types::{H160, H256, U256};
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};

/// Statistic type to gather tests pass completion status
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct TestCompletionStatus {
	pub completed: usize,
	pub skipped: usize,
}

impl std::ops::AddAssign for TestCompletionStatus {
	fn add_assign(&mut self, rhs: Self) {
		self.completed += rhs.completed;
		self.skipped += rhs.skipped;
	}
}

impl TestCompletionStatus {
	/// Increment `completed` statistic field
	pub fn inc_completed(&mut self) {
		self.completed += 1
	}

	/// Increment `skipped` statistic field
	pub fn inc_skipped(&mut self) {
		self.skipped += 1
	}

	/// Get total passed tests
	pub fn get_total(&self) -> usize {
		self.completed + self.skipped
	}

	/// Print completion status.
	/// Most useful for single file completion statistic info
	pub fn print_completion(&self) {
		println!("COMPLETED: {} tests", self.completed);
		println!("SKIPPED: {} tests\n", self.skipped);
	}

	/// Print tests pass total statistic info for directory
	pub fn print_total_for_dir(&self, filename: &str) {
		println!(
			"TOTAL tests for: {filename}\n\tCOMPLETED: {}\n\tSKIPPED: {}",
			self.completed, self.skipped
		);
	}

	// Print total statistics info
	pub fn print_total(&self) {
		println!(
			"\nTOTAL: {} tests\n\tCOMPLETED: {}\n\tSKIPPED: {}",
			self.get_total(),
			self.completed,
			self.skipped
		);
	}
}

/// `TestMulti` represents raw data from `jsontest` data file.
/// It contains multiple test data for passing tests.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TestMulti {
	#[serde(rename = "_info")]
	pub info: TestInfo,
	pub env: TestEnv,
	pub post: BTreeMap<Fork, Vec<TestPostState>>,
	pub pre: BTreeMap<H160, TestPreState>,
	pub transaction: TestMultiTransaction,
}

impl TestMulti {
	/// Fill tests data from `TestMulti` data.
	/// Return array of `TestData`, that represent single test,
	/// that ready to pass the test flow.
	pub fn tests(&self) -> Vec<TestData> {
		let mut tests = Vec::new();

		for (fork, post_states) in &self.post {
			for (index, post_state) in post_states.iter().enumerate() {
				tests.push(TestData {
					info: self.info.clone(),
					env: self.env.clone(),
					fork: *fork,
					index,
					post: post_state.clone(),
					pre: self.pre.clone(),
					transaction: TestTransaction {
						data: self.transaction.data[post_state.indexes.data].0.clone(),
						gas_limit: self.transaction.gas_limit[post_state.indexes.gas],
						gas_price: self
							.transaction
							.gas_price
							.unwrap_or(self.env.current_base_fee),
						gas_priority_fee: self.transaction.max_priority_fee_per_gas,
						nonce: self.transaction.nonce,
						secret_key: self.transaction.secret_key,
						sender: self.transaction.sender,
						to: self.transaction.to,
						value: self.transaction.value[post_state.indexes.value],
						access_list: match &self.transaction.access_lists {
							Some(access_lists) => access_lists[post_state.indexes.data].clone().unwrap_or_default(),
							None => Vec::new(),
						},
					},
				});
			}
		}

		tests
	}
}

/// Structure that contains data to run single test
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestData {
	pub info: TestInfo,
	pub env: TestEnv,
	pub fork: Fork,
	pub index: usize,
	pub post: TestPostState,
	pub pre: BTreeMap<H160, TestPreState>,
	pub transaction: TestTransaction,
}

/// `TestInfo` contains information data about test from json file
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestInfo {
	pub comment: String,
	#[serde(rename = "filling-rpc-server")]
	pub filling_rpc_server: String,
	#[serde(rename = "filling-tool-version")]
	pub filling_tool_version: String,
	pub generated_test_hash: String,
	pub lllcversion: String,
	pub solidity: String,
	pub source: String,
	pub source_hash: String,
}

/// `TestEnv` represents Ethereum environment data
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestEnv {
	pub current_base_fee: U256,
	pub current_beacon_root: H256,
	pub current_coinbase: H160,
	pub current_difficulty: U256,
	pub current_gas_limit: U256,
	pub current_number: U256,
	pub current_random: H256,
	pub current_timestamp: U256,
	pub current_withdrawals_root: H256,
	pub previous_hash: H256,
}

/// Available Ethereum forks for testing
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum Fork {
	Berlin,
	Cancun,
	London,
	Merge,
	Shanghai,
	Byzantium,
	Constantinople,
	ConstantinopleFix,
	EIP150,
	EIP158,
	Frontier,
	Homestead,
	Istanbul,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestPostState {
	pub hash: H256,
	pub indexes: TestPostStateIndexes,
	pub logs: H256,
	pub txbytes: HexBytes,
	pub expect_exception: Option<TestExpectException>,
}

/// `TestExpectException` expected Ethereum exception
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[allow(non_camel_case_types, clippy::enum_variant_names)]
pub enum TestExpectException {
	TR_TypeNotSupported,
	TR_IntrinsicGas,
	TR_NonceHasMaxValue,
	TR_NoFundsOrGas,
	TR_NoFunds,
	TR_NoFundsX,
	TR_RLP_WRONGVALUE,
	IntrinsicGas,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TestPostStateIndexes {
	pub data: usize,
	pub gas: usize,
	pub value: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TestPreState {
	pub balance: U256,
	pub code: HexBytes,
	pub nonce: U256,
	#[serde(deserialize_with = "deserialize_storage")]
	pub storage: BTreeMap<H256, H256>,
}

fn deserialize_storage<'de, D>(deserializer: D) -> Result<BTreeMap<H256, H256>, D::Error>
where
	D: Deserializer<'de>,
{
	let m: BTreeMap<U256, U256> = Deserialize::deserialize(deserializer)?;
	Ok(m.into_iter()
		.map(|(k, v)| (u256_to_h256(k), u256_to_h256(v)))
		.collect())
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestMultiTransaction {
	pub data: Vec<HexBytes>,
	pub gas_limit: Vec<U256>,
	pub gas_price: Option<U256>,
	pub max_fee_per_gas: Option<U256>,
	pub max_priority_fee_per_gas: Option<U256>,
	pub nonce: U256,
	pub secret_key: H256,
	pub sender: H160,
	#[serde(serialize_with = "serialize_to", deserialize_with = "deserialize_to")]
	pub to: Option<H160>,
	pub value: Vec<MaybeError<U256>>,
	pub access_lists: Option<Vec<Option<Vec<TestAccessListItem>>>>,
}

fn deserialize_to<'de, D>(deserializer: D) -> Result<Option<H160>, D::Error>
where
	D: Deserializer<'de>,
{
	let data: String = Deserialize::deserialize(deserializer)?;

	if data.is_empty() {
		Ok(None)
	} else {
		Ok(Some(H160::from_str(&data).map_err(de::Error::custom)?))
	}
}

fn serialize_to<S>(value: &Option<H160>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let s = if let Some(v) = value {
		format!("{v:?}")
	} else {
		"".to_string()
	};
	s.serialize(serializer)
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestAccessListItem {
	pub address: H160,
	pub storage_keys: Vec<H256>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestTransaction {
	pub data: Vec<u8>,
	pub gas_limit: U256,
	pub gas_price: U256,
	pub gas_priority_fee: Option<U256>,
	pub nonce: U256,
	pub secret_key: H256,
	pub sender: H160,
	pub to: Option<H160>,
	pub value: MaybeError<U256>,
	pub access_list: Vec<TestAccessListItem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct HexBytes(
	#[serde(
		deserialize_with = "deserialize_hex_bytes",
		serialize_with = "serialize_hex_bytes"
	)]
	pub Vec<u8>,
);

fn deserialize_hex_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let data = String::deserialize(deserializer)?;
	if &data[0..2] != "0x" {
		return Err(de::Error::custom("should start with 0x"));
	}
	FromHex::from_hex(&data[2..]).map_err(de::Error::custom)
}

fn serialize_hex_bytes<S>(value: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let mut s = "0x".to_string();
	s.push_str(&hex::encode(value));
	s.serialize(serializer)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct MaybeError<T: Serialize + de::DeserializeOwned>(
	#[serde(serialize_with = "serialize_maybe_error", deserialize_with = "deserialize_maybe_error")]
	pub Result<T, ()>,
);

fn deserialize_maybe_error<'de, D, T: Deserialize<'de>>(deserializer: D) -> Result<Result<T, ()>, D::Error>
where
	D: Deserializer<'de>
{
	match T::deserialize(deserializer) {
		Ok(value) => Ok(Ok(value)),
		Err(_) => Ok(Err(())),
	}
}

fn serialize_maybe_error<'de, S, T: Serialize>(value: &Result<T, ()>, serializer: S) -> Result<S::Ok, S::Error> where
	S: Serializer,
{
	value.as_ref().map_err(|()| ser::Error::custom("invalid value"))?.serialize(serializer)
}
