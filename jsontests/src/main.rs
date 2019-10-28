use std::fs::File;
use std::collections::HashMap;
use std::io::BufReader;
use clap::{App, Arg};
use evm_jsontests::vm as vmtests;

fn main() {
	let matches = App::new("jsontests")
        .version("0.1.0")
        .author("Wei Tang <hi@that.world>")
        .about("EVM json test utilities")
        .arg(Arg::with_name("FILE")
             .help("Target yaml file to import")
             .required(true)
			 .min_values(1))
		.get_matches();

	for file_name in matches.values_of("FILE").unwrap() {
		let file = File::open(file_name).expect("Open file failed");

		let reader = BufReader::new(file);
		let coll = serde_json::from_reader::<_, HashMap<String, vmtests::Test>>(reader)
			.expect("Parse test cases failed");

		for (name, test) in coll {
			evm_jsontests::vm::test(&name, test);
		}
	}
}
