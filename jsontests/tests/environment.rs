#![allow(non_snake_case)]

extern crate jsontests;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use serde_json::Value;
use jsontests::test_transaction;

lazy_static! {
    static ref TESTS: Value =
        serde_json::from_str(include_str!("../res/files/vmEnvironmentalInfoTest.json")).unwrap();
}

#[test] fn extCodeSizeAddressInputTooBigLeftMyAddress() { assert_eq!(test_transaction("ExtCodeSizeAddressInputTooBigLeftMyAddress", &TESTS["ExtCodeSizeAddressInputTooBigLeftMyAddress"], true), true); }
#[test] fn extCodeSizeAddressInputTooBigRightMyAddress() { assert_eq!(test_transaction("ExtCodeSizeAddressInputTooBigRightMyAddress", &TESTS["ExtCodeSizeAddressInputTooBigRightMyAddress"], true), true); }
#[test] fn address0() { assert_eq!(test_transaction("address0", &TESTS["address0"], true), true); }
#[test] fn address1() { assert_eq!(test_transaction("address1", &TESTS["address1"], true), true); }
#[test] fn balance0() { assert_eq!(test_transaction("balance0", &TESTS["balance0"], true), true); }
#[test] fn balance01() { assert_eq!(test_transaction("balance01", &TESTS["balance01"], true), true); }
#[test] fn balance1() { assert_eq!(test_transaction("balance1", &TESTS["balance1"], true), true); }
#[test] fn balanceAddress2() { assert_eq!(test_transaction("balanceAddress2", &TESTS["balanceAddress2"], true), true); }
#[test] fn balanceAddressInputTooBig() { assert_eq!(test_transaction("balanceAddressInputTooBig", &TESTS["balanceAddressInputTooBig"], true), true); }
#[test] fn balanceAddressInputTooBigLeftMyAddress() { assert_eq!(test_transaction("balanceAddressInputTooBigLeftMyAddress", &TESTS["balanceAddressInputTooBigLeftMyAddress"], true), true); }
#[test] fn balanceAddressInputTooBigRightMyAddress() { assert_eq!(test_transaction("balanceAddressInputTooBigRightMyAddress", &TESTS["balanceAddressInputTooBigRightMyAddress"], true), true); }
#[test] fn balanceCaller3() { assert_eq!(test_transaction("balanceCaller3", &TESTS["balanceCaller3"], true), true); }
#[test] fn calldatacopy0() { assert_eq!(test_transaction("calldatacopy0", &TESTS["calldatacopy0"], true), true); }
#[test] fn calldatacopy0_return() { assert_eq!(test_transaction("calldatacopy0_return", &TESTS["calldatacopy0_return"], true), true); }
#[test] fn calldatacopy1() { assert_eq!(test_transaction("calldatacopy1", &TESTS["calldatacopy1"], true), true); }
#[test] fn calldatacopy1_return() { assert_eq!(test_transaction("calldatacopy1_return", &TESTS["calldatacopy1_return"], true), true); }
#[test] fn calldatacopy2() { assert_eq!(test_transaction("calldatacopy2", &TESTS["calldatacopy2"], true), true); }
#[test] fn calldatacopy2_return() { assert_eq!(test_transaction("calldatacopy2_return", &TESTS["calldatacopy2_return"], true), true); }
#[test] fn calldatacopyUnderFlow() { assert_eq!(test_transaction("calldatacopyUnderFlow", &TESTS["calldatacopyUnderFlow"], true), true); }
#[test] fn calldatacopyZeroMemExpansion() { assert_eq!(test_transaction("calldatacopyZeroMemExpansion", &TESTS["calldatacopyZeroMemExpansion"], true), true); }
#[test] fn calldatacopyZeroMemExpansion_return() { assert_eq!(test_transaction("calldatacopyZeroMemExpansion_return", &TESTS["calldatacopyZeroMemExpansion_return"], true), true); }
#[test] fn calldatacopy_DataIndexTooHigh() { assert_eq!(test_transaction("calldatacopy_DataIndexTooHigh", &TESTS["calldatacopy_DataIndexTooHigh"], true), true); }
#[test] fn calldatacopy_DataIndexTooHigh2() { assert_eq!(test_transaction("calldatacopy_DataIndexTooHigh2", &TESTS["calldatacopy_DataIndexTooHigh2"], true), true); }
#[test] fn calldatacopy_DataIndexTooHigh2_return() { assert_eq!(test_transaction("calldatacopy_DataIndexTooHigh2_return", &TESTS["calldatacopy_DataIndexTooHigh2_return"], true), true); }
#[test] fn calldatacopy_DataIndexTooHigh_return() { assert_eq!(test_transaction("calldatacopy_DataIndexTooHigh_return", &TESTS["calldatacopy_DataIndexTooHigh_return"], true), true); }
#[test] fn calldatacopy_sec() { assert_eq!(test_transaction("calldatacopy_sec", &TESTS["calldatacopy_sec"], true), true); }
#[test] fn calldataload0() { assert_eq!(test_transaction("calldataload0", &TESTS["calldataload0"], true), true); }
#[test] fn calldataload1() { assert_eq!(test_transaction("calldataload1", &TESTS["calldataload1"], true), true); }
#[test] fn calldataload2() { assert_eq!(test_transaction("calldataload2", &TESTS["calldataload2"], true), true); }
#[test] fn calldataloadSizeTooHigh() { assert_eq!(test_transaction("calldataloadSizeTooHigh", &TESTS["calldataloadSizeTooHigh"], true), true); }
#[test] fn calldataloadSizeTooHighPartial() { assert_eq!(test_transaction("calldataloadSizeTooHighPartial", &TESTS["calldataloadSizeTooHighPartial"], true), true); }
#[test] fn calldataload_BigOffset() { assert_eq!(test_transaction("calldataload_BigOffset", &TESTS["calldataload_BigOffset"], true), true); }
#[test] fn calldatasize0() { assert_eq!(test_transaction("calldatasize0", &TESTS["calldatasize0"], true), true); }
#[test] fn calldatasize1() { assert_eq!(test_transaction("calldatasize1", &TESTS["calldatasize1"], true), true); }
#[test] fn calldatasize2() { assert_eq!(test_transaction("calldatasize2", &TESTS["calldatasize2"], true), true); }
#[test] fn caller() { assert_eq!(test_transaction("caller", &TESTS["caller"], true), true); }
#[test] fn callvalue() { assert_eq!(test_transaction("callvalue", &TESTS["callvalue"], true), true); }
#[test] fn codecopy0() { assert_eq!(test_transaction("codecopy0", &TESTS["codecopy0"], true), true); }
#[test] fn codecopyZeroMemExpansion() { assert_eq!(test_transaction("codecopyZeroMemExpansion", &TESTS["codecopyZeroMemExpansion"], true), true); }
#[test] fn codecopy_DataIndexTooHigh() { assert_eq!(test_transaction("codecopy_DataIndexTooHigh", &TESTS["codecopy_DataIndexTooHigh"], true), true); }
#[test] fn codesize() { assert_eq!(test_transaction("codesize", &TESTS["codesize"], true), true); }
#[test] fn extcodecopy0() { assert_eq!(test_transaction("extcodecopy0", &TESTS["extcodecopy0"], true), true); }
#[test] fn extcodecopy0AddressTooBigLeft() { assert_eq!(test_transaction("extcodecopy0AddressTooBigLeft", &TESTS["extcodecopy0AddressTooBigLeft"], true), true); }
#[test] fn extcodecopy0AddressTooBigRight() { assert_eq!(test_transaction("extcodecopy0AddressTooBigRight", &TESTS["extcodecopy0AddressTooBigRight"], true), true); }
#[test] fn extcodecopyZeroMemExpansion() { assert_eq!(test_transaction("extcodecopyZeroMemExpansion", &TESTS["extcodecopyZeroMemExpansion"], true), true); }
#[test] fn extcodecopy_DataIndexTooHigh() { assert_eq!(test_transaction("extcodecopy_DataIndexTooHigh", &TESTS["extcodecopy_DataIndexTooHigh"], true), true); }
#[test] fn extcodesize0() { assert_eq!(test_transaction("extcodesize0", &TESTS["extcodesize0"], true), true); }
#[test] fn extcodesize1() { assert_eq!(test_transaction("extcodesize1", &TESTS["extcodesize1"], true), true); }
#[test] fn extcodesizeUnderFlow() { assert_eq!(test_transaction("extcodesizeUnderFlow", &TESTS["extcodesizeUnderFlow"], true), true); }
#[test] fn gasprice() { assert_eq!(test_transaction("gasprice", &TESTS["gasprice"], true), true); }
#[test] fn origin() { assert_eq!(test_transaction("origin", &TESTS["origin"], true), true); }
