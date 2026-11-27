use evm::{
	backend::InMemoryBackend,
	interpreter::runtime::Log,
	uint::{H256, U256, U256Ext},
};
use rlp::{Encodable, RlpStream};
use sha3::{Digest, Keccak256};

#[derive(Debug)]
struct TrieAccount {
	nonce: U256,
	balance: U256,
	storage_root: H256,
	code_hash: H256,
}

impl Encodable for TrieAccount {
	fn rlp_append(&self, stream: &mut RlpStream) {
		stream.begin_list(4);
		stream.append(&self.nonce);
		stream.append(&self.balance);
		stream.append(&self.storage_root);
		stream.append(&self.code_hash);
	}
}

pub fn state_root(backend: &InMemoryBackend) -> H256 {
	let accounts = backend.state.iter().map(|(address, account)| {
		let storage = account.storage.iter().map(|(key, value)| {
			(
				key.as_bytes().to_vec(),
				rlp::encode(&U256::from_h256(*value)).to_vec(),
			)
		});
		let storage_root = from_ethereum_h256(ethereum::util::sec_trie_root(storage));
		let account = TrieAccount {
			nonce: account.nonce,
			balance: account.balance,
			storage_root,
			code_hash: H256::from_slice(&Keccak256::digest(&account.code)),
		};
		(address.as_bytes().to_vec(), rlp::encode(&account).to_vec())
	});
	from_ethereum_h256(ethereum::util::sec_trie_root(accounts))
}

pub fn logs_hash(logs: &[Log]) -> H256 {
	let mut outer = RlpStream::new_list(logs.len());
	for log in logs {
		outer.begin_list(3);
		outer.append(&log.address);
		outer.begin_list(log.topics.len());
		for topic in &log.topics {
			outer.append(topic);
		}
		outer.append(&log.data);
	}
	H256::from_slice(&Keccak256::digest(outer.out()))
}

fn from_ethereum_h256(value: impl AsRef<[u8]>) -> H256 {
	H256::from_slice(value.as_ref())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn empty_log_hash_matches_state_test_constant() {
		assert_eq!(
			hex::encode(logs_hash(&[]).as_bytes()),
			"1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347"
		);
	}
}
