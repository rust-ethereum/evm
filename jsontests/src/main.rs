mod types;

use crate::types::TestMulti;
use clap::Parser;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use thiserror::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	filename: String,
}

#[derive(Error, Debug)]
enum Error {
	#[error("io error")]
	IO(#[from] std::io::Error),
	#[error("json error")]
	JSON(#[from] serde_json::Error),
	#[error("evm error")]
	EVM(#[from] evm::ExitError),
}

fn main() -> Result<(), Error> {
	let cli = Cli::parse();

	let test_multi: BTreeMap<String, TestMulti> =
		serde_json::from_reader(BufReader::new(File::open(cli.filename)?))?;
	println!("test multi: {:?}", test_multi);

	Ok(())
}
