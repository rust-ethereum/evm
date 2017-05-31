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

use sputnikvm::{Gas, Address};
use bigint::{U256, M256, read_hex};
use sputnikvm::vm::{BlockHeader, Context, VM};
use gethrpc::{regression, GethRPCClient, RPCCall, RPCBlock, RPCTransaction};

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
    let mut client = GethRPCClient::new("http://127.0.0.1:8545");

    println!("net version: {}", client.net_version());

    let block = client.get_block_by_number(format!("0x{:x}", 49439).as_str());
    println!("block {}, transaction count: {}", block.number, block.transactions.len());

    for transaction_hash in block.transactions {
        println!("\nworking on transaction {}", transaction_hash);
        let transaction = client.get_transaction_by_hash(&transaction_hash);
        println!("transaction: {:?}", transaction);
        let receipt = client.get_transaction_receipt(&transaction_hash);
        println!("receipt: {:?}", receipt);

        println!("\ntests after the vm has run:");
        println!("2. test gasUsed == {:?}", receipt.gasUsed);
        println!("3. logs and order is {:?}", receipt.logs);
    }

    println!("\nwhen the block is finished, test:");
    println!("1. balances of all used accounts.");
    println!("2. storage values touched.");

    // let matches = clap_app!(regression_test =>
    //     (version: "0.1")
    //     (author: "Ethereum Classic Contributors")
    //     (about: "Gaslighter - Tests the Ethereum Classic Virtual Machine in 5 different ways.")
    //     (@arg KEEP_GOING: -k --keep_going "Don't exit the program even if a test fails.")
    //     (@subcommand reg =>
    //         (about: "Performs an regression test on the entire Ethereum Classic blockchain.\n\nSteps to reproduce:\n* Install Ethereum Classic Geth: `$ go install github.com/ethereumproject/go-ethereum/cmd/geth`.\n* Run Geth with this command: `$ ~/go/bin/geth`.\n* Wait for the chain to sync.\n* <ctrl-c>\n* Change directory into the gaslighter directory `$ cd gaslighter`\n* Run this command: `$ RUST_BACKTRACE=1 RUST_LOG=gaslighter cargo run --bin gaslighter -- -k reg -c ~/.ethereum/chaindata/`")
    //         (@arg RPC: -r --rpc +takes_value +required "Domain of Ethereum Classic Geth's RPC endpoint. e.g. `-r localhost:8888`.")
    //     )
    // ).get_matches();
    // let mut has_regression_test_passed = true;
    // let keep_going = if matches.is_present("KEEP_GOING") { true } else { false };
    // if let Some(ref matches) = matches.subcommand_matches("reg") {
    //     let path = matches.value_of("RPC").unwrap();
    //     has_regression_test_passed = regression(path);
    // }
    // if has_regression_test_passed {
    //     process::exit(0);
    // } else {
    //     process::exit(1);
    // }
}
