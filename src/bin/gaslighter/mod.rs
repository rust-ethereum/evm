#[macro_use]
extern crate clap;
extern crate sputnikvm;
extern crate capnp;
extern crate libloading;
extern crate libc;
extern crate serde_json;
extern crate rustyline;

mod hierarchy_capnp;
mod vm_capnp;
mod test_capnp;
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

fn main() {
    let matches = clap_app!(gaslighter =>
        (version: "0.1")
        (author: "Ethereum Classic Contributors")
        (about: "Gaslighter - Tests the Ethereum Classic Virtual Machine in 5 different ways.")
        (@arg KEEP_GOING: -k --keep_going "Don't exit the program even if a test fails.")
        (@subcommand reg =>
            (about: "Performs a regression test by executing the entire ETC blockchain in this mode of execution.")
        )
        (@subcommand crat =>
            (about: "Execute the ethereumpoject/tests JSON files. The emphasis is on using rust crates directly there is no FFI nor socket activity.")
            (@arg FILE: -f --file +takes_value +required "ethereumproject/tests JSON file to run for this test")
            (@arg TEST: -t --test +takes_value "test to run in the given file")
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
            (about: "Tests Foreign Function Interfacing (FFI) in this mode of execution. Capnproto schema define the input and output of the vm. Gaslighter also provide callbacks over the ABI which mock the blockchain and retrieving data about transactions.")
            (version: "0.1")
            (@arg PATH_TO_CAPNPROTO_TYPECHECKED_TEST_BIN: -t --capnp_test_bin +takes_value +required "Path to a type checked binary compiled by the capnp tool. The source of this artefact is in the tests directory. Please run `$ capnp eval -b tests/mod.capnp all > tests.bin` in the root directory to generate the binary.")
            (@arg SPUTNIKVMSO_PATH: -s --sputnikvm_path +takes_value +required "Path to libsputnikvm.so, typically it is `-s target/release/libsputnikvm.so`")
            (@arg TESTS_TO_RUN: -r --run_test +takes_value +required "The format is [directory]/[file]/[test] e.g. `--run_test arith/add/add1` will run the arith/add/add1 test, `--run_test arith/add/` will run every test in the tests/arith/add.capnp file. Likewise `--run_test arith//` will run every test in every file of the `arith` directory. Lastly `--run_test //` will run every single test available.")
        )
    ).get_matches();
    let mut has_all_ffi_tests_passed = true;
    let keep_going = if matches.is_present("KEEP_GOING") { true } else { false };
    if let Some(ref matches) = matches.subcommand_matches("cli") {
        let code_hex = read_hex(match matches.value_of("CODE") {
            Some(c) => c,
            None => "",
        });
        let code = code_hex.expect("code must be provided");
        let initial_gas = (value_t!(matches, "GAS", isize).unwrap_or(0xff)).into();
        let data = match matches.value_of("DATA") {
            Some(d) => d.as_bytes().into(),
            None => "".as_bytes().into(),
        };
        let mut machine = FakeVectorMachine::fake(code.as_slice(), data, initial_gas);
        machine.fire();
        println!("gas used: {:?}", machine.used_gas());
    }
    if let Some(ref matches) = matches.subcommand_matches("crat") {
        let path = Path::new(matches.value_of("FILE").unwrap());
        let file = File::open(&path).unwrap();
        let reader = BufReader::new(file);
        let json: Value = serde_json::from_reader(reader).unwrap();

        match matches.value_of("TEST") {
            Some(test) => {
                test_transaction(test, &json[test]);
            },
            None => {
                for (test, data) in json.as_object().unwrap() {
                    test_transaction(test, &data);
                }
            },
        }
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
        let capnp_test_bin = match matches.value_of("PATH_TO_CAPNPROTO_TYPECHECKED_TEST_BIN") {
            Some(c) => c,
            None => "",
        };
        let test_to_run = match matches.value_of("TESTS_TO_RUN") {
            Some(c) => c,
            None => "",
        };
        let sputnikvm_path = match matches.value_of("SPUTNIKVMSO_PATH") {
            Some(c) => c,
            None => "",
        };
        let path = Path::new(capnp_test_bin);
        let display = path.display();
        let file = match File::open(&path) {
            Err(_) => panic!("couldn't open {}", display),
            Ok(file) => file,
        };
        if ffi::execute(file, test_to_run, sputnikvm_path, keep_going){
            has_all_ffi_tests_passed = true;
        } else {
            has_all_ffi_tests_passed = false;
        }
    }
    if has_all_ffi_tests_passed {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
