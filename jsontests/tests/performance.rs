#![allow(non_snake_case)]

extern crate jsontests;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use serde_json::Value;
use jsontests::test_transaction;

lazy_static! {
    static ref TESTS: Value =
        serde_json::from_str(include_str!("../res/files/vmPerformanceTest.json")).unwrap();
}

#[test] fn ackermann31() { assert_eq!(test_transaction("ackermann31", &TESTS["ackermann31"], true), true); }
#[test] fn ackermann32() { assert_eq!(test_transaction("ackermann32", &TESTS["ackermann32"], true), true); }
#[test] fn ackermann33() { assert_eq!(test_transaction("ackermann33", &TESTS["ackermann33"], true), true); }
#[test] fn fibonacci10() { assert_eq!(test_transaction("fibonacci10", &TESTS["fibonacci10"], true), true); }
#[test] fn fibonacci16() { assert_eq!(test_transaction("fibonacci16", &TESTS["fibonacci16"], true), true); }
#[test] fn manyFunctions100() { assert_eq!(test_transaction("manyFunctions100", &TESTS["manyFunctions100"], true), true); }
