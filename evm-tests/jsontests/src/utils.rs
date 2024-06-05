use evm::backend::MemoryAccount;
use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};
use std::collections::BTreeMap;

pub fn u256_to_h256(u: U256) -> H256 {
	let mut h = H256::default();
	u.to_big_endian(&mut h[..]);
	h
}

pub fn unwrap_to_account(s: &ethjson::spec::Account) -> MemoryAccount {
	MemoryAccount {
		balance: s.balance.unwrap().into(),
		nonce: s.nonce.unwrap().0,
		code: s.code.clone().unwrap().into(),
		storage: s
			.storage
			.as_ref()
			.unwrap()
			.iter()
			.filter_map(|(k, v)| {
				if v.0.is_zero() {
					// If value is zero then the key is not really there
					None
				} else {
					Some((u256_to_h256((*k).into()), u256_to_h256((*v).into())))
				}
			})
			.collect(),
	}
}

pub fn unwrap_to_state(a: &ethjson::spec::State) -> BTreeMap<H160, MemoryAccount> {
	match &a.0 {
		ethjson::spec::HashOrMap::Map(m) => m
			.iter()
			.map(|(k, v)| ((*k).into(), unwrap_to_account(v)))
			.collect(),
		ethjson::spec::HashOrMap::Hash(_) => panic!("Hash can not be converted."),
	}
}

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
		let use_short_version = self.code_version == U256::zero();

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

		Ok(Self {
			nonce: rlp.val_at(0)?,
			balance: rlp.val_at(1)?,
			storage_root: rlp.val_at(2)?,
			code_hash: rlp.val_at(3)?,
			code_version: if use_short_version {
				U256::zero()
			} else {
				rlp.val_at(4)?
			},
		})
	}
}

pub fn assert_valid_state(a: &ethjson::spec::State, b: &BTreeMap<H160, MemoryAccount>) {
	match &a.0 {
		ethjson::spec::HashOrMap::Map(m) => {
			assert_eq!(
				&m.iter()
					.map(|(k, v)| { ((*k).into(), unwrap_to_account(v)) })
					.collect::<BTreeMap<_, _>>(),
				b
			);
		}
		ethjson::spec::HashOrMap::Hash(h) => {
			let x = assert_valid_hash(&(*h).into(), b);
			if !x.0 {
				panic!("Wrong hash: {:#x?}", x.1);
			}
		}
	}
}

pub fn assert_valid_hash(h: &H256, b: &BTreeMap<H160, MemoryAccount>) -> (bool, H256) {
	let tree = b
		.iter()
		.map(|(address, account)| {
			let storage_root = ethereum::util::sec_trie_root(
				account
					.storage
					.iter()
					.map(|(k, v)| (k, rlp::encode(&U256::from_big_endian(&v[..])))),
			);
			let code_hash = H256::from_slice(Keccak256::digest(&account.code).as_slice());

			let account = TrieAccount {
				nonce: account.nonce,
				balance: account.balance,
				storage_root,
				code_hash,
				code_version: U256::zero(),
			};

			(address, rlp::encode(&account))
		})
		.collect::<Vec<_>>();

	let root = ethereum::util::sec_trie_root(tree);
	let expect = h;
	(root == *expect, root)
}

pub fn flush() {
	use std::io::{self, Write};

	io::stdout().flush().expect("Could not flush stdout");
}

pub mod transaction {
	use ethjson::hash::Address;
	use ethjson::maybe::MaybeEmpty;
	use ethjson::spec::ForkSpec;
	use ethjson::test_helpers::state::MultiTransaction;
	use ethjson::transaction::Transaction;
	use ethjson::uint::Uint;
	use evm::backend::MemoryVicinity;
	use evm::gasometer::{self, Gasometer};
	use evm::utils::{MAX_BLOB_NUMBER_PER_BLOCK, VERSIONED_HASH_VERSION_KZG};
	use primitive_types::{H160, H256, U256};

