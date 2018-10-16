#![allow(non_snake_case)]

extern crate jsontests;
extern crate serde_json;
extern crate evm;

use serde_json::Value;
use jsontests::test_transaction;

// Log format is broken for input limits tests

#[test]
#[ignore]
fn inputLimitsLight() {
    let TESTS: Value = serde_json::from_str(include_str!("../res/files/vmInputLimitsLight/vmInputLimitsLight.json")).unwrap();
    for (name, value) in TESTS.as_object().unwrap().iter() {
        print!("\t{} ... ", name);
        match test_transaction::<evm::VMTestPatch>(name, value, true) {
            Ok(false) => panic!("test inputLimitsLight::{} failed", name),
            _ => (),
        }
    }
}

#[test]
#[ignore]
fn inputLimits() {
    let TESTS: Value = serde_json::from_str(include_str!("../res/files/vmInputLimits/vmInputLimits.json")).unwrap();
    for (name, value) in TESTS.as_object().unwrap().iter() {
        print!("\t{} ... ", name);
        match test_transaction::<evm::VMTestPatch>(name, value, true) {
            Ok(false) => panic!("test inputLimits::{} failed", name),
            _ => (),
        }
    }
}

