use std::fmt;

use ethereum::{EnvelopedDecodable, TransactionV4, unchecked::UncheckedTransactionEnvelope};
use evm::{
	backend::InMemoryBackend,
	standard::{TransactArgsCallCreate, TransactGasPrice},
	uint::{H160, H256, U256},
};
use rlp::Rlp;

use crate::schema::{AccessListItem, FixtureTransaction, HexParseError, PostIndexes};

pub const GAS_PER_BLOB: u64 = 1 << 17;
const MAX_BLOBS_PER_BLOCK: usize = 6;
const MAX_INITCODE_SIZE: usize = 0xc000;
const MAX_ACCOUNT_NONCE: u64 = u64::MAX;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionType {
	Legacy,
	AccessList,
	FeeMarket,
	Blob,
}

#[derive(Clone, Debug)]
pub struct NormalizedTransaction {
	pub transaction_type: TransactionType,
	pub chain_id: Option<U256>,
	pub nonce: U256,
	pub gas_limit: U256,
	pub gas_price: TransactGasPrice,
	pub maximum_gas_price: U256,
	pub action: Option<H160>,
	pub value: U256,
	pub input: Vec<u8>,
	pub access_list: Vec<(H160, Vec<H256>)>,
	pub max_fee_per_blob_gas: Option<U256>,
	pub blob_versioned_hashes: Vec<H256>,
	pub expected_sender: H160,
	pub raw: Vec<u8>,
}

impl NormalizedTransaction {
	pub fn from_fixture(
		transaction: &FixtureTransaction,
		indexes: PostIndexes,
		raw: Vec<u8>,
	) -> Result<Self, NormalizeError> {
		let transaction_type = match raw.first().copied() {
			Some(0x01) => TransactionType::AccessList,
			Some(0x02) => TransactionType::FeeMarket,
			Some(0x03) => TransactionType::Blob,
			Some(value) if value >= 0xc0 => TransactionType::Legacy,
			Some(value) => return Err(NormalizeError::UnsupportedType(value)),
			None => return Err(NormalizeError::EmptyEnvelope),
		};
		let data = transaction
			.data
			.get(indexes.data)
			.ok_or(NormalizeError::Index("data", indexes.data))?
			.bytes()?;
		let gas_limit = transaction
			.gas_limit
			.get(indexes.gas)
			.ok_or(NormalizeError::Index("gas", indexes.gas))?
			.u256()?;
		let value = transaction
			.value
			.get(indexes.value)
			.ok_or(NormalizeError::Index("value", indexes.value))?
			.u256()?;
		let access_list = transaction
			.access_lists
			.as_ref()
			.and_then(|lists| lists.get(indexes.data))
			.map(|items| normalize_access_list(items))
			.transpose()?
			.unwrap_or_default();
		let (gas_price, maximum_gas_price) = match transaction.gas_price.as_ref() {
			Some(price) => {
				let price = price.u256()?;
				(TransactGasPrice::Legacy(price), price)
			}
			None => {
				let priority = transaction
					.max_priority_fee_per_gas
					.as_ref()
					.ok_or(NormalizeError::Missing("maxPriorityFeePerGas"))?
					.u256()?;
				let maximum = transaction
					.max_fee_per_gas
					.as_ref()
					.ok_or(NormalizeError::Missing("maxFeePerGas"))?
					.u256()?;
				(
					TransactGasPrice::FeeMarket {
						max_priority: priority,
						max: maximum,
					},
					maximum,
				)
			}
		};
		let action = if transaction.to.is_empty() {
			None
		} else {
			Some(crate::schema::HexValue(transaction.to.clone()).h160()?)
		};
		let blob_versioned_hashes = transaction
			.blob_versioned_hashes
			.as_deref()
			.unwrap_or_default()
			.iter()
			.map(|value| value.h256())
			.collect::<Result<Vec<_>, _>>()?;

		Ok(Self {
			transaction_type,
			chain_id: transaction
				.chain_id
				.as_ref()
				.map(|value| value.u256())
				.transpose()?,
			nonce: transaction.nonce.u256()?,
			gas_limit,
			gas_price,
			maximum_gas_price,
			action,
			value,
			input: data,
			access_list,
			max_fee_per_blob_gas: transaction
				.max_fee_per_blob_gas
				.as_ref()
				.map(|value| value.u256())
				.transpose()?,
			blob_versioned_hashes,
			expected_sender: transaction.sender.h160()?,
			raw,
		})
	}

