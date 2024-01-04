use clap::{App, Arg, SubCommand};
use evm_jsontests::state as statetests;
use evm_jsontests::vm as vmtests;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

fn main() {
	let matches = App::new("jsontests")
		.version("0.1.0")
		.author("Wei Tang <hi@that.world>")
		.about("EVM json test utilities")
		.subcommand(
			SubCommand::with_name("vm").arg(
				Arg::with_name("FILE")
					.help("Target yaml file to import")
					.required(true)
					.min_values(1),
			),
		)
		.subcommand(
			SubCommand::with_name("state").arg(
				Arg::with_name("FILE")
					.help("Target yaml file to import")
					.required(true)
					.min_values(1),
			),
		)
		.get_matches();

	if let Some(matches) = matches.subcommand_matches("vm") {
		for file_name in matches.values_of("FILE").unwrap() {
			let file = File::open(file_name).expect("Open file failed");

			let reader = BufReader::new(file);
			let coll = serde_json::from_reader::<_, HashMap<String, vmtests::Test>>(reader)
				.expect("Parse test cases failed");

			for (name, test) in coll {
				vmtests::test(&name, test);
			}
		}
	}

	if let Some(matches) = matches.subcommand_matches("state") {
		for file_name in matches.values_of("FILE").unwrap() {
			let file = File::open(file_name).expect("Open file failed");

			let reader = BufReader::new(file);
			let coll = serde_json::from_reader::<_, HashMap<String, statetests::Test>>(reader)
				.expect("Parse test cases failed");

			for (name, test) in coll {
				statetests::test(&name, test);
			}
		}
	}
}
