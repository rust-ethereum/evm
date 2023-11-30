mod error;
mod hash;
mod in_memory;
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

	#[arg(short, long, default_value_t = false)]
	debug: bool,
}

fn run_file(filename: &str, debug: bool) -> Result<(), Error> {
	let test_multi: BTreeMap<String, TestMulti> =
		serde_json::from_reader(BufReader::new(File::open(filename)?))?;

	for (test_name, test_multi) in test_multi {
		let tests = test_multi.tests();

		for test in tests {
			if debug {
				println!(
					"{}/{}/{:?}/{} ===>",
					filename, test_name, test.fork, test.index
				);
			} else {
				print!(
					"{}/{}/{:?}/{}: ",
					filename, test_name, test.fork, test.index
				);
			}
			match crate::run::run_test(filename, &test_name, test, debug) {
				Ok(()) => println!("okay"),
				Err(Error::UnsupportedFork) => println!("skipped"),
				Err(err) => {
					println!("err {:?}", err);
					return Err(err);
				}
			}
			if debug {
				println!();
			}
		}
	}

	Ok(())
}

fn run_single(filename: &str, debug: bool) -> Result<(), Error> {
	if fs::metadata(filename)?.is_dir() {
		for filename in fs::read_dir(filename)? {
			let filepath = filename?.path();
			let filename = filepath.to_str().ok_or(Error::NonUtf8Filename)?;
			run_file(filename, debug)?;
		}
	} else {
		run_file(filename, debug)?;
	}

	Ok(())
}

fn main() -> Result<(), Error> {
	let cli = Cli::parse();

	for filename in cli.filenames {
		run_single(&filename, cli.debug)?;
	}

	Ok(())
}
