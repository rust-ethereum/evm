mod error;
mod hash;
mod in_memory;
mod run;
mod types;

use clap::Parser;

use crate::{error::Error, types::*};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	filenames: Vec<String>,

	#[arg(short, long, default_value_t = false)]
	debug: bool,
}

fn main() -> Result<(), Error> {
	let cli = Cli::parse();

	let mut tests_status = TestCompletionStatus::default();
	for filename in cli.filenames {
		tests_status += run::run_single(&filename, cli.debug)?;
	}
	tests_status.print_total();

	Ok(())
}
