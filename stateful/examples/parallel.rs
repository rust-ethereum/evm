extern crate trie;
extern crate block;
extern crate hexutil;
extern crate bigint;
extern crate sputnikvm;
extern crate sputnikvm_stateful;

use hexutil::*;
use block::TransactionAction;
use bigint::{Address, U256, Gas};
use sputnikvm::{AccountChange, HeaderParams, SeqTransactionVM, VM, Storage, EIP160Patch, ValidTransaction};
use sputnikvm_stateful::MemoryStateful;
use std::thread;
use std::collections::HashSet;
use std::sync::Arc;
use std::str::FromStr;

fn is_modified(modified_addresses: &HashSet<Address>, accounts: &[AccountChange]) -> bool {
    for account in accounts {
        if modified_addresses.contains(&account.address()) {
            return true;
        }
    }
    return false;
}

pub fn parallel_execute(
    stateful: MemoryStateful, transactions: &[ValidTransaction]
) -> MemoryStateful {
    let header = HeaderParams {
        beneficiary: Address::zero(),
        timestamp: 0,
        number: U256::zero(),
        difficulty: U256::zero(),
        gas_limit: Gas::max_value(),
    };

    let stateful = Arc::new(stateful);
    let mut threads = Vec::new();

    // Execute all transactions in parallel.
    for transaction in transactions {
        let transaction = transaction.clone();
        let header = header.clone();
        let stateful = stateful.clone();

        threads.push(thread::spawn(move || {
            let vm: SeqTransactionVM<EIP160Patch> = stateful.call(
                transaction, header, &[]);
            let accounts: Vec<AccountChange> = vm.accounts().map(|v| v.clone()).collect();
            (accounts, vm.used_addresses())
        }));
    }

    // Join all results together.
    let mut thread_accounts = Vec::new();
    for thread in threads {
        let accounts = thread.join().unwrap();
        thread_accounts.push(accounts);
    }

    // Now we only have a single reference to stateful, unwrap it and
    // start the state transition.
    let mut stateful = match Arc::try_unwrap(stateful) {
        Ok(val) => val,
        Err(_) => panic!(),
    };
    let mut modified_addresses = HashSet::new();

    for (index, (accounts, used_addresses)) in thread_accounts.into_iter().enumerate() {
        let (accounts, used_addresses) = if is_modified(&modified_addresses, &accounts) {
            // Re-execute the transaction if conflict is detected.
            println!("Transaction index {}: conflict detected, re-execute.", index);
            let vm: SeqTransactionVM<EIP160Patch> = stateful.call(
                transactions[index].clone(), header.clone(), &[]);
            let accounts: Vec<AccountChange> = vm.accounts().map(|v| v.clone()).collect();
            (accounts, vm.used_addresses())
        } else {
            println!("Transaction index {}: parallel execution successful.", index);
            (accounts, used_addresses)
        };

        stateful.transit(&accounts);
        modified_addresses.extend(used_addresses.into_iter());
    }

    stateful
}

fn main() {
    let mut stateful = MemoryStateful::default();

    let addr1 = Address::from_str("0x0000000000000000000000000000000000001000").unwrap();
    let addr2 = Address::from_str("0x0000000000000000000000000000000000001001").unwrap();
    let addr3 = Address::from_str("0x0000000000000000000000000000000000001002").unwrap();

    // Input some initial accounts.
    stateful.transit(&[
        AccountChange::Create {
            nonce: U256::zero(),
            address: addr1,
            balance: U256::from_str("0x1000000000000000000").unwrap(),
            storage: Storage::full(addr1),
            code: read_hex("0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055").unwrap(),
            exists: true,
        },
        AccountChange::Create {
            nonce: U256::zero(),
            address: addr2,
            balance: U256::from_str("0x1000000000000000000").unwrap(),
            storage: Storage::full(addr2),
            code: read_hex("0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055").unwrap(),
            exists: true,
        },
        AccountChange::Create {
            nonce: U256::zero(),
            address: addr3,
            balance: U256::from_str("0x1000000000000000000").unwrap(),
            storage: Storage::full(addr3),
            code: read_hex("0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055").unwrap(),
            exists: true,
        },
    ]);

    // Execute several crafted transactions.
    let stateful = parallel_execute(stateful, &[
        ValidTransaction {
            caller: Some(addr1),
            action: TransactionAction::Call(addr1),
            gas_price: Gas::zero(),
            gas_limit: Gas::max_value(),
            value: U256::from_str("0x1000").unwrap(),
            input: Vec::new(),
            nonce: U256::zero(),
        },
        ValidTransaction {
            caller: Some(addr2),
            action: TransactionAction::Call(addr3),
            gas_price: Gas::zero(),
            gas_limit: Gas::max_value(),
            value: U256::from_str("0x1000").unwrap(),
            input: Vec::new(),
            nonce: U256::zero(),
        },
        ValidTransaction {
            caller: Some(addr3),
            action: TransactionAction::Call(addr2),
            gas_price: Gas::zero(),
            gas_limit: Gas::max_value(),
            value: U256::from_str("0x1000").unwrap(),
            input: Vec::new(),
            nonce: U256::zero(),
        },
    ]);

    println!("New state root: 0x{:x}", stateful.root());
}
