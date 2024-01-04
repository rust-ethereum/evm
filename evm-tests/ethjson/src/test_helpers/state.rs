// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! State test deserialization.

/// Type for running `State` tests
pub type Test = super::tester::GenericTester<String, State>;

use crate::{
	bytes::Bytes,
	hash::{Address, H256},
	maybe::MaybeEmpty,
	spec::{ForkSpec, State as AccountState},
	transaction::Transaction,
	uint::Uint,
	vm::Env,
};
use ethereum_types::U256;
use serde::Deserialize;
use std::collections::BTreeMap;

/// State test deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct State {
	/// Environment.
	pub env: Env,
	/// Pre state.
	#[serde(rename = "pre")]
	pub pre_state: AccountState,
	/// Post state.
	#[serde(rename = "post")]
	pub post_states: BTreeMap<ForkSpec, Vec<PostStateResult>>,
	/// Transaction.
	pub transaction: MultiTransaction,
}

/// State test transaction deserialization.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiTransaction {
	/// Transaction data set.
	pub data: Vec<Bytes>,
	/// Access lists (see EIP-2930)
	#[serde(default)]
	pub access_lists: Vec<Option<AccessList>>,
	/// Gas limit set.
	pub gas_limit: Vec<Uint>,
	/// Gas price.
	#[serde(default)]
	pub gas_price: Uint,
	/// for details on `maxFeePerGas` see EIP-1559
	#[serde(default)]
	pub max_fee_per_gas: Uint,
	/// for details on `maxPriorityFeePerGas` see EIP-1559
	#[serde(default)]
	pub max_priority_fee_per_gas: Uint,
	/// Nonce.
	pub nonce: Uint,
	/// Secret key.
	#[serde(rename = "secretKey")]
	pub secret: Option<H256>,
	/// To.
	pub to: MaybeEmpty<Address>,
	/// Value set.
	pub value: Vec<Uint>,
}

impl MultiTransaction {
	/// max_priority_fee_per_gas (see EIP-1559)
	pub const fn max_priority_fee_per_gas(&self) -> U256 {
		if self.max_priority_fee_per_gas.0.is_zero() {
			self.gas_price.0
		} else {
			self.max_priority_fee_per_gas.0
		}
	}

	/// max_fee_per_gas (see EIP-1559)
	pub const fn max_fee_per_gas(&self) -> U256 {
		if self.max_fee_per_gas.0.is_zero() {
			self.gas_price.0
		} else {
			self.max_fee_per_gas.0
		}
	}

	/// Build transaction with given indexes.
	pub fn select(&self, indexes: &PostStateIndexes) -> Transaction {
		let data_index = indexes.data as usize;
		let access_list = if data_index < self.access_lists.len() {
			self.access_lists
				.get(data_index)
				.unwrap()
				.as_ref()
				.cloned()
				.unwrap_or_default()
				.into_iter()
				.map(|a| (a.address, a.storage_keys))
				.collect()
		} else {
			Vec::new()
		};

		let gas_price = if self.gas_price.0.is_zero() {
			self.max_fee_per_gas.0 + self.max_priority_fee_per_gas.0
		} else {
			self.gas_price.0
		};

		Transaction {
			data: self.data[data_index].clone(),
			gas_limit: self.gas_limit[indexes.gas as usize],
			gas_price: Uint(gas_price),
			nonce: self.nonce,
			to: self.to.clone(),
			value: self.value[indexes.value as usize],
			r: Default::default(),
			s: Default::default(),
			v: Default::default(),
			secret: self.secret,
			access_list,
		}
	}
}

/// Type alias for access lists (see EIP-2930)
pub type AccessList = Vec<AccessListTuple>;

/// Access list tuple (see https://eips.ethereum.org/EIPS/eip-2930).
/// Example test spec: https://github.com/ethereum/tests/blob/5490db3ff58d371c0c74826280256ba016b0bd5c/GeneralStateTests/stExample/accessListExample.json
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessListTuple {
	/// Address to access
	pub address: Address,
	/// Keys (slots) to access at that address
	pub storage_keys: Vec<H256>,
}

/// State test indexes deserialization.
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct PostStateIndexes {
	/// Index into transaction data set.
	pub data: u64,
	/// Index into transaction gas limit set.
	pub gas: u64,
	/// Index into transaction value set.
	pub value: u64,
}

/// State test indexed state result deserialization.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostStateResult {
	/// Post state hash
	pub hash: H256,
	/// Indexes
	pub indexes: PostStateIndexes,
	/// Expected error if the test is meant to fail
	pub expect_exception: Option<String>,
	/// Transaction bytes
	pub txbytes: Bytes,
}

