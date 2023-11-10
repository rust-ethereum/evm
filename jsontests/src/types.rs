use hex::FromHex;
use primitive_types::{H160, H256, U256};
use serde::{
	de::{Error, Visitor},
	Deserialize, Deserializer,
};
use std::collections::BTreeMap;
use std::fmt;

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
	pub fn tests(&self) -> Vec<Test> {
		let mut tests = Vec::new();

		for (fork, post_states) in &self.post {
			for (index, post_state) in post_states.iter().enumerate() {
				tests.push(Test {
					info: self.info.clone(),
                    env: self.env.clone(),
					fork: fork.clone(),
					index,
					post: post_state.clone(),
					pre: self.pre.clone(),
					transaction: TestTransaction {
						data: self.transaction.data[post_state.indexes.data].0.clone(),
						gas_limit: self.transaction.gas_limit[post_state.indexes.gas],
						gas_price: self.transaction.gas_price,
						nonce: self.transaction.nonce,
						secret_key: self.transaction.secret_key,
						sender: self.transaction.sender,
						to: self.transaction.to,
						value: self.transaction.value[post_state.indexes.value],
					},
				});
			}
		}

		tests
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Test {
	pub info: TestInfo,
    pub env: TestEnv,
	pub fork: Fork,
	pub index: usize,
	pub post: TestPostState,
	pub pre: BTreeMap<H160, TestPreState>,
	pub transaction: TestTransaction,
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
pub enum Fork {
	Berlin,
	Cancun,
	London,
	Merge,
	Shanghai,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct TestPostState {
	pub hash: H256,
	pub indexes: TestPostStateIndexes,
	pub logs: H256,
	pub txbytes: HexBytes,
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
	pub gas_price: U256,
	pub nonce: U256,
	pub secret_key: H256,
	pub sender: H160,
	pub to: H160,
	pub value: Vec<U256>,
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
