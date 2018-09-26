#![cfg_attr(feature = "bench", feature(test))]
#![allow(non_snake_case)]
#![allow(unused)]

#[macro_use]
extern crate jsontests_derive;
extern crate jsontests;

#[cfg(feature = "bench")]
extern crate test;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/SputnikVM/vmConstantinopleTests"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct ConstantinopleTests;
