#![allow(non_snake_case)]

extern crate jsontests;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use serde_json::Value;
use jsontests::test_transaction;

lazy_static! {
    static ref TESTS: Value =
        serde_json::from_str(include_str!("../res/files/vmPushDupSwapTest.json")).unwrap();
}

#[test] fn dup1() { assert_eq!(test_transaction("dup1", &TESTS["dup1"], true), true); }
#[test] fn dup10() { assert_eq!(test_transaction("dup10", &TESTS["dup10"], true), true); }
#[test] fn dup11() { assert_eq!(test_transaction("dup11", &TESTS["dup11"], true), true); }
#[test] fn dup12() { assert_eq!(test_transaction("dup12", &TESTS["dup12"], true), true); }
#[test] fn dup13() { assert_eq!(test_transaction("dup13", &TESTS["dup13"], true), true); }
#[test] fn dup14() { assert_eq!(test_transaction("dup14", &TESTS["dup14"], true), true); }
#[test] fn dup15() { assert_eq!(test_transaction("dup15", &TESTS["dup15"], true), true); }
#[test] fn dup16() { assert_eq!(test_transaction("dup16", &TESTS["dup16"], true), true); }
#[test] fn dup2() { assert_eq!(test_transaction("dup2", &TESTS["dup2"], true), true); }
#[test] fn dup2error() { assert_eq!(test_transaction("dup2error", &TESTS["dup2error"], true), true); }
#[test] fn dup3() { assert_eq!(test_transaction("dup3", &TESTS["dup3"], true), true); }
#[test] fn dup4() { assert_eq!(test_transaction("dup4", &TESTS["dup4"], true), true); }
#[test] fn dup5() { assert_eq!(test_transaction("dup5", &TESTS["dup5"], true), true); }
#[test] fn dup6() { assert_eq!(test_transaction("dup6", &TESTS["dup6"], true), true); }
#[test] fn dup7() { assert_eq!(test_transaction("dup7", &TESTS["dup7"], true), true); }
#[test] fn dup8() { assert_eq!(test_transaction("dup8", &TESTS["dup8"], true), true); }
#[test] fn dup9() { assert_eq!(test_transaction("dup9", &TESTS["dup9"], true), true); }
#[test] fn push1() { assert_eq!(test_transaction("push1", &TESTS["push1"], true), true); }
#[test] fn push10() { assert_eq!(test_transaction("push10", &TESTS["push10"], true), true); }
#[test] fn push11() { assert_eq!(test_transaction("push11", &TESTS["push11"], true), true); }
#[test] fn push12() { assert_eq!(test_transaction("push12", &TESTS["push12"], true), true); }
#[test] fn push13() { assert_eq!(test_transaction("push13", &TESTS["push13"], true), true); }
#[test] fn push14() { assert_eq!(test_transaction("push14", &TESTS["push14"], true), true); }
#[test] fn push15() { assert_eq!(test_transaction("push15", &TESTS["push15"], true), true); }
#[test] fn push16() { assert_eq!(test_transaction("push16", &TESTS["push16"], true), true); }
#[test] fn push17() { assert_eq!(test_transaction("push17", &TESTS["push17"], true), true); }
#[test] fn push18() { assert_eq!(test_transaction("push18", &TESTS["push18"], true), true); }
#[test] fn push19() { assert_eq!(test_transaction("push19", &TESTS["push19"], true), true); }
#[test] fn push1_missingStack() { assert_eq!(test_transaction("push1_missingStack", &TESTS["push1_missingStack"], true), true); }
#[test] fn push2() { assert_eq!(test_transaction("push2", &TESTS["push2"], true), true); }
#[test] fn push20() { assert_eq!(test_transaction("push20", &TESTS["push20"], true), true); }
#[test] fn push21() { assert_eq!(test_transaction("push21", &TESTS["push21"], true), true); }
#[test] fn push22() { assert_eq!(test_transaction("push22", &TESTS["push22"], true), true); }
#[test] fn push23() { assert_eq!(test_transaction("push23", &TESTS["push23"], true), true); }
#[test] fn push24() { assert_eq!(test_transaction("push24", &TESTS["push24"], true), true); }
#[test] fn push25() { assert_eq!(test_transaction("push25", &TESTS["push25"], true), true); }
#[test] fn push26() { assert_eq!(test_transaction("push26", &TESTS["push26"], true), true); }
#[test] fn push27() { assert_eq!(test_transaction("push27", &TESTS["push27"], true), true); }
#[test] fn push28() { assert_eq!(test_transaction("push28", &TESTS["push28"], true), true); }
#[test] fn push29() { assert_eq!(test_transaction("push29", &TESTS["push29"], true), true); }
#[test] fn push3() { assert_eq!(test_transaction("push3", &TESTS["push3"], true), true); }
#[test] fn push30() { assert_eq!(test_transaction("push30", &TESTS["push30"], true), true); }
#[test] fn push31() { assert_eq!(test_transaction("push31", &TESTS["push31"], true), true); }
#[test] fn push32() { assert_eq!(test_transaction("push32", &TESTS["push32"], true), true); }
#[test] fn push32AndSuicide() { assert_eq!(test_transaction("push32AndSuicide", &TESTS["push32AndSuicide"], true), true); }
#[test] fn push32FillUpInputWithZerosAtTheEnd() { assert_eq!(test_transaction("push32FillUpInputWithZerosAtTheEnd", &TESTS["push32FillUpInputWithZerosAtTheEnd"], true), true); }
#[test] fn push32Undefined() { assert_eq!(test_transaction("push32Undefined", &TESTS["push32Undefined"], true), true); }
#[test] fn push32Undefined2() { assert_eq!(test_transaction("push32Undefined2", &TESTS["push32Undefined2"], true), true); }
#[test] fn push33() { assert_eq!(test_transaction("push33", &TESTS["push33"], true), true); }
#[test] fn push4() { assert_eq!(test_transaction("push4", &TESTS["push4"], true), true); }
#[test] fn push5() { assert_eq!(test_transaction("push5", &TESTS["push5"], true), true); }
#[test] fn push6() { assert_eq!(test_transaction("push6", &TESTS["push6"], true), true); }
#[test] fn push7() { assert_eq!(test_transaction("push7", &TESTS["push7"], true), true); }
#[test] fn push8() { assert_eq!(test_transaction("push8", &TESTS["push8"], true), true); }
#[test] fn push9() { assert_eq!(test_transaction("push9", &TESTS["push9"], true), true); }
#[test] fn swap1() { assert_eq!(test_transaction("swap1", &TESTS["swap1"], true), true); }
#[test] fn swap10() { assert_eq!(test_transaction("swap10", &TESTS["swap10"], true), true); }
#[test] fn swap11() { assert_eq!(test_transaction("swap11", &TESTS["swap11"], true), true); }
#[test] fn swap12() { assert_eq!(test_transaction("swap12", &TESTS["swap12"], true), true); }
#[test] fn swap13() { assert_eq!(test_transaction("swap13", &TESTS["swap13"], true), true); }
#[test] fn swap14() { assert_eq!(test_transaction("swap14", &TESTS["swap14"], true), true); }
#[test] fn swap15() { assert_eq!(test_transaction("swap15", &TESTS["swap15"], true), true); }
#[test] fn swap16() { assert_eq!(test_transaction("swap16", &TESTS["swap16"], true), true); }
#[test] fn swap2() { assert_eq!(test_transaction("swap2", &TESTS["swap2"], true), true); }
#[test] fn swap2error() { assert_eq!(test_transaction("swap2error", &TESTS["swap2error"], true), true); }
#[test] fn swap3() { assert_eq!(test_transaction("swap3", &TESTS["swap3"], true), true); }
#[test] fn swap4() { assert_eq!(test_transaction("swap4", &TESTS["swap4"], true), true); }
#[test] fn swap5() { assert_eq!(test_transaction("swap5", &TESTS["swap5"], true), true); }
#[test] fn swap6() { assert_eq!(test_transaction("swap6", &TESTS["swap6"], true), true); }
#[test] fn swap7() { assert_eq!(test_transaction("swap7", &TESTS["swap7"], true), true); }
#[test] fn swap8() { assert_eq!(test_transaction("swap8", &TESTS["swap8"], true), true); }
#[test] fn swap9() { assert_eq!(test_transaction("swap9", &TESTS["swap9"], true), true); }
#[test] fn swapjump1() { assert_eq!(test_transaction("swapjump1", &TESTS["swapjump1"], true), true); }
