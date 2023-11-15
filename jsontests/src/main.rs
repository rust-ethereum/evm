mod error;
mod hash;
mod run;
mod types;

use crate::error::Error;
use crate::types::*;
use clap::Parser;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::BufReader;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	filenames: Vec<String>,
}

fn run_file(filename: &str) -> Result<(), Error> {
	let test_multi: BTreeMap<String, TestMulti> =
		serde_json::from_reader(BufReader::new(File::open(filename)?))?;

	for (test_name, test_multi) in test_multi {
		let tests = test_multi.tests();

		for test in tests {
			print!(
				"{}/{}/{:?}/{}: ",
				filename, test_name, test.fork, test.index
			);
			match crate::run::run_test(filename, &test_name, test) {
				Ok(()) => println!("okay"),
				Err(Error::UnsupportedFork) => println!("skipped"),
				Err(err) => {
					println!("err {:?}", err);
					return Err(err);
				}
			}
		}
	}

	Ok(())
}

fn run_single(filename: &str) -> Result<(), Error> {
	if fs::metadata(&filename)?.is_dir() {
		for filename in fs::read_dir(&filename)? {
			let filepath = filename?.path();
			let filename = filepath.to_str().ok_or(Error::NonUtf8Filename)?;
			run_file(filename)?;
		}
	} else {
		run_file(&filename)?;
	}

	Ok(())
}

fn main() -> Result<(), Error> {
	let cli = Cli::parse();

	for filename in cli.filenames {
		run_single(&filename)?;
	}

	Ok(())
}
