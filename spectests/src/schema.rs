use std::{collections::BTreeMap, fmt, str::FromStr};

use evm::uint::{H160, H256, U256};
use serde::Deserialize;

/// Release metadata compiled into the runner and shared with CI.
#[derive(Clone, Debug, Deserialize)]
pub struct ReleaseManifest {
	pub repository: String,
	pub release: String,
	pub asset: String,
	pub sha256: String,
	pub index_root_hash: String,
	pub targets: BTreeMap<String, ReleaseTarget>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReleaseTarget {
	pub path: String,
	pub format: String,
	pub expected_count: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FixtureIndex {
	pub root_hash: String,
	#[serde(default)]
	pub created_at: Option<String>,
	pub test_count: usize,
	#[serde(default)]
	pub test_cases: Vec<FixtureIndexEntry>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FixtureIndexEntry {
	pub id: String,
	pub fixture_hash: String,
	pub fork: String,
	pub format: String,
	pub json_path: String,
}

pub type FixtureFile = BTreeMap<String, StateFixture>;

/// Modern EELS state-test fixture. Unknown fields are deliberately accepted.
#[derive(Clone, Debug, Deserialize)]
pub struct StateFixture {
	pub env: Environment,
	pub pre: BTreeMap<String, AccountState>,
	pub transaction: FixtureTransaction,
	pub post: BTreeMap<String, Vec<PostState>>,
	pub config: FixtureConfig,
	#[serde(rename = "_info", default)]
	pub info: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
	pub current_coinbase: HexValue,
	pub current_gas_limit: HexValue,
	pub current_number: HexValue,
	pub current_timestamp: HexValue,
	pub current_random: HexValue,
	pub current_difficulty: HexValue,
	pub current_base_fee: HexValue,
	pub current_excess_blob_gas: HexValue,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountState {
	pub nonce: HexValue,
	pub balance: HexValue,
	pub code: HexValue,
	#[serde(default)]
	pub storage: BTreeMap<String, HexValue>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureTransaction {
	#[serde(default)]
	pub chain_id: Option<HexValue>,
	pub nonce: HexValue,
	#[serde(default)]
	pub gas_price: Option<HexValue>,
	#[serde(default)]
	pub max_priority_fee_per_gas: Option<HexValue>,
	#[serde(default)]
	pub max_fee_per_gas: Option<HexValue>,
	#[serde(default)]
	pub max_fee_per_blob_gas: Option<HexValue>,
	pub gas_limit: Vec<HexValue>,
	pub to: String,
	pub value: Vec<HexValue>,
	pub data: Vec<HexValue>,
	#[serde(default)]
	pub access_lists: Option<Vec<Vec<AccessListItem>>>,
	#[serde(default)]
	pub blob_versioned_hashes: Option<Vec<HexValue>>,
	pub sender: HexValue,
	#[serde(default)]
	pub secret_key: Option<HexValue>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessListItem {
	pub address: HexValue,
	#[serde(default)]
	pub storage_keys: Vec<HexValue>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PostState {
	pub hash: HexValue,
	pub logs: HexValue,
	pub txbytes: HexValue,
	pub indexes: PostIndexes,
	#[serde(rename = "expectException", default)]
	pub expect_exception: Option<String>,
	#[serde(default)]
	pub state: BTreeMap<String, AccountState>,
	#[serde(default)]
	pub receipt: Option<serde_json::Value>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct PostIndexes {
	pub data: usize,
	pub gas: usize,
	pub value: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FixtureConfig {
	pub chainid: HexValue,
	#[serde(rename = "blobSchedule", default)]
	pub blob_schedule: BTreeMap<String, BlobSchedule>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlobSchedule {
	pub target: HexValue,
	pub max: HexValue,
	pub base_fee_update_fraction: HexValue,
}

/// A fixture hex value retained as text until its semantic validation point.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct HexValue(pub String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HexParseError {
	value: String,
	reason: &'static str,
}

impl fmt::Display for HexParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "invalid hex value {:?}: {}", self.value, self.reason)
	}
}

impl std::error::Error for HexParseError {}

impl HexValue {
	pub fn bytes(&self) -> Result<Vec<u8>, HexParseError> {
		let raw = self
			.0
			.strip_prefix("0x")
			.ok_or_else(|| self.error("missing 0x prefix"))?;
		if raw.len() % 2 != 0 {
			return Err(self.error("odd number of digits"));
		}
		hex::decode(raw).map_err(|_| self.error("non-hex digit"))
	}

	pub fn u256(&self) -> Result<U256, HexParseError> {
		let bytes = self.bytes()?;
		if bytes.len() > 32 {
			return Err(self.error("integer exceeds 256 bits"));
		}
		Ok(U256::from_big_endian(&bytes))
	}

	pub fn h160(&self) -> Result<H160, HexParseError> {
		let bytes = self.bytes()?;
		if bytes.len() != 20 {
			return Err(self.error("address is not 20 bytes"));
		}
		Ok(H160::from_slice(&bytes))
	}

	pub fn h256(&self) -> Result<H256, HexParseError> {
		let bytes = self.bytes()?;
		if bytes.len() > 32 {
			return Err(self.error("word exceeds 32 bytes"));
		}
		let mut padded = [0u8; 32];
		padded[32 - bytes.len()..].copy_from_slice(&bytes);
		Ok(H256::from(padded))
	}

	fn error(&self, reason: &'static str) -> HexParseError {
		HexParseError {
			value: self.0.clone(),
			reason,
		}
	}
}

impl FromStr for HexValue {
	type Err = HexParseError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		let value = Self(value.to_owned());
		value.bytes()?;
		Ok(value)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn keeps_unknown_fixture_metadata() {
		let json = r#"{
			"env":{"currentCoinbase":"0x0000000000000000000000000000000000000000","currentGasLimit":"0x01","currentNumber":"0x01","currentTimestamp":"0x01","currentRandom":"0x00","currentDifficulty":"0x00","currentBaseFee":"0x07","currentExcessBlobGas":"0x00","future":1},
			"pre":{},
			"transaction":{"nonce":"0x00","gasLimit":["0x5208"],"to":"","value":["0x00"],"data":["0x"],"sender":"0x0000000000000000000000000000000000000000","future":true},
			"post":{"Cancun":[{"hash":"0x00","logs":"0x00","txbytes":"0x00","indexes":{"data":0,"gas":0,"value":0},"state":{}}]},
			"config":{"chainid":"0x01","blobSchedule":{"Cancun":{"target":"0x03","max":"0x06","baseFeeUpdateFraction":"0x32f0ed"}},"future":{}},
			"_info":{"fixture-format":"state_test","new-key":"ok"},
			"future_top_level":[]
		}"#;
		let fixture: StateFixture = serde_json::from_str(json).unwrap();
		assert_eq!(fixture.info["new-key"], "ok");
	}

	#[test]
	fn delays_wide_integer_failure_until_conversion() {
		let value: HexValue = serde_json::from_str(
			"\"0x010000000000000000000000000000000000000000000000000000000000000000\"",
		)
		.unwrap();
		assert!(value.u256().is_err());
	}
}
