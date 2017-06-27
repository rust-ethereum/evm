#![allow(non_snake_case)]

extern crate jsontests;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use serde_json::Value;
use jsontests::test_transaction;

lazy_static! {
    static ref TESTS: Value =
        serde_json::from_str(include_str!("../res/files/vmSha3Test.json")).unwrap();
}

#[test] fn sha3_0() { assert_eq!(test_transaction("sha3_0", &TESTS["sha3_0"], true), true); }
#[test] fn sha3_1() { assert_eq!(test_transaction("sha3_1", &TESTS["sha3_1"], true), true); }
#[test] fn sha3_2() { assert_eq!(test_transaction("sha3_2", &TESTS["sha3_2"], true), true); }
#[test] fn sha3_3() { assert_eq!(test_transaction("sha3_3", &TESTS["sha3_3"], true), true); }
#[test] fn sha3_4() { assert_eq!(test_transaction("sha3_4", &TESTS["sha3_4"], true), true); }
#[test] fn sha3_5() { assert_eq!(test_transaction("sha3_5", &TESTS["sha3_5"], true), true); }
#[test] fn sha3_6() { assert_eq!(test_transaction("sha3_6", &TESTS["sha3_6"], true), true); }
#[test] fn sha3_bigOffset() { assert_eq!(test_transaction("sha3_bigOffset", &TESTS["sha3_bigOffset"], true), true); }
#[test] fn sha3_bigOffset2() { assert_eq!(test_transaction("sha3_bigOffset2", &TESTS["sha3_bigOffset2"], true), true); }
#[test] fn sha3_bigSize() { assert_eq!(test_transaction("sha3_bigSize", &TESTS["sha3_bigSize"], true), true); }
#[test] fn sha3_memSizeNoQuadraticCost31() { assert_eq!(test_transaction("sha3_memSizeNoQuadraticCost31", &TESTS["sha3_memSizeNoQuadraticCost31"], true), true); }
#[test] fn sha3_memSizeQuadraticCost32() { assert_eq!(test_transaction("sha3_memSizeQuadraticCost32", &TESTS["sha3_memSizeQuadraticCost32"], true), true); }
#[test] fn sha3_memSizeQuadraticCost32_zeroSize() { assert_eq!(test_transaction("sha3_memSizeQuadraticCost32_zeroSize", &TESTS["sha3_memSizeQuadraticCost32_zeroSize"], true), true); }
#[test] fn sha3_memSizeQuadraticCost33() { assert_eq!(test_transaction("sha3_memSizeQuadraticCost33", &TESTS["sha3_memSizeQuadraticCost33"], true), true); }
#[test] fn sha3_memSizeQuadraticCost63() { assert_eq!(test_transaction("sha3_memSizeQuadraticCost63", &TESTS["sha3_memSizeQuadraticCost63"], true), true); }
#[test] fn sha3_memSizeQuadraticCost64() { assert_eq!(test_transaction("sha3_memSizeQuadraticCost64", &TESTS["sha3_memSizeQuadraticCost64"], true), true); }
#[test] fn sha3_memSizeQuadraticCost64_2() { assert_eq!(test_transaction("sha3_memSizeQuadraticCost64_2", &TESTS["sha3_memSizeQuadraticCost64_2"], true), true); }
#[test] fn sha3_memSizeQuadraticCost65() { assert_eq!(test_transaction("sha3_memSizeQuadraticCost65", &TESTS["sha3_memSizeQuadraticCost65"], true), true); }
