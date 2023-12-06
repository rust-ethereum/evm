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

const BASIC_FILE_PATH: &str = "jsontests/res/ethtests/GeneralStateTests/";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	filenames: Vec<String>,

	#[arg(short, long, default_value_t = false)]
	debug: bool,
}

fn run_file(filename: &str, debug: bool) -> Result<TestCompletionStatus, Error> {
	let test_multi: BTreeMap<String, TestMulti> =
		serde_json::from_reader(BufReader::new(File::open(filename)?))?;
	let mut tests_status = TestCompletionStatus::default();

	for (test_name, test_multi) in test_multi {
		let tests = test_multi.tests();
		let short_file_name = filename.replace(BASIC_FILE_PATH, "");
		for test in &tests {
			if debug {
				print!(
					"[{:?}] {} | {}/{} DEBUG: ",
					test.fork, short_file_name, test_name, test.index
				);
			} else {
				print!(
					"[{:?}] {} | {}/{}: ",
					test.fork, short_file_name, test_name, test.index
				);
			}
			match run::run_test(filename, &test_name, test.clone(), debug) {
				Ok(()) => {
					tests_status.inc_completed();
					println!("ok")
				}
				Err(Error::UnsupportedFork) => {
					tests_status.inc_skipped();
					println!("skipped")
				}
				Err(err) => {
					println!("ERROR: {:?}", err);
					return Err(err);
				}
			}
			if debug {
				println!();
			}
		}

		tests_status.print_completion();
	}

	Ok(tests_status)
}

fn run_single(filename: &str, debug: bool) -> Result<TestCompletionStatus, Error> {
	if fs::metadata(filename)?.is_dir() {
		let mut tests_status = TestCompletionStatus::default();

		for filename in fs::read_dir(filename)? {
			let filepath = filename?.path();
			let filename = filepath.to_str().ok_or(Error::NonUtf8Filename)?;
			println!("RUM for: {filename}");
			tests_status += run_file(filename, debug)?;
		}
		tests_status.print_total_for_dir(filename);
		Ok(tests_status)
	} else {
		run_file(filename, debug)
	}
}

fn main() -> Result<(), Error> {
	let cli = Cli::parse();

	let mut tests_status = TestCompletionStatus::default();
	for filename in cli.filenames {
		tests_status += run_single(&filename, cli.debug)?;
	}
	tests_status.print_total();

	Ok(())
}
