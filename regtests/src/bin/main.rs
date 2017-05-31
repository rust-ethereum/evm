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
use sputnikvm::vm::{BlockHeader, Context, SeqVM, Patch, AccountCommitment, Account};
use sputnikvm::vm::errors::RequireError;
use gethrpc::{regression, GethRPCClient, RPCCall, RPCBlock, RPCTransaction};

fn upfront_cost(data: &str) -> Gas {
    let mut cost = Gas::from(21000u64);
    let data = read_hex(data).unwrap();
    for v in data {
        if v == 0 {
            cost = cost + Gas::from(4u64);
        } else {
            cost = cost + Gas::from(68u64);
        }
    }
    return cost;
}

fn from_rpc_block(block: &RPCBlock) -> BlockHeader {
    BlockHeader {
        coinbase: Address::from_str(&block.miner).unwrap(),
        timestamp: M256::from_str(&block.timestamp).unwrap(),
        number: M256::from_str(&block.number).unwrap(),
        difficulty: M256::from_str(&block.difficulty).unwrap(),
        gas_limit: Gas::from_str(&block.gasLimit).unwrap(),
    }
}

fn from_rpc_transaction_and_code(transaction: &RPCTransaction, code: &str) -> Context {
    Context {
        caller: Address::from_str(&transaction.from).unwrap(),
        address: Address::from_str(&transaction.to).unwrap(),
        origin: Address::from_str(&transaction.to).unwrap(),
        value: U256::from_str(&transaction.value).unwrap(),
        code: read_hex(code).unwrap(),
        data: read_hex(&transaction.input).unwrap(),
        gas_limit: Gas::from_str(&transaction.gas).unwrap(),
        gas_price: Gas::from_str(&transaction.gasPrice).unwrap(),
    }
}

fn main() {
    let matches = clap_app!(regtests =>
        (version: "0.1")
        (author: "Ethereum Classic Contributors")
        (about: "Performs an regression test on the entire Ethereum Classic blockchain.\n\nSteps to reproduce:\n* Install Ethereum Classic Geth: `$ go install github.com/ethereumproject/go-ethereum/cmd/geth`.\n* Run Geth with this command: `$ ~/go/bin/geth --rpc --rpcaddr 127.0.0.1 --rpcport 8545`.\n* Wait for the chain to sync.\n* Change directory into the `regtests` directory `$ cd regtests`\n* Run this command: `$ RUST_BACKTRACE=1 cargo run --bin regtests -- -r http://127.0.0.1:8545")
        (@arg RPC: -r --rpc +takes_value +required "Domain of Ethereum Classic Geth's RPC endpoint. e.g. `-r http://127.0.0.1:8545`.")
    ).get_matches();

    let address = matches.value_of("RPC").unwrap();
    let mut client = GethRPCClient::new(address);

    println!("net version: {}", client.net_version());

    let block = client.get_block_by_number(format!("0x{:x}", 49439).as_str());
    println!("block {}, transaction count: {}", block.number, block.transactions.len());
    let last_block_number = format!("0x{:x}", M256::from_str(&block.number).unwrap() - M256::from(1u64));

    let block_header = from_rpc_block(&block);

    for transaction_hash in block.transactions {
        println!("\nworking on transaction {}", transaction_hash);
        let transaction = client.get_transaction_by_hash(&transaction_hash);
        println!("transaction: {:?}", transaction);
        let receipt = client.get_transaction_receipt(&transaction_hash);
        println!("receipt: {:?}", receipt);
        if transaction.to.is_empty() {
            continue;
        }
        let code = client.get_code(&transaction.to, &block.number);
        println!("code: {:?}\n", code);

        let context = from_rpc_transaction_and_code(&transaction, &code);

        let mut vm = SeqVM::new(context.clone(), block_header.clone(), Patch::empty());
        loop {
            match vm.fire() {
                Ok(()) => {
                    println!("VM exited successfully, checking results ...");
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
                }
                Err(err) => {
                    println!("Unhandled require: {:?}", err);
                    unimplemented!()
                },
            }
        }

        println!("\ntests after the vm has run:");
        println!("1. return status: {:?}", vm.status());
        println!("2. test gasUsed == {}, actual VM result: 0x{:x}", receipt.gasUsed,
                 context.gas_limit + upfront_cost(&transaction.input) - vm.available_gas());
        println!("3. logs and order is {:?}, actual VM result: {:?}", receipt.logs, vm.logs());

        println!("\nwhen the block is finished, test:");
        println!("1. balances of all used accounts.");
        println!("2. storage values touched.");
        for account in vm.accounts() {
            match account {
                &Account::Full {
                    address,
                    balance,
                    ref changing_storage,
                    ..
                } => {
                    let expected_balance = client.get_balance(&format!("0x{:x}", address),
                                                              &block.number);
                    println!("account 0x{:x}, balance: 0x{:x} == {}", address,
                             balance, expected_balance);
                    let changing_storage: HashMap<M256, M256> = changing_storage.clone().into();
                    for (key, value) in changing_storage {
                        let expected_value = client.get_storage_at(&format!("0x{:x}", address),
                                                                   &format!("0x{:x}", key),
                                                                   &block.number);
                        println!("account 0x{:x}, storage 0x{:x}: 0x{:x} == {}", address,
                                 key, value, expected_value);
                    }
                },
                _ => unimplemented!(),
            }
        }
    }
}
