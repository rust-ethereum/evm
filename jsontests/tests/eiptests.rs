#![cfg_attr(feature = "bench", feature(test))]
#![allow(non_snake_case)]
#![allow(unused)]

#[macro_use]
extern crate jsontests_derive;
extern crate jsontests;

#[cfg(feature = "bench")]
extern crate test;

/// Shifting opcodes tests
#[derive(JsonTests)]
#[directory = "jsontests/res/files/SputnikVM/vmEIP215"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct EIP215;

/// EXTCODEHASH tests
#[derive(JsonTests)]
#[directory = "jsontests/res/files/SputnikVM/vmEIP215"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct EIP1052;

/// CREATE2 tests
#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmEIP1014"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct EIP1014;