	pub fn call_create(&self) -> TransactArgsCallCreate {
		match self.action {
			Some(address) => TransactArgsCallCreate::Call {
				address,
				data: self.input.clone(),
			},
			None => TransactArgsCallCreate::Create {
				salt: None,
				init_code: self.input.clone(),
			},
		}
	}

	pub fn is_blob(&self) -> bool {
		self.transaction_type == TransactionType::Blob
	}
}

fn normalize_access_list(
	items: &[AccessListItem],
) -> Result<Vec<(H160, Vec<H256>)>, HexParseError> {
	items
		.iter()
		.map(|item| {
			Ok((
				item.address.h160()?,
				item.storage_keys
					.iter()
					.map(|key| key.h256())
					.collect::<Result<Vec<_>, _>>()?,
			))
		})
		.collect()
}

#[derive(Debug, thiserror::Error)]
pub enum NormalizeError {
	#[error(transparent)]
	Hex(#[from] HexParseError),
	#[error("empty transaction envelope")]
	EmptyEnvelope,
	#[error("unsupported transaction type 0x{0:02x}")]
	UnsupportedType(u8),
	#[error("missing transaction field {0}")]
	Missing(&'static str),
	#[error("transaction {0} index {1} is out of bounds")]
	Index(&'static str, usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationError {
	InvalidSignature,
	InvalidChainId,
	SenderNotEoa,
	NonceIsMax,
	NonceTooLow,
	NonceTooHigh,
	GasAllowanceExceeded,
	IntrinsicGasTooLow,
	InitcodeSizeExceeded,
	PriorityGreaterThanMaxFee,
	InsufficientMaxFeePerGas,
	GasPriceProductOverflow,
	InsufficientAccountFunds,
	BlobContractCreation,
	BlobCountExceeded,
	BlobZeroBlobs,
	BlobInvalidVersionedHash,
	InsufficientMaxFeePerBlobGas,
}

impl ValidationError {
	pub const fn fixture_name(self) -> &'static str {
		match self {
			Self::InvalidSignature => "TransactionException.INVALID_SIGNATURE_VRS",
			Self::InvalidChainId => "TransactionException.INVALID_CHAINID",
			Self::SenderNotEoa => "TransactionException.SENDER_NOT_EOA",
			Self::NonceIsMax => "TransactionException.NONCE_IS_MAX",
			Self::NonceTooLow => "TransactionException.NONCE_MISMATCH_TOO_LOW",
			Self::NonceTooHigh => "TransactionException.NONCE_MISMATCH_TOO_HIGH",
			Self::GasAllowanceExceeded => "TransactionException.GAS_ALLOWANCE_EXCEEDED",
			Self::IntrinsicGasTooLow => "TransactionException.INTRINSIC_GAS_TOO_LOW",
			Self::InitcodeSizeExceeded => "TransactionException.INITCODE_SIZE_EXCEEDED",
			Self::PriorityGreaterThanMaxFee => {
				"TransactionException.PRIORITY_GREATER_THAN_MAX_FEE_PER_GAS"
			}
			Self::InsufficientMaxFeePerGas => "TransactionException.INSUFFICIENT_MAX_FEE_PER_GAS",
			Self::GasPriceProductOverflow => "TransactionException.GASLIMIT_PRICE_PRODUCT_OVERFLOW",
			Self::InsufficientAccountFunds => "TransactionException.INSUFFICIENT_ACCOUNT_FUNDS",
			Self::BlobContractCreation => "TransactionException.TYPE_3_TX_CONTRACT_CREATION",
			Self::BlobCountExceeded => "TransactionException.TYPE_3_TX_BLOB_COUNT_EXCEEDED",
			Self::BlobZeroBlobs => "TransactionException.TYPE_3_TX_ZERO_BLOBS",
			Self::BlobInvalidVersionedHash => {
				"TransactionException.TYPE_3_TX_INVALID_BLOB_VERSIONED_HASH"
			}
			Self::InsufficientMaxFeePerBlobGas => {
				"TransactionException.INSUFFICIENT_MAX_FEE_PER_BLOB_GAS"
			}
		}
	}
}

impl fmt::Display for ValidationError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.fixture_name())
	}
}

