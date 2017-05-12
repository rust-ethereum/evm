#![allow(non_snake_case)]

extern crate jsontests;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use serde_json::Value;
use jsontests::test_transaction;

lazy_static! {
    static ref TESTS: Value =
        serde_json::from_str(include_str!("files/vmSystemOperationsTest.json")).unwrap();
}

#[test] fn aBAcalls0() { assert_eq!(test_transaction("ABAcalls0", &TESTS["ABAcalls0"], true), true); }
#[test] fn aBAcalls1() { assert_eq!(test_transaction("ABAcalls1", &TESTS["ABAcalls1"], true), true); }
#[test] fn aBAcalls2() { assert_eq!(test_transaction("ABAcalls2", &TESTS["ABAcalls2"], true), true); }
// #[test] fn aBAcalls3() { assert_eq!(test_transaction("ABAcalls3", &TESTS["ABAcalls3"], true), true); }
// #[test] fn aBAcallsSuicide0() { assert_eq!(test_transaction("ABAcallsSuicide0", &TESTS["ABAcallsSuicide0"], true), true); }
// #[test] fn aBAcallsSuicide1() { assert_eq!(test_transaction("ABAcallsSuicide1", &TESTS["ABAcallsSuicide1"], true), true); }
// #[test] fn callRecursiveBomb0() { assert_eq!(test_transaction("CallRecursiveBomb0", &TESTS["CallRecursiveBomb0"], true), true); }
// #[test] fn callRecursiveBomb1() { assert_eq!(test_transaction("CallRecursiveBomb1", &TESTS["CallRecursiveBomb1"], true), true); }
// #[test] fn callRecursiveBomb2() { assert_eq!(test_transaction("CallRecursiveBomb2", &TESTS["CallRecursiveBomb2"], true), true); }
// #[test] fn callRecursiveBomb3() { assert_eq!(test_transaction("CallRecursiveBomb3", &TESTS["CallRecursiveBomb3"], true), true); }
// #[test] fn callToNameRegistrator0() { assert_eq!(test_transaction("CallToNameRegistrator0", &TESTS["CallToNameRegistrator0"], true), true); }
#[test] fn callToNameRegistratorNotMuchMemory0() { assert_eq!(test_transaction("CallToNameRegistratorNotMuchMemory0", &TESTS["CallToNameRegistratorNotMuchMemory0"], true), true); }
#[test] fn callToNameRegistratorNotMuchMemory1() { assert_eq!(test_transaction("CallToNameRegistratorNotMuchMemory1", &TESTS["CallToNameRegistratorNotMuchMemory1"], true), true); }
#[test] fn callToNameRegistratorOutOfGas() { assert_eq!(test_transaction("CallToNameRegistratorOutOfGas", &TESTS["CallToNameRegistratorOutOfGas"], true), true); }
#[test] fn callToNameRegistratorTooMuchMemory0() { assert_eq!(test_transaction("CallToNameRegistratorTooMuchMemory0", &TESTS["CallToNameRegistratorTooMuchMemory0"], true), true); }
#[test] fn callToNameRegistratorTooMuchMemory1() { assert_eq!(test_transaction("CallToNameRegistratorTooMuchMemory1", &TESTS["CallToNameRegistratorTooMuchMemory1"], true), true); }
#[test] fn callToNameRegistratorTooMuchMemory2() { assert_eq!(test_transaction("CallToNameRegistratorTooMuchMemory2", &TESTS["CallToNameRegistratorTooMuchMemory2"], true), true); }
// #[test] fn callToPrecompiledContract() { assert_eq!(test_transaction("CallToPrecompiledContract", &TESTS["CallToPrecompiledContract"], true), true); }
// #[test] fn callToReturn1() { assert_eq!(test_transaction("CallToReturn1", &TESTS["CallToReturn1"], true), true); }
// #[test] fn postToNameRegistrator0() { assert_eq!(test_transaction("PostToNameRegistrator0", &TESTS["PostToNameRegistrator0"], true), true); }
// #[test] fn postToReturn1() { assert_eq!(test_transaction("PostToReturn1", &TESTS["PostToReturn1"], true), true); }
#[test] fn testNameRegistrator() { assert_eq!(test_transaction("TestNameRegistrator", &TESTS["TestNameRegistrator"], true), true); }
// #[test] fn callcodeToNameRegistrator0() { assert_eq!(test_transaction("callcodeToNameRegistrator0", &TESTS["callcodeToNameRegistrator0"], true), true); }
// #[test] fn callcodeToReturn1() { assert_eq!(test_transaction("callcodeToReturn1", &TESTS["callcodeToReturn1"], true), true); }
#[test] fn callstatelessToNameRegistrator0() { assert_eq!(test_transaction("callstatelessToNameRegistrator0", &TESTS["callstatelessToNameRegistrator0"], true), true); }
#[test] fn callstatelessToReturn1() { assert_eq!(test_transaction("callstatelessToReturn1", &TESTS["callstatelessToReturn1"], true), true); }
// #[test] fn createNameRegistrator() { assert_eq!(test_transaction("createNameRegistrator", &TESTS["createNameRegistrator"], true), true); }
#[test] fn createNameRegistratorOutOfMemoryBonds0() { assert_eq!(test_transaction("createNameRegistratorOutOfMemoryBonds0", &TESTS["createNameRegistratorOutOfMemoryBonds0"], true), true); }
#[test] fn createNameRegistratorOutOfMemoryBonds1() { assert_eq!(test_transaction("createNameRegistratorOutOfMemoryBonds1", &TESTS["createNameRegistratorOutOfMemoryBonds1"], true), true); }
#[test] fn createNameRegistratorValueTooHigh() { assert_eq!(test_transaction("createNameRegistratorValueTooHigh", &TESTS["createNameRegistratorValueTooHigh"], true), true); }
#[test] fn return0() { assert_eq!(test_transaction("return0", &TESTS["return0"], true), true); }
#[test] fn return1() { assert_eq!(test_transaction("return1", &TESTS["return1"], true), true); }
#[test] fn return2() { assert_eq!(test_transaction("return2", &TESTS["return2"], true), true); }
#[test] fn suicide0() { assert_eq!(test_transaction("suicide0", &TESTS["suicide0"], true), true); }
#[test] fn suicideNotExistingAccount() { assert_eq!(test_transaction("suicideNotExistingAccount", &TESTS["suicideNotExistingAccount"], true), true); }
#[test] fn suicideSendEtherToMe() { assert_eq!(test_transaction("suicideSendEtherToMe", &TESTS["suicideSendEtherToMe"], true), true); }
