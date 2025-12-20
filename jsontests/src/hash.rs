use evm::backend::InMemoryBackend;
#[allow(unused_imports)]
use evm::uint::{H256, U256, U256Ext};
use sha3::{Digest, Keccak256};

/// Basic account type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrieAccount {
	/// Nonce of the account.
	pub nonce: U256,
	/// Balance of the account.
	pub balance: U256,
	/// Storage root of the account.
	pub storage_root: H256,
	/// Code hash of the account.
	pub code_hash: H256,
	/// Code version of the account.
	pub code_version: U256,
}

impl rlp::Encodable for TrieAccount {
	fn rlp_append(&self, stream: &mut rlp::RlpStream) {
		let use_short_version = self.code_version == U256::ZERO;

		match use_short_version {
			true => {
				stream.begin_list(4);
			}
			false => {
				stream.begin_list(5);
			}
		}

		stream.append(&self.nonce);
		stream.append(&self.balance);
		stream.append(&self.storage_root);
		stream.append(&self.code_hash);

		if !use_short_version {
			stream.append(&self.code_version);
		}
	}
}

impl rlp::Decodable for TrieAccount {
	fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
		let use_short_version = match rlp.item_count()? {
			4 => true,
			5 => false,
			_ => return Err(rlp::DecoderError::RlpIncorrectListLen),
		};

		Ok(TrieAccount {
			nonce: rlp.val_at(0)?,
			balance: rlp.val_at(1)?,
			storage_root: rlp.val_at(2)?,
			code_hash: rlp.val_at(3)?,
			code_version: if use_short_version {
				U256::ZERO
			} else {
				rlp.val_at(4)?
			},
		})
	}
}

pub fn state_root(backend: &InMemoryBackend) -> H256 {
	let tree = backend
		.state
		.iter()
		.map(|(address, account)| {
			let storage_root = ethereum::util::sec_trie_root(
				account
					.storage
					.iter()
					.map(|(k, v)| (k, rlp::encode(&U256::from_h256(*v)))),
			);

			let code_hash = H256::from_slice(&Keccak256::digest(&account.code));
			let account = TrieAccount {
				nonce: account.nonce,
				balance: account.balance,
				storage_root,
				code_hash,
				code_version: U256::ZERO,
			};

			(address, rlp::encode(&account))
		})
		.collect::<Vec<_>>();

	ethereum::util::sec_trie_root(tree)
}
