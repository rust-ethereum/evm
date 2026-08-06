use std::collections::BTreeMap;

use evm::{
	backend::{InMemoryAccount, InMemoryBackend, InMemoryEnvironment, OverlayedBackend},
	standard::{Config, TransactArgs},
	uint::{H256, U256},
};
use sha3::{Digest, Keccak256};

use crate::{
	hash::{logs_hash, state_root},
	schema::{HexParseError, PostState, StateFixture},
	transaction::{NormalizeError, NormalizedTransaction, blob_base_fee, validate},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionFailure {
	pub detail: String,
}

impl ExecutionFailure {
	fn new(detail: impl Into<String>) -> Self {
		Self {
			detail: detail.into(),
		}
	}
}

pub fn execute_case(fixture: &StateFixture, fork: &str) -> Result<(), ExecutionFailure> {
	if fork != "Cancun" {
		return Err(ExecutionFailure::new(format!(
			"unsupported target fork {fork:?}"
		)));
	}
	let posts = fixture
		.post
		.get(fork)
		.ok_or_else(|| ExecutionFailure::new(format!("fixture has no {fork} post state")))?;
	if posts.len() != 1 {
		return Err(ExecutionFailure::new(format!(
			"expected exactly one {fork} post state, found {}",
			posts.len()
		)));
	}
	execute_post(fixture, &posts[0], fork)
}

fn execute_post(
	fixture: &StateFixture,
	post: &PostState,
	fork: &str,
) -> Result<(), ExecutionFailure> {
	let chain_id = fixture.config.chainid.u256().map_err(parse_failure)?;
	let schedule = fixture.config.blob_schedule.get(fork).ok_or_else(|| {
		ExecutionFailure::new(format!("fixture config has no {fork} blob schedule"))
	})?;
	let blob_base_fee = blob_base_fee(
		fixture
			.env
			.current_excess_blob_gas
			.u256()
			.map_err(parse_failure)?,
		schedule
			.base_fee_update_fraction
			.u256()
			.map_err(parse_failure)?,
	)
	.ok_or_else(|| ExecutionFailure::new("blob base-fee calculation overflowed"))?;
	let mut backend = build_backend(fixture, chain_id, blob_base_fee)?;
	let raw = post.txbytes.bytes().map_err(parse_failure)?;
	let transaction = NormalizedTransaction::from_fixture(&fixture.transaction, post.indexes, raw)
		.map_err(normalize_failure)?;
	let validation = validate(transaction, &backend, chain_id, blob_base_fee);

	if let Some(expected) = &post.expect_exception {
		return match validation {
			Err(actual)
				if expected_errors(expected).any(|value| value == actual.fixture_name()) =>
			{
				compare_commitment(&backend, &[], post)
			}
			Err(actual) => Err(ExecutionFailure::new(format!(
				"transaction rejected as {actual}, expected one of {expected}"
			))),
			Ok(_) => Err(ExecutionFailure::new(format!(
				"transaction was accepted, expected rejection {expected}"
			))),
		};
	}

	let validated = validation.map_err(|actual| {
		ExecutionFailure::new(format!("transaction unexpectedly rejected as {actual}"))
	})?;
	if !validated.blob_gas_fee.is_zero() {
		let account = backend.state.entry(validated.sender).or_default();
		account.balance = account
			.balance
			.checked_sub(validated.blob_gas_fee)
			.ok_or_else(|| {
				ExecutionFailure::new("validated sender cannot pay the actual blob base fee")
			})?;
	}

	let config = Config::cancun();
	let args = TransactArgs {
		call_create: validated.transaction.call_create(),
		caller: validated.sender,
		value: validated.transaction.value,
		gas_limit: validated.transaction.gas_limit,
		gas_price: validated.transaction.gas_price,
		access_list: validated.transaction.access_list,
		config: &config,
	};
	let mut overlay = OverlayedBackend::new(&backend, &config.runtime);
	// A REVERT or execution exception is a valid transaction. Its outer nonce and fee effects are
	// represented in the overlay and are checked through the resulting commitments.
	let _execution_result = evm_mainnet::transact(args, &mut overlay);
	let (_, changeset) = overlay.deconstruct();
	let logs = changeset.logs.clone();
	backend.apply_overlayed(&changeset);
	compare_commitment(&backend, &logs, post)
}

fn build_backend(
	fixture: &StateFixture,
	chain_id: U256,
	blob_base_fee: U256,
) -> Result<InMemoryBackend, ExecutionFailure> {
	let block_number = fixture.env.current_number.u256().map_err(parse_failure)?;
	let block_hashes = if block_number.is_zero() {
		BTreeMap::new()
	} else {
		let previous = block_number - U256::one();
		BTreeMap::from([(
			previous,
			H256::from_slice(&Keccak256::digest(previous.to_string().as_bytes())),
		)])
	};
	let state = fixture
		.pre
		.iter()
		.map(|(address, account)| {
			let address = crate::schema::HexValue(address.clone())
				.h160()
				.map_err(parse_failure)?;
			let storage = account
				.storage
				.iter()
				.map(|(key, value)| {
					Ok((
						crate::schema::HexValue(key.clone())
							.h256()
							.map_err(parse_failure)?,
						value.h256().map_err(parse_failure)?,
					))
				})
				.collect::<Result<BTreeMap<_, _>, ExecutionFailure>>()?;
			Ok((
				address,
				InMemoryAccount {
					balance: account.balance.u256().map_err(parse_failure)?,
					code: account.code.bytes().map_err(parse_failure)?,
					nonce: account.nonce.u256().map_err(parse_failure)?,
					storage: storage
						.into_iter()
						.filter(|(_, value)| *value != H256::zero())
						.collect(),
					transient_storage: BTreeMap::new(),
				},
			))
		})
		.collect::<Result<BTreeMap<_, _>, ExecutionFailure>>()?;
	let blob_versioned_hashes = fixture
		.transaction
		.blob_versioned_hashes
		.as_deref()
		.unwrap_or_default()
		.iter()
		.map(|value| value.h256().map_err(parse_failure))
		.collect::<Result<Vec<_>, _>>()?;

	Ok(InMemoryBackend {
		environment: InMemoryEnvironment {
			// The state-test convention supplies the immediately preceding hash as
			// keccak256(decimal_block_number), even though modern JSON omits `previousHash`.
			block_hashes,
			block_number,
			block_coinbase: fixture.env.current_coinbase.h160().map_err(parse_failure)?,
			block_timestamp: fixture
				.env
				.current_timestamp
				.u256()
				.map_err(parse_failure)?,
			block_difficulty: fixture
				.env
				.current_difficulty
				.u256()
				.map_err(parse_failure)?,
			block_randomness: Some(fixture.env.current_random.h256().map_err(parse_failure)?),
			block_gas_limit: fixture
				.env
				.current_gas_limit
				.u256()
				.map_err(parse_failure)?,
			block_base_fee_per_gas: fixture.env.current_base_fee.u256().map_err(parse_failure)?,
			blob_base_fee_per_gas: blob_base_fee,
			blob_versioned_hashes,
			chain_id,
		},
		state,
	})
}

fn compare_commitment(
	backend: &InMemoryBackend,
	logs: &[evm::interpreter::runtime::Log],
	post: &PostState,
) -> Result<(), ExecutionFailure> {
	let expected_state = post.hash.h256().map_err(parse_failure)?;
	let expected_logs = post.logs.h256().map_err(parse_failure)?;
	let actual_state = state_root(backend);
	let actual_logs = logs_hash(logs);
	if actual_state != expected_state || actual_logs != expected_logs {
		return Err(ExecutionFailure::new(format!(
			"commitment mismatch: state actual=0x{} expected=0x{}; logs actual=0x{} expected=0x{}",
			hex::encode(actual_state.as_bytes()),
			hex::encode(expected_state.as_bytes()),
			hex::encode(actual_logs.as_bytes()),
			hex::encode(expected_logs.as_bytes()),
		)));
	}
	Ok(())
}

fn expected_errors(value: &str) -> impl Iterator<Item = &str> {
	value.split('|')
}

fn parse_failure(error: HexParseError) -> ExecutionFailure {
	ExecutionFailure::new(error.to_string())
}

fn normalize_failure(error: NormalizeError) -> ExecutionFailure {
	ExecutionFailure::new(error.to_string())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::transaction::ValidationError;

	#[test]
	fn alternatives_are_independent_expected_errors() {
		let values = expected_errors(
			"TransactionException.INSUFFICIENT_ACCOUNT_FUNDS|TransactionException.INTRINSIC_GAS_TOO_LOW",
		)
		.collect::<Vec<_>>();
		assert_eq!(values.len(), 2);
		assert_eq!(
			values[1],
			ValidationError::IntrinsicGasTooLow.fixture_name()
		);
	}
}
