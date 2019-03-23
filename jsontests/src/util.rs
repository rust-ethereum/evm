use evm::Patch;
use serde_json as json;
use serde_json::Value;

use crate::{bench_transaction, test_transaction};

pub fn run_test<P: Patch>(name: &str, test: &str) {
    let test: Value = json::from_str(test).unwrap();
    assert_eq!(test_transaction::<P>(name, &test, true), Ok(true));
}

use criterion::Criterion;

pub fn run_bench<P: Patch>(c: &mut Criterion, name: &'static str, test: &str) {
    let test: Value = json::from_str(test).unwrap();
    bench_transaction::<P>(name, test, c);
}