pub struct ValidatedTransaction {
	pub transaction: NormalizedTransaction,
	pub sender: H160,
	pub blob_gas_fee: U256,
}

pub fn validate(
	transaction: NormalizedTransaction,
	backend: &InMemoryBackend,
	chain_id: U256,
	blob_base_fee: U256,
) -> Result<ValidatedTransaction, ValidationError> {
	if let Some(transaction_chain_id) = transaction.chain_id
		&& transaction_chain_id != chain_id
	{
		return Err(ValidationError::InvalidChainId);
	}
	if transaction.is_blob() && transaction.action.is_none() {
		return Err(ValidationError::BlobContractCreation);
	}
	let sender = recover_sender(&transaction)?;
	let account = backend.state.get(&sender).cloned().unwrap_or_default();
	if !account.code.is_empty() {
		return Err(ValidationError::SenderNotEoa);
	}
	if transaction.nonce >= U256::from(MAX_ACCOUNT_NONCE) {
		return Err(ValidationError::NonceIsMax);
	}
	if transaction.nonce < account.nonce {
		return Err(ValidationError::NonceTooLow);
	}
	if transaction.nonce > account.nonce {
		return Err(ValidationError::NonceTooHigh);
	}
	if transaction.gas_limit > backend.environment.block_gas_limit {
		return Err(ValidationError::GasAllowanceExceeded);
	}
	if transaction.action.is_none() && transaction.input.len() > MAX_INITCODE_SIZE {
		return Err(ValidationError::InitcodeSizeExceeded);
	}
	let intrinsic = intrinsic_gas(&transaction).ok_or(ValidationError::IntrinsicGasTooLow)?;
	if transaction.gas_limit < intrinsic {
		return Err(ValidationError::IntrinsicGasTooLow);
	}
	if let TransactGasPrice::FeeMarket { max_priority, max } = transaction.gas_price
		&& max_priority > max
	{
		return Err(ValidationError::PriorityGreaterThanMaxFee);
	}
	if transaction.maximum_gas_price < backend.environment.block_base_fee_per_gas {
		return Err(ValidationError::InsufficientMaxFeePerGas);
	}

	let mut blob_gas_fee = U256::zero();
	let mut maximum_blob_cost = U256::zero();
	if transaction.is_blob() {
		if transaction.blob_versioned_hashes.is_empty() {
			return Err(ValidationError::BlobZeroBlobs);
		}
		if transaction.blob_versioned_hashes.len() > MAX_BLOBS_PER_BLOCK {
			return Err(ValidationError::BlobCountExceeded);
		}
		if transaction
			.blob_versioned_hashes
			.iter()
			.any(|hash| hash.as_bytes()[0] != 0x01)
		{
			return Err(ValidationError::BlobInvalidVersionedHash);
		}
		let max_blob_fee = transaction.max_fee_per_blob_gas.unwrap_or_default();
		if max_blob_fee < blob_base_fee {
			return Err(ValidationError::InsufficientMaxFeePerBlobGas);
		}
		let blob_gas = U256::from(GAS_PER_BLOB)
			.checked_mul(U256::from(transaction.blob_versioned_hashes.len()))
			.ok_or(ValidationError::GasPriceProductOverflow)?;
		blob_gas_fee = blob_gas
			.checked_mul(blob_base_fee)
			.ok_or(ValidationError::GasPriceProductOverflow)?;
		maximum_blob_cost = blob_gas
			.checked_mul(max_blob_fee)
			.ok_or(ValidationError::GasPriceProductOverflow)?;
	}

	let execution_cost = transaction
		.gas_limit
		.checked_mul(transaction.maximum_gas_price)
		.ok_or(ValidationError::GasPriceProductOverflow)?;
	let maximum_cost = execution_cost
		.checked_add(transaction.value)
		.and_then(|cost| cost.checked_add(maximum_blob_cost))
		.ok_or(ValidationError::InsufficientAccountFunds)?;
	if account.balance < maximum_cost {
		return Err(ValidationError::InsufficientAccountFunds);
	}

	Ok(ValidatedTransaction {
		transaction,
		sender,
		blob_gas_fee,
	})
}

