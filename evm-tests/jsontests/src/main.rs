use clap::{arg, command, value_parser, Arg, ArgAction, Command};
use ethjson::spec::ForkSpec;
use evm_jsontests::state as statetests;
use evm_jsontests::state::{TestExecutionResult, VerboseOutput};
use evm_jsontests::vm as vmtests;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[allow(clippy::cognitive_complexity)]
fn main() {
	let matches = command!()
		.version(env!("CARGO_PKG_VERSION"))
		.subcommand_required(true)
		.subcommand(
			Command::new("vm").about("vm tests runner").arg(
				Arg::new("PATH")
					.help("json file or directory for tests run")
					.required(true),
			),
		)
		.subcommand(
			Command::new("state")
				.about("state tests runner")
				.arg(
					arg!([PATH] "json file or directory for tests run")
						.required(true)
						.value_parser(value_parser!(PathBuf)),
				)
				.arg(arg!(-s --spec <SPEC> "Ethereum hard fork"))
				.arg(
					arg!(-v --verbose "Verbose output")
						.default_value("false")
						.action(ArgAction::SetTrue),
				)
				.arg(
					arg!(-f --verbose_failed "Verbose failed only output")
						.default_value("false")
						.action(ArgAction::SetTrue),
				)
				.arg(
					arg!(-w --very_verbose "Very verbose output")
						.default_value("false")
						.action(ArgAction::SetTrue),
				)
				.arg(
					arg!(-p --print_state "When test failed print state")
						.default_value("false")
						.action(ArgAction::SetTrue),
				),
		)
		.get_matches();

	if let Some(matches) = matches.subcommand_matches("vm") {
		for file_name in matches.get_many::<PathBuf>("PATH").unwrap() {
			let file = File::open(file_name).expect("Open failed");

			let reader = BufReader::new(file);
			let test_suite = serde_json::from_reader::<_, HashMap<String, vmtests::Test>>(reader)
				.expect("Parse test cases failed");

			for (name, test) in test_suite {
				vmtests::test(&name, test);
			}
		}
	}

	if let Some(matches) = matches.subcommand_matches("state") {
		let spec: Option<ForkSpec> = matches
			.get_one::<String>("spec")
			.and_then(|spec| spec.clone().try_into().ok());

		let verbose_output = VerboseOutput {
			verbose: matches.get_flag("verbose"),
			verbose_failed: matches.get_flag("verbose_failed"),
			very_verbose: matches.get_flag("very_verbose"),
			print_state: matches.get_flag("print_state"),
		};
		let mut tests_result = TestExecutionResult::new();
		for src_name in matches.get_many::<PathBuf>("PATH").unwrap() {
			let path = Path::new(src_name);
			assert!(path.exists(), "data source is not exist");
			if path.is_file() {
				run_test_for_file(&spec, &verbose_output, path, &mut tests_result);
			} else if path.is_dir() {
				run_test_for_dir(&spec, &verbose_output, path, &mut tests_result);
			}
		}
		println!("\nTOTAL: {}", tests_result.total);
		println!("FAILED: {}\n", tests_result.failed);
	}
}

fn run_test_for_dir(
	spec: &Option<ForkSpec>,
	verbose_output: &VerboseOutput,
	dir_name: &Path,
	tests_result: &mut TestExecutionResult,
) {
	if should_skip(dir_name) {
		println!("Skipping test case {:?}", dir_name);
		return;
	}
	for entry in fs::read_dir(dir_name).unwrap() {
		let entry = entry.unwrap();
		if let Some(s) = entry.file_name().to_str() {
			if s.starts_with('.') {
				continue;
			}
		}
		let path = entry.path();
		if path.is_dir() {
			run_test_for_dir(spec, verbose_output, path.as_path(), tests_result);
		} else {
			run_test_for_file(spec, verbose_output, path.as_path(), tests_result);
		}
	}
}

