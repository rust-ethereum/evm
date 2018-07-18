#![cfg(feature = "bench")]
#![feature(test)]
#![allow(non_snake_case)]

#[macro_use]
extern crate jsontests_derive;
extern crate jsontests;
extern crate test;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/vmPerformance"]
#[test_with = "jsontests::util::run_test"]
#[bench_with = "jsontests::util::run_bench"]
struct Performance;