fn recover_sender(transaction: &NormalizedTransaction) -> Result<H160, ValidationError> {
	if !signature_is_low_s(&transaction.raw) {
		return Err(ValidationError::InvalidSignature);
	}
	let unchecked = UncheckedTransactionEnvelope::decode(&transaction.raw)
		.map_err(|_| ValidationError::InvalidSignature)?;
	if !envelope_matches_fixture(&unchecked, transaction) {
		return Err(ValidationError::InvalidSignature);
	}
	let checked =
		TransactionV4::try_from(unchecked).map_err(|_| ValidationError::InvalidSignature)?;
	let recovered = checked
		.recover_signer()
		.map_err(|_| ValidationError::InvalidSignature)?;
	let recovered = H160::from_slice(recovered.as_bytes());
	if recovered != transaction.expected_sender {
		return Err(ValidationError::InvalidSignature);
	}
	Ok(recovered)
}

fn envelope_matches_fixture(
	envelope: &UncheckedTransactionEnvelope,
	expected: &NormalizedTransaction,
) -> bool {
	match envelope {
		UncheckedTransactionEnvelope::Legacy(transaction) => {
			expected.transaction_type == TransactionType::Legacy
				&& integer_matches(&transaction.nonce, expected.nonce)
				&& integer_matches(&transaction.gas_price, expected.maximum_gas_price)
				&& integer_matches(&transaction.gas_limit, expected.gas_limit)
				&& action_matches(&transaction.action, expected.action)
				&& integer_matches(&transaction.value, expected.value)
				&& transaction.input.as_slice() == expected.input
		}
		UncheckedTransactionEnvelope::EIP2930(transaction) => {
			expected.transaction_type == TransactionType::AccessList
				&& expected
					.chain_id
					.is_some_and(|value| integer_matches(&transaction.chain_id, value))
				&& integer_matches(&transaction.nonce, expected.nonce)
				&& integer_matches(&transaction.gas_price, expected.maximum_gas_price)
				&& integer_matches(&transaction.gas_limit, expected.gas_limit)
				&& action_matches(&transaction.action, expected.action)
				&& integer_matches(&transaction.value, expected.value)
				&& transaction.input.as_slice() == expected.input
				&& access_list_matches(&transaction.access_list, &expected.access_list)
		}
		UncheckedTransactionEnvelope::EIP1559(transaction) => {
			let (max_priority, max) = fee_market_prices(expected);
			expected.transaction_type == TransactionType::FeeMarket
				&& expected
					.chain_id
					.is_some_and(|value| integer_matches(&transaction.chain_id, value))
				&& integer_matches(&transaction.nonce, expected.nonce)
				&& integer_matches(&transaction.max_priority_fee_per_gas, max_priority)
				&& integer_matches(&transaction.max_fee_per_gas, max)
				&& integer_matches(&transaction.gas_limit, expected.gas_limit)
				&& action_matches(&transaction.action, expected.action)
				&& integer_matches(&transaction.value, expected.value)
				&& transaction.input.as_slice() == expected.input
				&& access_list_matches(&transaction.access_list, &expected.access_list)
		}
		UncheckedTransactionEnvelope::EIP4844(transaction) => {
			let (max_priority, max) = fee_market_prices(expected);
			expected.transaction_type == TransactionType::Blob
				&& expected
					.chain_id
					.is_some_and(|value| integer_matches(&transaction.chain_id, value))
				&& integer_matches(&transaction.nonce, expected.nonce)
				&& integer_matches(&transaction.max_priority_fee_per_gas, max_priority)
				&& integer_matches(&transaction.max_fee_per_gas, max)
				&& integer_matches(&transaction.gas_limit, expected.gas_limit)
				&& action_matches(&transaction.destination, expected.action)
				&& integer_matches(&transaction.value, expected.value)
				&& transaction.data.as_slice() == expected.input
				&& access_list_matches(&transaction.access_list, &expected.access_list)
				&& expected
					.max_fee_per_blob_gas
					.is_some_and(|value| integer_matches(&transaction.max_fee_per_blob_gas, value))
				&& transaction.blob_versioned_hashes.len() == expected.blob_versioned_hashes.len()
				&& transaction
					.blob_versioned_hashes
					.iter()
					.zip(&expected.blob_versioned_hashes)
					.all(|(actual, expected)| actual.as_bytes() == expected.as_bytes())
		}
		UncheckedTransactionEnvelope::EIP7702(_) => false,
	}
}

