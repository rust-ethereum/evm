#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate sputnikvm;

use sputnikvm::{read_hex, Gas};
use sputnikvm::vm::{Machine, FakeVectorMachine};

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use log::LogLevel;

fn main() {
    let matches = clap_app!(svm =>
        (version: "0.1.0")
        (author: "SputnikVM Contributors")
        (about: "SputnikVM - Ethereum Classic Virtual Machine")
        (@arg GAS: -g --gas +takes_value +required "Sets the gas amount")
        (@arg DATA: -d --data +takes_value +required "Sets the data needed")
        (@arg CODE: -c --code +takes_value +required "Sets the path to a file which contains the vm byte code")
        (@arg STATS: -s --stats "Return statistics on the execution")
        (@arg debug: -D --debug ... "Sets the level of debugging information")
    ).get_matches();

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
    if log_enabled!(LogLevel::Info) {
        info!("gas used: {:?}", machine.used_gas());
    }
}