	// TODO: it will be refactored as old solution inefficient, also will be removed clippy-allow flag
	#[allow(clippy::too_many_arguments)]
	pub fn validate(
		tx: &Transaction,
		block_gas_limit: U256,
		caller_balance: U256,
		config: &evm::Config,
		test_tx: &MultiTransaction,
		vicinity: &MemoryVicinity,
		blob_gas_price: Option<u128>,
		data_fee: Option<U256>,
		spec: &ForkSpec,
	) -> Result<(), InvalidTxReason> {
		match intrinsic_gas(tx, config) {
			None => return Err(InvalidTxReason::IntrinsicGas),
			Some(required_gas) => {
				if tx.gas_limit < Uint(U256::from(required_gas)) {
					return Err(InvalidTxReason::IntrinsicGas);
				}
			}
		}

		if block_gas_limit < tx.gas_limit.0 {
			return Err(InvalidTxReason::GasLimitReached);
		}

		let required_funds = tx
			.gas_limit
			.0
			.checked_mul(vicinity.gas_price)
			.ok_or(InvalidTxReason::OutOfFund)?
			.checked_add(tx.value.0)
			.ok_or(InvalidTxReason::OutOfFund)?;

		let required_funds = if let Some(data_fee) = data_fee {
			required_funds
				.checked_add(data_fee)
				.ok_or(InvalidTxReason::OutOfFund)?
		} else {
			required_funds
		};
		if caller_balance < required_funds {
			return Err(InvalidTxReason::OutOfFund);
		}

		// CANCUN tx validation
		// Presence of max_fee_per_blob_gas means that this is blob transaction.
		if *spec >= ForkSpec::Cancun {
			if let Some(max) = test_tx.max_fee_per_blob_gas {
				// ensure that the user was willing to at least pay the current blob gasprice
				if U256::from(blob_gas_price.expect("expect blob_gas_price")) > max.0 {
					return Err(InvalidTxReason::BlobGasPriceGreaterThanMax);
				}

				// there must be at least one blob
				if test_tx.blob_versioned_hashes.is_empty() {
					return Err(InvalidTxReason::EmptyBlobs);
				}

				// The field `to` deviates slightly from the semantics with the exception
				// that it MUST NOT be nil and therefore must always represent
				// a 20-byte address. This means that blob transactions cannot
				// have the form of a create transaction.
				let to_address: Option<Address> = test_tx.to.clone().into();
				if to_address.is_none() {
					return Err(InvalidTxReason::BlobCreateTransaction);
				}

				// all versioned blob hashes must start with VERSIONED_HASH_VERSION_KZG
				for blob in test_tx.blob_versioned_hashes.iter() {
					let mut blob_hash = H256([0; 32]);
					blob.to_big_endian(&mut blob_hash[..]);
					if blob_hash[0] != VERSIONED_HASH_VERSION_KZG {
						return Err(InvalidTxReason::BlobVersionNotSupported);
					}
				}

				// ensure the total blob gas spent is at most equal to the limit
				// assert blob_gas_used <= MAX_BLOB_GAS_PER_BLOCK
				if test_tx.blob_versioned_hashes.len() > MAX_BLOB_NUMBER_PER_BLOCK as usize {
					return Err(InvalidTxReason::TooManyBlobs);
				}
			}
		} else {
			if !test_tx.blob_versioned_hashes.is_empty() {
				return Err(InvalidTxReason::BlobVersionedHashesNotSupported);
			}
			if test_tx.max_fee_per_blob_gas.is_some() {
				return Err(InvalidTxReason::MaxFeePerBlobGasNotSupported);
			}
		}

		Ok(())
	}

	fn intrinsic_gas(tx: &Transaction, config: &evm::Config) -> Option<u64> {
		let is_contract_creation = match tx.to {
			MaybeEmpty::None => true,
			MaybeEmpty::Some(_) => false,
		};
		let data = &tx.data;
		let access_list: Vec<(H160, Vec<H256>)> = tx
			.access_list
			.iter()
			.map(|(a, s)| (a.0, s.iter().map(|h| h.0).collect()))
			.collect();

		let cost = if is_contract_creation {
			gasometer::create_transaction_cost(data, &access_list)
		} else {
			gasometer::call_transaction_cost(data, &access_list)
		};

		let mut g = Gasometer::new(u64::MAX, config);
		g.record_transaction(cost).ok()?;

		Some(g.total_used_gas())
	}

	#[derive(Debug)]
	pub enum InvalidTxReason {
		IntrinsicGas,
		OutOfFund,
		GasLimitReached,
		PriorityFeeTooLarge,
		GasPriceLessThenBlockBaseFee,
		BlobCreateTransaction,
		BlobVersionNotSupported,
		TooManyBlobs,
		EmptyBlobs,
		BlobGasPriceGreaterThanMax,
		BlobVersionedHashesNotSupported,
		MaxFeePerBlobGasNotSupported,
	}
}
