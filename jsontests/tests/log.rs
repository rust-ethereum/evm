#![allow(non_snake_case)]

extern crate jsontests;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use serde_json::Value;
use jsontests::test_transaction;

lazy_static! {
    static ref TESTS: Value =
        serde_json::from_str(include_str!("../res/files/vmLogTest.json")).unwrap();
}

#[test] fn log0_emptyMem() { assert_eq!(test_transaction("log0_emptyMem", &TESTS["log0_emptyMem"], true), true); }
#[test] fn log0_logMemStartTooHigh() { assert_eq!(test_transaction("log0_logMemStartTooHigh", &TESTS["log0_logMemStartTooHigh"], true), true); }
#[test] fn log0_logMemsizeTooHigh() { assert_eq!(test_transaction("log0_logMemsizeTooHigh", &TESTS["log0_logMemsizeTooHigh"], true), true); }
#[test] fn log0_logMemsizeZero() { assert_eq!(test_transaction("log0_logMemsizeZero", &TESTS["log0_logMemsizeZero"], true), true); }
#[test] fn log0_nonEmptyMem() { assert_eq!(test_transaction("log0_nonEmptyMem", &TESTS["log0_nonEmptyMem"], true), true); }
#[test] fn log0_nonEmptyMem_logMemSize1() { assert_eq!(test_transaction("log0_nonEmptyMem_logMemSize1", &TESTS["log0_nonEmptyMem_logMemSize1"], true), true); }
#[test] fn log0_nonEmptyMem_logMemSize1_logMemStart31() { assert_eq!(test_transaction("log0_nonEmptyMem_logMemSize1_logMemStart31", &TESTS["log0_nonEmptyMem_logMemSize1_logMemStart31"], true), true); }
#[test] fn log1_Caller() { assert_eq!(test_transaction("log1_Caller", &TESTS["log1_Caller"], true), true); }
#[test] fn log1_MaxTopic() { assert_eq!(test_transaction("log1_MaxTopic", &TESTS["log1_MaxTopic"], true), true); }
#[test] fn log1_emptyMem() { assert_eq!(test_transaction("log1_emptyMem", &TESTS["log1_emptyMem"], true), true); }
#[test] fn log1_logMemStartTooHigh() { assert_eq!(test_transaction("log1_logMemStartTooHigh", &TESTS["log1_logMemStartTooHigh"], true), true); }
#[test] fn log1_logMemsizeTooHigh() { assert_eq!(test_transaction("log1_logMemsizeTooHigh", &TESTS["log1_logMemsizeTooHigh"], true), true); }
#[test] fn log1_logMemsizeZero() { assert_eq!(test_transaction("log1_logMemsizeZero", &TESTS["log1_logMemsizeZero"], true), true); }
#[test] fn log1_nonEmptyMem() { assert_eq!(test_transaction("log1_nonEmptyMem", &TESTS["log1_nonEmptyMem"], true), true); }
#[test] fn log1_nonEmptyMem_logMemSize1() { assert_eq!(test_transaction("log1_nonEmptyMem_logMemSize1", &TESTS["log1_nonEmptyMem_logMemSize1"], true), true); }
#[test] fn log1_nonEmptyMem_logMemSize1_logMemStart31() { assert_eq!(test_transaction("log1_nonEmptyMem_logMemSize1_logMemStart31", &TESTS["log1_nonEmptyMem_logMemSize1_logMemStart31"], true), true); }
#[test] fn log2_Caller() { assert_eq!(test_transaction("log2_Caller", &TESTS["log2_Caller"], true), true); }
#[test] fn log2_MaxTopic() { assert_eq!(test_transaction("log2_MaxTopic", &TESTS["log2_MaxTopic"], true), true); }
#[test] fn log2_emptyMem() { assert_eq!(test_transaction("log2_emptyMem", &TESTS["log2_emptyMem"], true), true); }
#[test] fn log2_logMemStartTooHigh() { assert_eq!(test_transaction("log2_logMemStartTooHigh", &TESTS["log2_logMemStartTooHigh"], true), true); }
#[test] fn log2_logMemsizeTooHigh() { assert_eq!(test_transaction("log2_logMemsizeTooHigh", &TESTS["log2_logMemsizeTooHigh"], true), true); }
#[test] fn log2_logMemsizeZero() { assert_eq!(test_transaction("log2_logMemsizeZero", &TESTS["log2_logMemsizeZero"], true), true); }
#[test] fn log2_nonEmptyMem() { assert_eq!(test_transaction("log2_nonEmptyMem", &TESTS["log2_nonEmptyMem"], true), true); }
#[test] fn log2_nonEmptyMem_logMemSize1() { assert_eq!(test_transaction("log2_nonEmptyMem_logMemSize1", &TESTS["log2_nonEmptyMem_logMemSize1"], true), true); }
#[test] fn log2_nonEmptyMem_logMemSize1_logMemStart31() { assert_eq!(test_transaction("log2_nonEmptyMem_logMemSize1_logMemStart31", &TESTS["log2_nonEmptyMem_logMemSize1_logMemStart31"], true), true); }
#[test] fn log3_Caller() { assert_eq!(test_transaction("log3_Caller", &TESTS["log3_Caller"], true), true); }
#[test] fn log3_MaxTopic() { assert_eq!(test_transaction("log3_MaxTopic", &TESTS["log3_MaxTopic"], true), true); }
#[test] fn log3_PC() { assert_eq!(test_transaction("log3_PC", &TESTS["log3_PC"], true), true); }
#[test] fn log3_emptyMem() { assert_eq!(test_transaction("log3_emptyMem", &TESTS["log3_emptyMem"], true), true); }
#[test] fn log3_logMemStartTooHigh() { assert_eq!(test_transaction("log3_logMemStartTooHigh", &TESTS["log3_logMemStartTooHigh"], true), true); }
#[test] fn log3_logMemsizeTooHigh() { assert_eq!(test_transaction("log3_logMemsizeTooHigh", &TESTS["log3_logMemsizeTooHigh"], true), true); }
#[test] fn log3_logMemsizeZero() { assert_eq!(test_transaction("log3_logMemsizeZero", &TESTS["log3_logMemsizeZero"], true), true); }
#[test] fn log3_nonEmptyMem() { assert_eq!(test_transaction("log3_nonEmptyMem", &TESTS["log3_nonEmptyMem"], true), true); }
#[test] fn log3_nonEmptyMem_logMemSize1() { assert_eq!(test_transaction("log3_nonEmptyMem_logMemSize1", &TESTS["log3_nonEmptyMem_logMemSize1"], true), true); }
#[test] fn log3_nonEmptyMem_logMemSize1_logMemStart31() { assert_eq!(test_transaction("log3_nonEmptyMem_logMemSize1_logMemStart31", &TESTS["log3_nonEmptyMem_logMemSize1_logMemStart31"], true), true); }
#[test] fn log4_Caller() { assert_eq!(test_transaction("log4_Caller", &TESTS["log4_Caller"], true), true); }
#[test] fn log4_MaxTopic() { assert_eq!(test_transaction("log4_MaxTopic", &TESTS["log4_MaxTopic"], true), true); }
#[test] fn log4_PC() { assert_eq!(test_transaction("log4_PC", &TESTS["log4_PC"], true), true); }
#[test] fn log4_emptyMem() { assert_eq!(test_transaction("log4_emptyMem", &TESTS["log4_emptyMem"], true), true); }
#[test] fn log4_logMemStartTooHigh() { assert_eq!(test_transaction("log4_logMemStartTooHigh", &TESTS["log4_logMemStartTooHigh"], true), true); }
#[test] fn log4_logMemsizeTooHigh() { assert_eq!(test_transaction("log4_logMemsizeTooHigh", &TESTS["log4_logMemsizeTooHigh"], true), true); }
#[test] fn log4_logMemsizeZero() { assert_eq!(test_transaction("log4_logMemsizeZero", &TESTS["log4_logMemsizeZero"], true), true); }
#[test] fn log4_nonEmptyMem() { assert_eq!(test_transaction("log4_nonEmptyMem", &TESTS["log4_nonEmptyMem"], true), true); }
#[test] fn log4_nonEmptyMem_logMemSize1() { assert_eq!(test_transaction("log4_nonEmptyMem_logMemSize1", &TESTS["log4_nonEmptyMem_logMemSize1"], true), true); }
#[test] fn log4_nonEmptyMem_logMemSize1_logMemStart31() { assert_eq!(test_transaction("log4_nonEmptyMem_logMemSize1_logMemStart31", &TESTS["log4_nonEmptyMem_logMemSize1_logMemStart31"], true), true); }
#[test] fn log_2logs() { assert_eq!(test_transaction("log_2logs", &TESTS["log_2logs"], true), true); }
