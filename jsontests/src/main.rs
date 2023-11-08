mod types;
mod error;
mod run;

use crate::types::*;
use clap::Parser;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use crate::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	filename: String,
}

fn main() -> Result<(), Error> {
	let cli = Cli::parse();

	let test_multi: BTreeMap<String, TestMulti> =
		serde_json::from_reader(BufReader::new(File::open(cli.filename)?))?;

	for (test_name, test_multi) in test_multi {
		let tests = test_multi.tests();

        for test in tests {
            crate::run::run_test(&test_name, test)?;
        }
	}

	Ok(())
}
