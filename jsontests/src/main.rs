use std::fs::File;
use std::collections::HashMap;
use std::io::{self, BufReader, Write};
use clap::{App, Arg};
use evm::gasometer;
use evm::executors::memory;
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
			print!("Running test {} ... ", name);
			io::stdout().flush().ok().expect("Could not flush stdout");

			let original_state = test.unwrap_to_pre_state();
			let vicinity = test.unwrap_to_vicinity();
			let gasometer_config = gasometer::Config::frontier();
			let mut executor = memory::Executor::new(
				&original_state,
				vicinity,
				test.unwrap_to_gas_limit(),
				&gasometer_config,
			);

			let code = test.unwrap_to_code();
			let data = test.unwrap_to_data();
			let context = test.unwrap_to_context();
			let mut runtime = evm::Runtime::new(code, data, 1024, 1000000, context);

			let reason = executor.execute(&mut runtime);

			if test.out.is_none() {
				print!("{:?} ", reason);

				assert!(reason.is_error());
				assert!(test.post.is_none() && test.gas.is_none());

				println!("succeed");
			} else {
				let expected_post_gas = test.unwrap_to_post_gas();
				print!("{:?} ", reason);

				assert_eq!(runtime.machine().return_value(), test.unwrap_to_return_value());
				assert_eq!(executor.state(), &test.unwrap_to_post_state());
				assert_eq!(executor.gas(), expected_post_gas);
				println!("succeed");
			}
		}
	}
}
