use std::path::PathBuf;
use std::fs::{self, File};
use std::io::BufReader;
use std::collections::HashMap;
use evm_jsontests::vm as vmtests;

pub fn run(dir: &str) {
	let mut dest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	dest.push(dir);

	for entry in fs::read_dir(dest).unwrap() {
		let entry = entry.unwrap();
        let path = entry.path();

		let file = File::open(path).expect("Open file failed");

		let reader = BufReader::new(file);
		let coll = serde_json::from_reader::<_, HashMap<String, vmtests::Test>>(reader)
			.expect("Parse test cases failed");

		for (name, test) in coll {
			vmtests::test(&name, test);
		}
	}
}

#[test] fn vm_arithmetic() { run("res/ethtests/VMTests/vmArithmeticTest"); }
#[test] fn vm_bitwise_logic() { run("res/ethtests/VMTests/vmBitwiseLogicOperation"); }
#[test] fn vm_block_info() { run("res/ethtests/VMTests/vmBlockInfoTest"); }
#[test] fn vm_environmental_info() { run("res/ethtests/VMTests/vmEnvironmentalInfo"); }
#[test] fn vm_io_and_flow() { run("res/ethtests/VMTests/vmIOandFlowOperations"); }
#[test] fn vm_log() { run("res/ethtests/VMTests/vmLogTest"); }
#[test] #[ignore] fn vm_performance() { run("res/ethtests/VMTests/vmPerformance"); }
#[test] fn vm_push_dup_swap() { run("res/ethtests/VMTests/vmPushDupSwapTest"); }
#[test] fn vm_random() { run("res/ethtests/VMTests/vmRandomTest"); }
#[test] fn vm_sha3() { run("res/ethtests/VMTests/vmSha3Test"); }
#[test] fn vm_system() { run("res/ethtests/VMTests/vmSystemOperations"); }
#[test] fn vm_other() { run("res/ethtests/VMTests/vmTests"); }
