use serde_json;
use sputnikvm;

use sputnikvm::{read_hex, Gas, M256, Address};
use sputnikvm::vm::{Machine, VectorMachine};
use sputnikvm::blockchain::Block;
use sputnikvm::transaction::{Transaction, VectorTransaction};

use super::json_schema::{create_machine, test_machine};

use serde_json::{Value, Error};
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Write, stdout};
use std::str::FromStr;

pub fn test_transaction(name: &str, v: &Value) {
    print!("Testing {} ...", name);
    stdout().flush();

    let mut machine = create_machine(v);

    let out = v["out"].as_str();

    if out.is_some() {
        machine.fire();
        if test_machine(v, &machine) {
            println!(" OK");
        } else {
            println!(" Failed");
        }
    } else {
        println!(" OK (no out value)");
    }
}
