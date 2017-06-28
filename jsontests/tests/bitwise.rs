#![allow(non_snake_case)]

extern crate jsontests;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use serde_json::Value;
use jsontests::test_transaction;

lazy_static! {
    static ref TESTS: Value =
        serde_json::from_str(include_str!("../res/files/vmBitwiseLogicOperationTest.json")).unwrap();
}

#[test] fn and0() { assert_eq!(test_transaction("and0", &TESTS["and0"], true), true); }
#[test] fn and1() { assert_eq!(test_transaction("and1", &TESTS["and1"], true), true); }
#[test] fn and2() { assert_eq!(test_transaction("and2", &TESTS["and2"], true), true); }
#[test] fn and3() { assert_eq!(test_transaction("and3", &TESTS["and3"], true), true); }
#[test] fn and4() { assert_eq!(test_transaction("and4", &TESTS["and4"], true), true); }
#[test] fn and5() { assert_eq!(test_transaction("and5", &TESTS["and5"], true), true); }
#[test] fn byte0() { assert_eq!(test_transaction("byte0", &TESTS["byte0"], true), true); }
#[test] fn byte1() { assert_eq!(test_transaction("byte1", &TESTS["byte1"], true), true); }
#[test] fn byte10() { assert_eq!(test_transaction("byte10", &TESTS["byte10"], true), true); }
#[test] fn byte11() { assert_eq!(test_transaction("byte11", &TESTS["byte11"], true), true); }
#[test] fn byte2() { assert_eq!(test_transaction("byte2", &TESTS["byte2"], true), true); }
#[test] fn byte3() { assert_eq!(test_transaction("byte3", &TESTS["byte3"], true), true); }
#[test] fn byte4() { assert_eq!(test_transaction("byte4", &TESTS["byte4"], true), true); }
#[test] fn byte5() { assert_eq!(test_transaction("byte5", &TESTS["byte5"], true), true); }
#[test] fn byte6() { assert_eq!(test_transaction("byte6", &TESTS["byte6"], true), true); }
#[test] fn byte7() { assert_eq!(test_transaction("byte7", &TESTS["byte7"], true), true); }
#[test] fn byte8() { assert_eq!(test_transaction("byte8", &TESTS["byte8"], true), true); }
#[test] fn byte9() { assert_eq!(test_transaction("byte9", &TESTS["byte9"], true), true); }
#[test] fn eq0() { assert_eq!(test_transaction("eq0", &TESTS["eq0"], true), true); }
#[test] fn eq1() { assert_eq!(test_transaction("eq1", &TESTS["eq1"], true), true); }
#[test] fn eq2() { assert_eq!(test_transaction("eq2", &TESTS["eq2"], true), true); }
#[test] fn gt0() { assert_eq!(test_transaction("gt0", &TESTS["gt0"], true), true); }
#[test] fn gt1() { assert_eq!(test_transaction("gt1", &TESTS["gt1"], true), true); }
#[test] fn gt2() { assert_eq!(test_transaction("gt2", &TESTS["gt2"], true), true); }
#[test] fn gt3() { assert_eq!(test_transaction("gt3", &TESTS["gt3"], true), true); }
#[test] fn iszeo2() { assert_eq!(test_transaction("iszeo2", &TESTS["iszeo2"], true), true); }
#[test] fn iszero0() { assert_eq!(test_transaction("iszero0", &TESTS["iszero0"], true), true); }
#[test] fn iszero1() { assert_eq!(test_transaction("iszero1", &TESTS["iszero1"], true), true); }
#[test] fn lt0() { assert_eq!(test_transaction("lt0", &TESTS["lt0"], true), true); }
#[test] fn lt1() { assert_eq!(test_transaction("lt1", &TESTS["lt1"], true), true); }
#[test] fn lt2() { assert_eq!(test_transaction("lt2", &TESTS["lt2"], true), true); }
#[test] fn lt3() { assert_eq!(test_transaction("lt3", &TESTS["lt3"], true), true); }
#[test] fn not0() { assert_eq!(test_transaction("not0", &TESTS["not0"], true), true); }
#[test] fn not1() { assert_eq!(test_transaction("not1", &TESTS["not1"], true), true); }
#[test] fn not2() { assert_eq!(test_transaction("not2", &TESTS["not2"], true), true); }
#[test] fn not3() { assert_eq!(test_transaction("not3", &TESTS["not3"], true), true); }
#[test] fn not4() { assert_eq!(test_transaction("not4", &TESTS["not4"], true), true); }
#[test] fn not5() { assert_eq!(test_transaction("not5", &TESTS["not5"], true), true); }
#[test] fn or0() { assert_eq!(test_transaction("or0", &TESTS["or0"], true), true); }
#[test] fn or1() { assert_eq!(test_transaction("or1", &TESTS["or1"], true), true); }
#[test] fn or2() { assert_eq!(test_transaction("or2", &TESTS["or2"], true), true); }
#[test] fn or3() { assert_eq!(test_transaction("or3", &TESTS["or3"], true), true); }
#[test] fn or4() { assert_eq!(test_transaction("or4", &TESTS["or4"], true), true); }
#[test] fn or5() { assert_eq!(test_transaction("or5", &TESTS["or5"], true), true); }
#[test] fn sgt0() { assert_eq!(test_transaction("sgt0", &TESTS["sgt0"], true), true); }
#[test] fn sgt1() { assert_eq!(test_transaction("sgt1", &TESTS["sgt1"], true), true); }
#[test] fn sgt2() { assert_eq!(test_transaction("sgt2", &TESTS["sgt2"], true), true); }
#[test] fn sgt3() { assert_eq!(test_transaction("sgt3", &TESTS["sgt3"], true), true); }
#[test] fn sgt4() { assert_eq!(test_transaction("sgt4", &TESTS["sgt4"], true), true); }
#[test] fn slt0() { assert_eq!(test_transaction("slt0", &TESTS["slt0"], true), true); }
#[test] fn slt1() { assert_eq!(test_transaction("slt1", &TESTS["slt1"], true), true); }
#[test] fn slt2() { assert_eq!(test_transaction("slt2", &TESTS["slt2"], true), true); }
#[test] fn slt3() { assert_eq!(test_transaction("slt3", &TESTS["slt3"], true), true); }
#[test] fn slt4() { assert_eq!(test_transaction("slt4", &TESTS["slt4"], true), true); }
#[test] fn xor0() { assert_eq!(test_transaction("xor0", &TESTS["xor0"], true), true); }
#[test] fn xor1() { assert_eq!(test_transaction("xor1", &TESTS["xor1"], true), true); }
#[test] fn xor2() { assert_eq!(test_transaction("xor2", &TESTS["xor2"], true), true); }
#[test] fn xor3() { assert_eq!(test_transaction("xor3", &TESTS["xor3"], true), true); }
#[test] fn xor4() { assert_eq!(test_transaction("xor4", &TESTS["xor4"], true), true); }
#[test] fn xor5() { assert_eq!(test_transaction("xor5", &TESTS["xor5"], true), true); }