#[cfg(test)]
mod tests {
	use super::{MultiTransaction, State};
	use serde_json;

	#[test]
	fn multi_transaction_deserialization() {
		let s = r#"{
			"data": [ "" ],
			"gasLimit": [ "0x2dc6c0", "0x222222" ],
			"gasPrice": "0x01",
			"nonce": "0x00",
			"secretKey": "45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8",
			"to": "1000000000000000000000000000000000000000",
			"value": [ "0x00", "0x01", "0x02" ]
		}"#;
		let _deserialized: MultiTransaction = serde_json::from_str(s).unwrap();
	}

	#[test]
	fn state_deserialization() {
		let s = r#"{
			"env": {
				"currentCoinbase": "2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
				"currentDifficulty": "0x0100",
				"currentGasLimit": "0x01c9c380",
				"currentNumber": "0x00",
				"currentTimestamp": "0x01",
				"previousHash": "5e20a0453cecd065ea59c37ac63e079ee08998b6045136a8ce6635c7912ec0b6"
			},
			"post": {
				"EIP150": [
					{
						"hash": "3e6dacc1575c6a8c76422255eca03529bbf4c0dda75dfc110b22d6dc4152396f",
                        "txbytes" : "0xf861800a84042c1d8094b94f5374fce5edbc8e2a8697c15331677e6ebf0b80801ca0f141d67812db948c9a4ea43c27d695248205c121ae8d924d23517ab09e38f369a03fe3cfedb4c9a7e61340b6fec87917690e92082f752ad820ad5006c7d49185ed",
						"indexes": { "data": 0, "gas": 0, "value": 0 }
					},
					{
						"hash": "99a450d8ce5b987a71346d8a0a1203711f770745c7ef326912e46761f14cd764",
                        "txbytes" : "0xf861800a84042c1d8094b94f5374fce5edbc8e2a8697c15331677e6ebf0b80801ca0f141d67812db948c9a4ea43c27d695248205c121ae8d924d23517ab09e38f369a03fe3cfedb4c9a7e61340b6fec87917690e92082f752ad820ad5006c7d49185ed",
						"indexes": { "data": 0, "gas": 0, "value": 1 }
					}
				],
				"EIP158": [
					{
						"hash": "3e6dacc1575c6a8c76422255eca03529bbf4c0dda75dfc110b22d6dc4152396f",
                        "txbytes" : "0xf861800a84042c1d8094b94f5374fce5edbc8e2a8697c15331677e6ebf0b80801ca0f141d67812db948c9a4ea43c27d695248205c121ae8d924d23517ab09e38f369a03fe3cfedb4c9a7e61340b6fec87917690e92082f752ad820ad5006c7d49185ed",
						"indexes": { "data": 0, "gas": 0, "value": 0 }
					},
					{
						"hash": "99a450d8ce5b987a71346d8a0a1203711f770745c7ef326912e46761f14cd764",
                        "txbytes" : "0xf861800a84042c1d8094b94f5374fce5edbc8e2a8697c15331677e6ebf0b80801ca0f141d67812db948c9a4ea43c27d695248205c121ae8d924d23517ab09e38f369a03fe3cfedb4c9a7e61340b6fec87917690e92082f752ad820ad5006c7d49185ed",
						"indexes": { "data": 0, "gas": 0, "value": 1  }
					}
				]
			},
			"pre": {
				"1000000000000000000000000000000000000000": {
					"balance": "0x0de0b6b3a7640000",
					"code": "0x6040600060406000600173100000000000000000000000000000000000000162055730f1600055",
					"nonce": "0x00",
					"storage": {
					}
				},
				"1000000000000000000000000000000000000001": {
					"balance": "0x0de0b6b3a7640000",
					"code": "0x604060006040600060027310000000000000000000000000000000000000026203d090f1600155",
					"nonce": "0x00",
					"storage": {
					}
				},
				"1000000000000000000000000000000000000002": {
					"balance": "0x00",
					"code": "0x600160025533600455346007553060e6553260e8553660ec553860ee553a60f055",
					"nonce": "0x00",
					"storage": {
					}
				},
				"a94f5374fce5edbc8e2a8697c15331677e6ebf0b": {
					"balance": "0x0de0b6b3a7640000",
					"code": "0x",
					"nonce": "0x00",
					"storage": {
					}
				}
			},
			"transaction": {
				"data": [ "" ],
				"gasLimit": [ "285000", "100000", "6000" ],
				"gasPrice": "0x01",
				"nonce": "0x00",
				"secretKey": "45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8",
				"to": "095e7baea6a6c7c4c2dfeb977efac326af552d87",
				"value": [ "10", "0" ]
			}
		}"#;
		let _deserialized: State = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
