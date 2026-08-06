use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use spectests::{CaseResult, RunOptions, run};

#[derive(Debug, Parser)]
#[command(about = "Run pinned EELS state-test fixtures against rust-evm")]
struct Args {
	/// Extracted fixture root containing `.meta/index.json`.
	#[arg(long)]
	fixtures: PathBuf,
	/// Pinned fixture target.
	#[arg(long, default_value = "Cancun")]
	target: String,
	/// Run an exact fixture ID. Repeat to select multiple cases.
	#[arg(long = "case")]
	cases: Vec<String>,
	/// Number of fixture files to execute concurrently.
	#[arg(long)]
	jobs: Option<usize>,
}

fn main() -> ExitCode {
	let args = Args::parse();
	let options = RunOptions {
		fixtures: args.fixtures,
		target: args.target,
		cases: args.cases,
		jobs: args.jobs,
	};
	match run(&options) {
		Ok(summary) => {
			for result in &summary.results {
				if let CaseResult::Failed { id, detail } = result {
					eprintln!("FAILED {id}\n  {detail}");
				}
			}
			println!(
				"{}: selected={} passed={} failed={} skipped=0",
				summary.target, summary.selected, summary.passed, summary.failed
			);
			if summary.failed == 0 {
				ExitCode::SUCCESS
			} else {
				ExitCode::from(1)
			}
		}
		Err(error) => {
			eprintln!("fixture error: {error}");
			ExitCode::from(2)
		}
	}
}
