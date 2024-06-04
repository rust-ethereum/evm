use std::{collections::BTreeMap, fmt};

use hex::FromHex;
use primitive_types::{H160, H256, U256};
use serde::{
	de::{Error, Visitor},
	Deserialize, Deserializer,
};

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
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
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
						gas_price: self.transaction.gas_price.unwrap_or(U256::zero()),
						nonce: self.transaction.nonce,
						secret_key: self.transaction.secret_key,
						sender: self.transaction.sender,
						to: self.transaction.to,
						value: self.transaction.value[post_state.indexes.value],
						access_list: match &self.transaction.access_lists {
							Some(access_lists) => access_lists[post_state.indexes.data].clone(),
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
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
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
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestPostState {
	pub hash: H256,
	pub indexes: TestPostStateIndexes,
	pub logs: H256,
	pub txbytes: HexBytes,
	pub expect_exception: Option<TestExpectException>,
}

/// `TestExpectException` expected Ethereum exception
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[allow(non_camel_case_types)]
pub enum TestExpectException {
	TR_TypeNotSupported,
	TR_IntrinsicGas,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct TestPostStateIndexes {
	pub data: usize,
	pub gas: usize,
	pub value: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct TestPreState {
	pub balance: U256,
	pub code: HexBytes,
	pub nonce: U256,
	pub storage: BTreeMap<U256, U256>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
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
	pub to: H160,
	pub value: Vec<U256>,
	pub access_lists: Option<Vec<Vec<TestAccessListItem>>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
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
	pub nonce: U256,
	pub secret_key: H256,
	pub sender: H160,
	pub to: H160,
	pub value: U256,
	pub access_list: Vec<TestAccessListItem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct HexBytes(#[serde(deserialize_with = "deserialize_hex_bytes")] pub Vec<u8>);

fn deserialize_hex_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	struct HexStrVisitor;

	impl<'de> Visitor<'de> for HexStrVisitor {
		type Value = Vec<u8>;

		fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
			write!(f, "a hex encoded string")
		}

		fn visit_str<E>(self, data: &str) -> Result<Self::Value, E>
		where
			E: Error,
		{
			if &data[0..2] != "0x" {
				return Err(Error::custom("should start with 0x"));
			}

			FromHex::from_hex(&data[2..]).map_err(Error::custom)
		}

		fn visit_borrowed_str<E>(self, data: &'de str) -> Result<Self::Value, E>
		where
			E: Error,
		{
			if &data[0..2] != "0x" {
				return Err(Error::custom("should start with 0x"));
			}

			FromHex::from_hex(&data[2..]).map_err(Error::custom)
		}
	}

	deserializer.deserialize_str(HexStrVisitor)
}
