#[macro_use]
extern crate clap;
extern crate sputnikvm;
extern crate bigint;
extern crate serde_json;
extern crate gethrpc;

use serde_json::{Value};
use std::process;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader};
use std::str::FromStr;
use std::collections::HashMap;

use sputnikvm::{Gas, Address};
use bigint::{U256, M256, read_hex};
use sputnikvm::vm::{BlockHeader, Context, SeqTransactionVM, Transaction, VM, Log, Patch, AccountCommitment, Account, FRONTIER_PATCH};
use sputnikvm::vm::errors::RequireError;
use gethrpc::{regression, GethRPCClient, RPCCall, RPCBlock, RPCTransaction, RPCLog};

fn from_rpc_block(block: &RPCBlock) -> BlockHeader {
    BlockHeader {
        coinbase: Address::from_str(&block.miner).unwrap(),
        timestamp: M256::from_str(&block.timestamp).unwrap(),
        number: M256::from_str(&block.number).unwrap(),
        difficulty: M256::from_str(&block.difficulty).unwrap(),
        gas_limit: Gas::from_str(&block.gasLimit).unwrap(),
    }
}

fn from_rpc_transaction(transaction: &RPCTransaction) -> Transaction {
    if transaction.to.is_empty() {
        Transaction::ContractCreation {
            caller: Address::from_str(&transaction.from).unwrap(),
            value: U256::from_str(&transaction.value).unwrap(),
            gas_limit: Gas::from_str(&transaction.gas).unwrap(),
            gas_price: Gas::from_str(&transaction.gasPrice).unwrap(),
            init: read_hex(&transaction.input).unwrap(),
        }
    } else {
        Transaction::MessageCall {
            caller: Address::from_str(&transaction.from).unwrap(),
            address: Address::from_str(&transaction.to).unwrap(),
            value: U256::from_str(&transaction.value).unwrap(),
            gas_limit: Gas::from_str(&transaction.gas).unwrap(),
            gas_price: Gas::from_str(&transaction.gasPrice).unwrap(),
            data: read_hex(&transaction.input).unwrap(),
        }
    }
}

fn from_rpc_log(log: &RPCLog) -> Log {
    let mut topics: Vec<M256> = Vec::new();
    for topic in &log.topics {
        topics.push(M256::from_str(&topic).unwrap());
    }
    Log {
        address: Address::from_str(&log.address).unwrap(),
        data: read_hex(&log.data).unwrap(),
        topics: topics,
    }
}

fn handle_fire(client: &mut GethRPCClient, vm: &mut SeqTransactionVM, last_block_number: &str) {
    loop {
        match vm.fire() {
            Ok(()) => {
                println!("VM exited with {:?}.", vm.status());
                break;
            },
            Err(RequireError::Account(address)) => {
                println!("Feeding VM account at 0x{:x} ...", address);
                let nonce = M256::from_str(&client.get_transaction_count(&format!("0x{:x}", address),
                                                                         &last_block_number)).unwrap();
                let balance = U256::from_str(&client.get_balance(&format!("0x{:x}", address),
                                                                 &last_block_number)).unwrap();
                let code = read_hex(&client.get_code(&format!("0x{:x}", address),
                                                     &last_block_number)).unwrap();
                vm.commit_account(AccountCommitment::Full {
                    nonce: nonce,
                    address: address,
                    balance: balance,
                    code: code,
                });
            },
            Err(RequireError::AccountStorage(address, index)) => {
                println!("Feeding VM account storage at 0x{:x} with index 0x{:x} ...", address, index);
                let value = M256::from_str(&client.get_storage_at(&format!("0x{:x}", address),
                                                                  &format!("0x{:x}", index),
                                                                  &last_block_number)).unwrap();
                vm.commit_account(AccountCommitment::Storage {
                    address: address,
                    index: index,
                    value: value,
                });
            },
            Err(RequireError::AccountCode(address)) => {
                println!("Feeding VM account code at 0x{:x} ...", address);
                let code = read_hex(&client.get_code(&format!("0x{:x}", address),
                                                     &last_block_number)).unwrap();
                vm.commit_account(AccountCommitment::Code {
                    address: address,
                    code: code,
                });
            }
            Err(err) => {
                println!("Unhandled require: {:?}", err);
                unimplemented!()
            },
        }
    }
}

