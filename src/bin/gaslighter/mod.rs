#[macro_use]
extern crate clap;
extern crate sputnikvm;
extern crate libloading;
extern crate libc;
extern crate serde_json;
extern crate rustyline;
extern crate rocksdb;

mod reg;
mod ffi;
mod crat;
mod json_schema;

use serde_json::{Value, Error};
use std::process;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Write, stdout};

use sputnikvm::{read_hex, Gas};
use sputnikvm::vm::{Machine, FakeVectorMachine};
use crat::{test_transaction, debug_transaction};
use ffi::{test_ffi_transaction};
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
        (@subcommand cli =>
            (version: "0.1.0")
            (about: "Provides a command line interface for accessing SputnikVM.")
            (@arg GAS: -g --gas +takes_value +required "Sets the gas amount")
            (@arg DATA: -d --data +takes_value +required "Sets the data needed")
            (@arg CODE: -c --code +takes_value +required "Sets the path to a file which contains the vm byte code")
        )
        (@subcommand cratedb =>
            (version: "0.1.0")
            (about: "Use the JSON schema to run a debug session, where machine inner state can be inspected.")
            (@arg FILE: -f --file +takes_value +required "ethereumproject/tests JSON file to run for this test")
            (@arg TEST: -t --test +takes_value +required "test to run in the given file"))
        (@subcommand srv =>
            (about: "Allows SputnikVM to be run as a service.")
        )
        (@subcommand ffi =>
            (about: "Executes the ethereumproject/tests JSON files over Foreign Function Interface.")
            (version: "0.1")
            (@arg FILE: -f --file +takes_value +required "ethereumproject/tests JSON file to run for this test")
            (@arg TEST: -t --test +takes_value "test to run in the given file")
            (@arg SPUTNIKVMSO_PATH: -s --sputnikvm_path +takes_value +required "Path to libsputnikvm.so, typically it is `-s target/release/libsputnikvm.so`")
        )
    ).get_matches();
    let mut has_all_ffi_tests_passed = true;
    let mut has_all_crat_tests_passed = true;
    let mut has_regression_test_passed = true;
    let keep_going = if matches.is_present("KEEP_GOING") { true } else { false };
    if let Some(ref matches) = matches.subcommand_matches("cli") {
        let code_hex = read_hex(match matches.value_of("CODE") {
            Some(c) => c,
            None => "",
        });
        let code = code_hex.expect("code must be provided");
        let initial_gas = (value_t!(matches, "GAS", usize).unwrap_or(0xff)).into();
        let data = match matches.value_of("DATA") {
            Some(d) => d.as_bytes().into(),
            None => "".as_bytes().into(),
        };
        let mut machine = FakeVectorMachine::fake(code.as_slice(), data, initial_gas);
        machine.fire();
    }
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
                for (test, data) in json.as_object().unwrap() {
                    test_transaction(test, &data, false);
                }
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
    if let Some(ref matches) = matches.subcommand_matches("ffi") {
        let path = Path::new(matches.value_of("FILE").unwrap());
        let file = File::open(&path).unwrap();
        let reader = BufReader::new(file);
        let json: Value = serde_json::from_reader(reader).unwrap();
        let sputnikvm_path = match matches.value_of("SPUTNIKVMSO_PATH") {
            Some(c) => c,
            None => "",
        };
        match matches.value_of("TEST") {
            Some(test) => {
                test_ffi_transaction(test, &json[test], true, sputnikvm_path);
            },
            None => {
                for (test, data) in json.as_object().unwrap() {
                    test_ffi_transaction(test, &data, false, sputnikvm_path);
                }
            },
        }
    }
    if has_all_ffi_tests_passed || has_regression_test_passed {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
