// Copyright 2025 Security Research Labs GmbH
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

mod backend;
mod grammar;
use std::alloc::System;

use backend::MockBackend;
use evm::{
	backend::{OverlayedBackend, RuntimeBaseBackend},
	standard::{Config, Etable, EtableResolver, Invoker, TransactArgs, TransactValue},
};
use evm_interpreter::error::ExitError;
use primitive_types::{H160, U256};
#[cfg(not(feature = "fuzzing"))]
use stats_alloc::{StatsAlloc, INSTRUMENTED_SYSTEM};

use crate::grammar::FuzzData;
#[cfg(not(feature = "fuzzing"))]
#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn main() {
	ziggy::fuzz!(|data: FuzzData| {
		// CONFIG
		let init_balance = U256::from(1_000_000_000);
		let gas_limit = U256::from(400_000);
		let gas_price = U256::from(30);
		let config = Config::cancun();

		let mut contract_one = vec![];
		for op in &data.contract_one {
			op.to_bytes(&mut contract_one)
		}
		let mut contract_two = vec![];
		for op in &data.contract_two {
			op.to_bytes(&mut contract_two)
		}
		let calldata = data.call_data;
		// INITIALIZE
		let user_address = H160::from_low_u64_be(1);
		let mut backend = backend::MockBackend::default();

		backend.state.insert(
			user_address,
			backend::MockAccount {
				balance: init_balance,
				code: vec![],
				nonce: U256::one(),
				storage: Default::default(),
				transient_storage: Default::default(),
			},
		);

		// CREATE
		let now = std::time::Instant::now();
		let args = TransactArgs::Create {
			caller: H160::from_low_u64_be(1),
			value: U256::zero(),
			init_code: contract_one,
			salt: None,
			gas_limit,
			gas_price,
			access_list: vec![],
		};
		let (deployed, backend, new_balance, _memory_alloc, _storage_alloc) = fuzz_transact(
			user_address,
			args,
			&config,
			backend,
			init_balance,
			init_balance,
			gas_price,
			gas_limit,
		);
		let Ok(TransactValue::Create { succeed, address }) = deployed else {
			return;
		};
		let args = TransactArgs::Create {
			caller: H160::from_low_u64_be(1),
			value: U256::zero(),
			init_code: contract_two,
			salt: None,
			gas_limit,
			gas_price,
			access_list: vec![],
		};
		// CREATE SECOND CONTRACT
		let (deployed, backend, new_balance, _memory_alloc, _storage_alloc) = fuzz_transact(
			user_address,
			args,
			&config,
			backend,
			new_balance,
			init_balance,
			gas_price,
			gas_limit,
		);
		// CALL
		let args = TransactArgs::Call {
			caller: user_address,
			address,
			value: U256::zero(),
			data: calldata,
			gas_limit,
			gas_price,
			access_list: vec![],
		};
		let (result, backend, new_balance, memory_alloc, storage_alloc) = fuzz_transact(
			user_address,
			args,
			&config,
			backend.clone(),
			new_balance,
			init_balance,
			gas_price,
			gas_limit,
		);
	});
}
fn assert_no_mint(backend: &MockBackend, initial_balance: U256) -> U256 {
	let mut total = U256::zero();
	for (k, v) in backend.state.iter() {
		total += v.balance;
	}
	debug_assert!(total <= initial_balance);
	total
}

fn inner_transact(
	config: &Config,
	args: TransactArgs,
	overlayed_backend: &mut OverlayedBackend<MockBackend>,
) -> (Result<TransactValue, ExitError>, usize, usize) {
	let gas_etable = Etable::single(evm::standard::eval_gasometer);
	let exec_etable = Etable::runtime();
	let etable = (gas_etable, exec_etable);
	let resolver = EtableResolver::new(config, &(), &etable);
	let invoker = Invoker::new(config, &resolver);

	#[cfg(not(feature = "fuzzing"))]
	let allocated_before = GLOBAL.stats().bytes_allocated;
	#[cfg(feature = "fuzzing")]
	let allocated_before = 0;

	#[cfg(not(feature = "fuzzing"))]
	let de_allocated_before = GLOBAL.stats().bytes_deallocated;
	#[cfg(feature = "fuzzing")]
	let de_allocated_before = 0;

	let result = evm::transact(args.clone(), Some(4), overlayed_backend, &invoker);

	#[cfg(not(feature = "fuzzing"))]
	let allocated_after = GLOBAL.stats().bytes_allocated;
	#[cfg(feature = "fuzzing")]
	let allocated_after = 0;

	#[cfg(not(feature = "fuzzing"))]
	let de_allocated_after = GLOBAL.stats().bytes_deallocated;
	#[cfg(feature = "fuzzing")]
	let de_allocated_after = 0;

	let memory_allocated = allocated_after - allocated_before;
	let storage_allocated =
		(allocated_after - de_allocated_after) - (allocated_before - de_allocated_before);

	(result, memory_allocated, storage_allocated)
}

fn fuzz_transact(
	user_address: H160,
	args: TransactArgs,
	config: &Config,
	backend: MockBackend,
	prev_balance: U256,
	initial_balance: U256,
	gas_price: U256,
	gas_limit: U256,
) -> (
	Result<TransactValue, ExitError>,
	MockBackend,
	U256,
	usize,
	usize,
) {
	let mut overlayed_backend = OverlayedBackend::new(backend, Default::default(), config);
	#[cfg(not(feature = "fuzzing"))]
	let now = std::time::Instant::now();

	let (result, memory_alloc, storage_alloc) =
		inner_transact(config, args, &mut overlayed_backend);

	#[cfg(not(feature = "fuzzing"))]
	println!(
		"time={:?};memory_alloc={:?};storage_alloc={:?}\n",
		now.elapsed(),
		memory_alloc,
		storage_alloc
	);

	let (mut backend, changeset) = overlayed_backend.deconstruct();
	backend.apply_overlayed(&changeset);
	let balance = backend.balance(user_address);
	// INVARIANTS
	debug_assert!(balance < prev_balance);
	assert_no_mint(&backend, initial_balance);
	debug_assert!(balance >= prev_balance - gas_price * gas_limit);
	(result, backend, balance, memory_alloc, storage_alloc)
}
