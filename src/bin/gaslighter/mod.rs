#[macro_use]
extern crate clap;
extern crate sputnikvm;
extern crate libloading;
extern crate libc;
extern crate serde_json;
extern crate rustyline;

mod reg;
mod crat;
mod json;

use serde_json::{Value, Error};
use std::process;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Write, stdout};

use sputnikvm::{read_hex, Gas};
use sputnikvm::vm::{Machine, SeqMachine};
use crat::{test_transaction, debug_transaction};
use reg::{perform_regression};

fn main() {
    let matches = clap_app!(gaslighter =>
        (version: "0.1")
        (author: "Ethereum Classic Contributors")
        (about: "Gaslighter - Tests the Ethereum Classic Virtual Machine in 5 different ways.")
        (@arg KEEP_GOING: -k --keep_going "Don't exit the program even if a test fails.")
        (@subcommand crat =>
            (about: "Execute the ethereumpoject/tests JSON files. The emphasis is on using rust crates directly there is no FFI nor socket activity.")
            (@arg FILE: -f --file +takes_value +required "ethereumproject/tests JSON file to run for this test")
            (@arg TEST: -t --test +takes_value "test to run in the given file")
        )
        (@subcommand reg =>
            (about: "Performs an regression test on the entire Ethereum Classic blockchain.\n\nSteps to reproduce:\n* Install Parity 1.4.10 with this command: `$ cargo install --git https://github.com/paritytech/parity.git parity`.\n* Run Parity with this command: `[~/.cargo/bin]$ ./parity --chain classic --db-path /path/to/regression/dir`.\n* Wait for the chain to sync.\n* <ctrl-c>\n* Run this command: `$ cargo run --bin gaslighter -- -k reg -c /path/to/regression/dir/classic/db/906a34e69aec8c0d/`")
            (@arg CHAINDATA: -c --chaindata +takes_value +required "Path to parity's `chaindata` folder. e.g. `-c /path/to/regression/dir/classic/db/906a34e69aec8c0d/`, note the 906a34e69aec8c0d will probably be different.")
        )
        (@subcommand cratedb =>
            (version: "0.1.0")
            (about: "Use the JSON schema to run a debug session, where machine inner state can be inspected.")
            (@arg FILE: -f --file +takes_value +required "ethereumproject/tests JSON file to run for this test")
            (@arg TEST: -t --test +takes_value +required "test to run in the given file"))
        (@subcommand srv =>
            (about: "Allows SputnikVM to be run as a service.")
        )
    ).get_matches();
    let mut has_all_ffi_tests_passed = true;
    let mut has_all_crat_tests_passed = true;
    let mut has_regression_test_passed = true;
    let keep_going = if matches.is_present("KEEP_GOING") { true } else { false };
    if let Some(ref matches) = matches.subcommand_matches("crat") {
        let path = Path::new(matches.value_of("FILE").unwrap());
        let file = File::open(&path).unwrap();
        let reader = BufReader::new(file);
        let json: Value = serde_json::from_reader(reader).unwrap();

        match matches.value_of("TEST") {
            Some(test) => {
                test_transaction(test, &json[test], true);
            },
            None => {
                let mut failed = 0;
                for (test, data) in json.as_object().unwrap() {
                    if !test_transaction(test, &data, false) {
                        failed = failed + 1;
                    }
                }
                println!("\n{} failed.", failed);
            },
        }
    }
    if let Some(ref matches) = matches.subcommand_matches("reg") {
        let path = matches.value_of("CHAINDATA").unwrap();
        has_regression_test_passed = reg::perform_regression(path);
    }
    if let Some(ref matches) = matches.subcommand_matches("cratedb") {
        let path = Path::new(matches.value_of("FILE").unwrap());
        let file = File::open(&path).unwrap();
        let reader = BufReader::new(file);
        let json: Value = serde_json::from_reader(reader).unwrap();

        match matches.value_of("TEST") {
            Some(test) => {
                debug_transaction(&json[test]);
            },
            None => {
                panic!()
            },
        }
    }
    if has_all_ffi_tests_passed || has_regression_test_passed {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
