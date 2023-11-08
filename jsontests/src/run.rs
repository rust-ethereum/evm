use crate::types::*;
use crate::error::Error;

static SUPPORTED_FORKS: [Fork; 0] = [];

pub fn run_test(test_name: &str, test: Test) -> Result<(), Error> {
	println!("test name: {}, fork: {:?}, index: {}", test_name, test.fork, test.index);

	Ok(())
}