fn run_test_for_file(
	spec: &Option<ForkSpec>,
	verbose_output: &VerboseOutput,
	file_name: &Path,
	tests_result: &mut TestExecutionResult,
) {
	if should_skip(file_name) {
		if verbose_output.verbose {
			println!("Skipping test case {:?}", file_name);
		}
		return;
	}
	if verbose_output.verbose {
		println!(
			"RUN for: {}",
			short_test_file_name(file_name.to_str().unwrap())
		);
	}
	let file = File::open(file_name).expect("Open file failed");

	let reader = BufReader::new(file);
	let test_suite = serde_json::from_reader::<_, HashMap<String, statetests::Test>>(reader)
		.expect("Parse test cases failed");

	for (name, test) in test_suite {
		let test_res = statetests::test(verbose_output.clone(), &name, test, spec.clone());

		if test_res.failed > 0 {
			if verbose_output.verbose {
				println!("Tests count:\t{}", test_res.total);
				println!(
					"Failed:\t\t{} - {}\n",
					test_res.failed,
					short_test_file_name(file_name.to_str().unwrap())
				);
			} else if verbose_output.verbose_failed {
				println!(
					"RUN for: {}",
					short_test_file_name(file_name.to_str().unwrap())
				);
				println!("Tests count:\t{}", test_res.total);
				println!(
					"Failed:\t\t{} - {}\n",
					test_res.failed,
					short_test_file_name(file_name.to_str().unwrap())
				);
			}
		} else if verbose_output.verbose {
			println!("Tests count: {}\n", test_res.total);
		}

		tests_result.merge(test_res);
	}
}

fn short_test_file_name(name: &str) -> String {
	let res: Vec<_> = name.split("GeneralStateTests/").collect();
	if res.len() > 1 {
		res[1].to_string()
	} else {
		res[0].to_string()
	}
}

const SKIPPED_CASES: &[&str] = &[
	// funky test with `bigint 0x00` value in json :) not possible to happen on mainnet and require
	// custom json parser. https://github.com/ethereum/tests/issues/971
	"stTransactionTest/ValueOverflow",
	"stTransactionTest/ValueOverflowParis",
	// These tests are passing, but they take a lot of time to execute so can going to skip them.
	// NOTE: do not remove it to know slowest tests. It's useful for development.
	// "stTimeConsuming/static_Call50000_sha256",
	// "vmPerformance/loopMul",
	// "stTimeConsuming/CALLBlake2f_MaxRounds",
];

/// Check if a path should be skipped.
/// It checks:
/// - path/and_file_stam - check path and file name (without extention)
/// - path/with/sub/path - recursively check path
fn should_skip(path: &Path) -> bool {
	let matches = |case: &str| {
		let case_path = Path::new(case);
		let case_path_components: Vec<_> = case_path.components().collect();
		let path_components: Vec<_> = path.components().collect();
		let case_path_len = case_path_components.len();
		let path_len = path_components.len();

		// Check path length without file name
		if case_path_len > path_len {
			return false;
		}
		// Check stem file name (without extension)
		if let (Some(file_path_stem), Some(case_file_path_stem)) =
			(path.file_stem(), case_path.file_stem())
		{
			if file_path_stem == case_file_path_stem {
				// If case path contains only file name
				if case_path_len == 1 {
					return true;
				}
				// Check sub path without file names
				if case_path_len > 1
					&& path_len > 1 && case_path_components[..case_path_len - 1]
					== path_components[path_len - case_path_len..path_len - 1]
				{
					return true;
				}
			}
		}
		// Check recursively path from the end without file name
		if case_path_len < path_len && path_len > 1 {
			for i in 1..=path_len - case_path_len {
				if case_path_components
					== path_components[path_len - case_path_len - i..path_len - i]
				{
					return true;
				}
			}
		}
		false
	};

	SKIPPED_CASES.iter().any(|case| matches(case))
}
