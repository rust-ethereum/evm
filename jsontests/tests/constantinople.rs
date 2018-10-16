#![cfg_attr(feature = "bench", feature(test))]
#![allow(non_snake_case)]
#![allow(unused)]

#[macro_use]
extern crate jsontests_derive;
extern crate jsontests;
extern crate evm;

#[cfg(feature = "bench")]
extern crate test;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmConstantinopleTests"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct ConstantinopleTests;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmEIP1283"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct EIP1283Tests;


