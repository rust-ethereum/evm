#[macro_use]
extern crate clap;
extern crate sputnikvm;
extern crate serde_json;
extern crate gethrpc;

use serde_json::{Value};
use std::process;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader};

use gethrpc::{regression, GethRPCClient, RPCCall};

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
        let call_result = client.call(RPCCall {
            from: transaction.from.clone(),
            to: transaction.to.clone(),
            gas: transaction.gas.clone(),
            gasPrice: transaction.gasPrice.clone(),
            value: transaction.value.clone(),
            data: transaction.input.clone(),
        }, &receipt.blockNumber);
        println!("call result: {:?}", call_result);

        println!("\ntests after the vm has run:");
        println!("1. test out == {:?}", call_result);
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
