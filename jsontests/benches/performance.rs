#![feature(test)]

extern crate test;
extern crate jsontests;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use test::Bencher;
use serde_json::Value;
use jsontests::test_transaction;

lazy_static! {
    static ref TESTS: Value =
        serde_json::from_str(include_str!("../res/files/vmPerformanceTest.json")).unwrap();
}

#[bench]
fn ackermann31(b: &mut Bencher) {
    b.iter(|| {
        assert_eq!(test_transaction("ackermann31", &TESTS["ackermann31"], false), true)
    });
}

#[bench]
fn ackermann32(b: &mut Bencher) {
    b.iter(|| {
        assert_eq!(test_transaction("ackermann32", &TESTS["ackermann32"], false), true)
    });
}

#[bench]
fn ackermann33(b: &mut Bencher) {
    b.iter(|| {
        assert_eq!(test_transaction("ackermann33", &TESTS["ackermann33"], false), true)
    });
}

#[bench]
fn fibonacci10(b: &mut Bencher) {
    b.iter(|| {
        assert_eq!(test_transaction("fibonacci10", &TESTS["fibonacci10"], false), true)
    });
}

#[bench]
fn fibonacci16(b: &mut Bencher) {
    b.iter(|| {
        assert_eq!(test_transaction("fibonacci16", &TESTS["fibonacci16"], false), true);
    })
}

#[bench]
fn many_functions100(b: &mut Bencher) {
    b.iter(|| {
        assert_eq!(test_transaction("manyFunctions100", &TESTS["manyFunctions100"], false), true);
    });
}