fn fee_market_prices(transaction: &NormalizedTransaction) -> (U256, U256) {
	match transaction.gas_price {
		TransactGasPrice::FeeMarket { max_priority, max } => (max_priority, max),
		TransactGasPrice::Legacy(value) => (value, value),
	}
}

fn integer_matches(actual: &impl fmt::Display, expected: U256) -> bool {
	actual.to_string() == expected.to_string()
}

fn action_matches(actual: &ethereum::TransactionAction, expected: Option<H160>) -> bool {
	match (actual, expected) {
		(ethereum::TransactionAction::Create, None) => true,
		(ethereum::TransactionAction::Call(actual), Some(expected)) => {
			actual.as_bytes() == expected.as_bytes()
		}
		_ => false,
	}
}

fn access_list_matches(actual: &ethereum::AccessList, expected: &[(H160, Vec<H256>)]) -> bool {
	actual.len() == expected.len()
		&& actual
			.iter()
			.zip(expected)
			.all(|(actual, (address, keys))| {
				actual.address.as_bytes() == address.as_bytes()
					&& actual.storage_keys.len() == keys.len()
					&& actual
						.storage_keys
						.iter()
						.zip(keys)
						.all(|(actual, expected)| actual.as_bytes() == expected.as_bytes())
			})
}

fn signature_is_low_s(raw: &[u8]) -> bool {
	let Some(first) = raw.first().copied() else {
		return false;
	};
	let (payload, s_index) = match first {
		0x01 => (&raw[1..], 10),
		0x02 => (&raw[1..], 11),
		0x03 => (&raw[1..], 13),
		value if value >= 0xc0 => (raw, 8),
		_ => return false,
	};
	let Ok(s) = Rlp::new(payload).val_at::<U256>(s_index) else {
		return false;
	};
	let low_s = U256::from_big_endian(
		&hex::decode("7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0")
			.expect("constant is valid hex"),
	);
	s <= low_s
}

fn intrinsic_gas(transaction: &NormalizedTransaction) -> Option<U256> {
	let mut gas = if transaction.action.is_some() {
		21_000u64
	} else {
		53_000u64
	};
	for byte in &transaction.input {
		gas = gas.checked_add(if *byte == 0 { 4 } else { 16 })?;
	}
	for (_, keys) in &transaction.access_list {
		gas = gas.checked_add(2_400)?;
		gas = gas.checked_add(1_900u64.checked_mul(keys.len().try_into().ok()?)?)?;
	}
	if transaction.action.is_none() {
		gas = gas.checked_add(
			2u64.checked_mul(transaction.input.len().div_ceil(32).try_into().ok()?)?,
		)?;
	}
	Some(U256::from(gas))
}

/// EIP-4844 `fake_exponential(1, excess_blob_gas, update_fraction)`.
pub fn blob_base_fee(excess_blob_gas: U256, update_fraction: U256) -> Option<U256> {
	if update_fraction.is_zero() {
		return None;
	}
	let mut output = U256::zero();
	let mut numerator_accumulator = update_fraction;
	let mut i = U256::one();
	while !numerator_accumulator.is_zero() {
		output = output.checked_add(numerator_accumulator)?;
		numerator_accumulator = numerator_accumulator
			.checked_mul(excess_blob_gas)?
			.checked_div(update_fraction.checked_mul(i)?)?;
		i = i.checked_add(U256::one())?;
	}
	output.checked_div(update_fraction)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn cancun_blob_base_fee_vectors() {
		let fraction = U256::from(3_338_477u64);
		assert_eq!(blob_base_fee(U256::zero(), fraction), Some(U256::one()));
		assert_eq!(
			blob_base_fee(U256::from(2_359_296u64), fraction),
			Some(U256::from(2u64))
		);
	}

	#[test]
	fn exception_names_are_canonical() {
		assert_eq!(
			ValidationError::BlobZeroBlobs.fixture_name(),
			"TransactionException.TYPE_3_TX_ZERO_BLOBS"
		);
	}
}
