#![cfg_attr(feature = "bench", feature(test))]
#![allow(non_snake_case)]
#![allow(unused)]

#[macro_use]
extern crate jsontests_derive;
extern crate jsontests;

#[cfg(feature = "bench")]
extern crate test;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmArithmeticTest"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct Arithmetic;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmBitwiseLogicOperation"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct BitwiseLogicOperation;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmBlockInfoTest"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct BlockInfo;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmEnvironmentalInfo"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct VmInverontemtalInfo;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmIOandFlowOperations"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct VmIOandFlowOperations;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmLogTest"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct Log;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmPushDupSwapTest"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct PushDupSwap;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmRandomTest"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct Random;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmSha3Test"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct Sha3;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmSystemOperations"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct SystemOperations;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/eth/VMTests/vmTests"]
#[test_with = "jsontests::util::run_test"]
#[cfg_attr(feature = "bench", bench_with = "jsontests::util::run_bench")]
struct VM;
