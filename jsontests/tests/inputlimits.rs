#![allow(non_snake_case)]

use evm::VMTestPatch;
use jsontests::test_transaction;
use serde_json::Value;

// Log format is broken for input limits tests

#[test]
#[ignore]
fn inputLimitsLight() {
    let TESTS: Value =
        serde_json::from_str(include_str!("../res/files/vmInputLimitsLight/vmInputLimitsLight.json")).unwrap();
    for (name, value) in TESTS.as_object().unwrap().iter() {
        print!("\t{} ... ", name);
        match test_transaction(name, VMTestPatch::default(), value, true) {
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
        match test_transaction(name, VMTestPatch::default(), value, true) {
            Ok(false) => panic!("test inputLimits::{} failed", name),
            _ => (),
        }
    }
}
