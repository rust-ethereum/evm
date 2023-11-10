mod error;
mod hash;
mod run;
mod types;

use crate::error::Error;
use crate::types::*;
use clap::Parser;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;

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
			match crate::run::run_test(&test_name, test) {
				Ok(()) => println!("succeed"),
				Err(Error::UnsupportedFork) => println!("skipped"),
				Err(err) => Err(err)?,
			}
		}
	}

	Ok(())
}
