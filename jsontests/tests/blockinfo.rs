#![allow(non_snake_case)]

extern crate jsontests;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use serde_json::Value;
use jsontests::test_transaction;

lazy_static! {
    static ref TESTS: Value =
        serde_json::from_str(include_str!("../res/files/vmBlockInfoTest.json")).unwrap();
}

#[test] fn blockhash257Block() { assert_eq!(test_transaction("blockhash257Block", &TESTS["blockhash257Block"], true), true); }
#[test] fn blockhash258Block() { assert_eq!(test_transaction("blockhash258Block", &TESTS["blockhash258Block"], true), true); }
#[test] fn blockhashInRange() { assert_eq!(test_transaction("blockhashInRange", &TESTS["blockhashInRange"], true), true); }
#[test] fn blockhashMyBlock() { assert_eq!(test_transaction("blockhashMyBlock", &TESTS["blockhashMyBlock"], true), true); }
#[test] fn blockhashNotExistingBlock() { assert_eq!(test_transaction("blockhashNotExistingBlock", &TESTS["blockhashNotExistingBlock"], true), true); }
#[test] fn blockhashOutOfRange() { assert_eq!(test_transaction("blockhashOutOfRange", &TESTS["blockhashOutOfRange"], true), true); }
#[test] fn blockhashUnderFlow() { assert_eq!(test_transaction("blockhashUnderFlow", &TESTS["blockhashUnderFlow"], true), true); }
#[test] fn coinbase() { assert_eq!(test_transaction("coinbase", &TESTS["coinbase"], true), true); }
#[test] fn difficulty() { assert_eq!(test_transaction("difficulty", &TESTS["difficulty"], true), true); }
#[test] fn gaslimit() { assert_eq!(test_transaction("gaslimit", &TESTS["gaslimit"], true), true); }
#[test] fn number() { assert_eq!(test_transaction("number", &TESTS["number"], true), true); }
#[test] fn timestamp() { assert_eq!(test_transaction("timestamp", &TESTS["timestamp"], true), true); }
