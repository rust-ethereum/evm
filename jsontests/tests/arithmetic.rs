extern crate jsontests;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use serde_json::Value;
use jsontests::test_transaction;

lazy_static! {
    static ref TESTS: Value =
        serde_json::from_str(include_str!("files/vmArithmeticTest.json")).unwrap();
}

#[test]
fn add0() { assert_eq!(test_transaction("add0", &TESTS["add0"], true), true); }
