use serde_json::Value;
use serde_json as json;
use test_transaction;

pub fn run_test(name: &str, test: &str) {
    let test: Value = json::from_str(test).unwrap();
    assert_eq!(test_transaction(name, &test, true), Ok(true));
}

#[cfg(feature = "bench")]
use test::Bencher;

#[cfg(feature = "bench")]
pub fn run_bench(b: &mut Bencher, name: &str, test: &str) {
    let test: Value = json::from_str(test).unwrap();
    b.iter(|| {
        // TODO: adjust test_transaction or write another function
        // TODO: in order to start benchmark as close to actual sputnik code as possible
        assert_eq!(test_transaction(name, &test, true), Ok(true));
    })
}