fn test_block(client: &mut GethRPCClient, number: usize) {
    let block = client.get_block_by_number(format!("0x{:x}", number).as_str());
    println!("block {}, transaction count: {}", block.number, block.transactions.len());
    let last_number = number - 1;
    let block_header = from_rpc_block(&block);

    let mut last_vm: Option<SeqTransactionVM> = None;
    for transaction_hash in block.transactions {
        println!("\nworking on transaction {}", transaction_hash);
        let transaction = from_rpc_transaction(&client.get_transaction_by_hash(&transaction_hash));
        let receipt = client.get_transaction_receipt(&transaction_hash);

        let mut vm = if last_vm.is_none() {
            SeqTransactionVM::new(transaction, block_header.clone(), &FRONTIER_PATCH)
        } else {
            SeqTransactionVM::with_previous(transaction, block_header.clone(), &FRONTIER_PATCH, last_vm.as_ref().unwrap())
        };

        handle_fire(client, &mut vm, &format!("0x{:x}", last_number));

        assert!(Gas::from_str(&receipt.gasUsed).unwrap() == vm.real_used_gas());
        assert!(receipt.logs.len() == vm.logs().len());
        for i in 0..receipt.logs.len() {
            assert!(from_rpc_log(&receipt.logs[i]) == vm.logs()[i]);
        }
    }

    if last_vm.is_some() {
        for account in last_vm.as_ref().unwrap().accounts() {
            match account {
                &Account::Full {
                    address,
                    balance,
                    ref changing_storage,
                    ..
                } => {
                    let expected_balance = client.get_balance(&format!("0x{:x}", address),
                                                              &block.number);
                    assert!(U256::from_str(&expected_balance).unwrap() == balance);
                    let changing_storage: HashMap<M256, M256> = changing_storage.clone().into();
                    for (key, value) in changing_storage {
                        let expected_value = client.get_storage_at(&format!("0x{:x}", address),
                                                                   &format!("0x{:x}", key),
                                                                   &block.number);
                        assert!(M256::from_str(&expected_value).unwrap() == value);
                    }
                },
                _ => unimplemented!(),
            }
        }
    }
}

fn main() {
    let matches = clap_app!(regtests =>
        (version: "0.1")
        (author: "Ethereum Classic Contributors")
        (about: "Performs an regression test on the entire Ethereum Classic blockchain.\n\nSteps to reproduce:\n* Install Ethereum Classic Geth: `$ go install github.com/ethereumproject/go-ethereum/cmd/geth`.\n* Run Geth with this command: `$ ~/go/bin/geth --rpc --rpcaddr 127.0.0.1 --rpcport 8545`.\n* Wait for the chain to sync.\n* Change directory into the `regtests` directory `$ cd regtests`\n* Run this command: `$ RUST_BACKTRACE=1 cargo run --bin regtests -- -r http://127.0.0.1:8545")
        (@arg RPC: -r --rpc +takes_value +required "Domain of Ethereum Classic Geth's RPC endpoint. e.g. `-r http://127.0.0.1:8545`.")
        (@arg NUMBER: -n --number +takes_value + required "Block number to run this test. Radix is 10. e.g. `-n 49439`.")
    ).get_matches();

    let address = matches.value_of("RPC").unwrap();
    let number = matches.value_of("NUMBER").unwrap();
    let mut client = GethRPCClient::new(address);
    println!("net version: {}", client.net_version());

    if number.contains("..") {
        let number: Vec<&str> = number.split("..").collect();
        let from = usize::from_str_radix(&number[0], 10).unwrap();
        let to = usize::from_str_radix(&number[1], 10).unwrap();
        for n in from..to {
            test_block(&mut client, n);
        }
    } else {
        let number = usize::from_str_radix(&number, 10).unwrap();
        test_block(&mut client, number);
    }
}
