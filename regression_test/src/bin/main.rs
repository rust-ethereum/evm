#[macro_use]
extern crate clap;
extern crate sputnikvm;
extern crate serde_json;
extern crate blockchain_geth;

use serde_json::{Value};
use std::process;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader};

use blockchain_geth::{regression};

fn main() {
    let matches = clap_app!(regression_test =>
        (version: "0.1")
        (author: "Ethereum Classic Contributors")
        (about: "Gaslighter - Tests the Ethereum Classic Virtual Machine in 5 different ways.")
        (@arg KEEP_GOING: -k --keep_going "Don't exit the program even if a test fails.")
        (@subcommand reg =>
            (about: "Performs an regression test on the entire Ethereum Classic blockchain.\n\nSteps to reproduce:\n* Install Ethereum Classic Geth: `$ go install github.com/ethereumproject/go-ethereum/cmd/geth`.\n* Run Geth with this command: `$ ~/go/bin/geth`.\n* Wait for the chain to sync.\n* <ctrl-c>\n* Change directory into the gaslighter directory `$ cd gaslighter`\n* Run this command: `$ RUST_BACKTRACE=1 RUST_LOG=gaslighter cargo run --bin gaslighter -- -k reg -c ~/.ethereum/chaindata/`")
            (@arg RPC: -r --rpc +takes_value +required "Domain of Ethereum Classic Geth's RPC endpoint. e.g. `-r localhost:8888`.")
        )
    ).get_matches();
    let mut has_regression_test_passed = true;
    let keep_going = if matches.is_present("KEEP_GOING") { true } else { false };
    if let Some(ref matches) = matches.subcommand_matches("reg") {
        let path = matches.value_of("RPC").unwrap();
        has_regression_test_passed = regression(path);
    }
    if has_regression_test_passed {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
