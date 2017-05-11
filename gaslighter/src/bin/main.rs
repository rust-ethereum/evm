#[macro_use]
extern crate clap;
extern crate sputnikvm;
extern crate libloading;
extern crate libc;
extern crate serde_json;
extern crate rustyline;

mod reg;

use serde_json::{Value};
use std::process;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader};

use reg::{perform_regression};

fn main() {
    let matches = clap_app!(gaslighter =>
        (version: "0.1")
        (author: "Ethereum Classic Contributors")
        (about: "Gaslighter - Tests the Ethereum Classic Virtual Machine in 5 different ways.")
        (@arg KEEP_GOING: -k --keep_going "Don't exit the program even if a test fails.")
        (@subcommand reg =>
            (about: "Performs an regression test on the entire Ethereum Classic blockchain.\n\nSteps to reproduce:\n* Install Parity 1.4.10 with this command: `$ cargo install --git https://github.com/paritytech/parity.git parity`.\n* Run Parity with this command: `[~/.cargo/bin]$ ./parity --chain classic --db-path /path/to/regression/dir`.\n* Wait for the chain to sync.\n* <ctrl-c>\n* Run this command: `$ cargo run --bin gaslighter -- -k reg -c /path/to/regression/dir/classic/db/906a34e69aec8c0d/`")
            (@arg CHAINDATA: -c --chaindata +takes_value +required "Path to parity's `chaindata` folder. e.g. `-c /path/to/regression/dir/classic/db/906a34e69aec8c0d/`, note the 906a34e69aec8c0d will probably be different.")
        )
        (@subcommand srv =>
            (about: "Allows SputnikVM to be run as a service.")
        )
    ).get_matches();
    let mut has_regression_test_passed = true;
    let keep_going = if matches.is_present("KEEP_GOING") { true } else { false };
    if let Some(ref matches) = matches.subcommand_matches("reg") {
        let path = matches.value_of("CHAINDATA").unwrap();
        has_regression_test_passed = perform_regression(path);
    }
    if has_regression_test_passed {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
