pub mod error;
pub mod hash;
pub mod in_memory;
pub mod run;
pub mod types;

#[test]
fn st_args_zero_one_balance() {
	const JSON_FILENAME: &str = "res/ethtests/GeneralStateTests/stArgsZeroOneBalance/";
	let tests_status = run::run_single(JSON_FILENAME, false).unwrap();
	tests_status.print_total();
}

#[test]
fn st_code_copy_test() {
	const JSON_FILENAME: &str = "res/ethtests/GeneralStateTests/stCodeCopyTest/";
	let tests_status = run::run_single(JSON_FILENAME, false).unwrap();
	tests_status.print_total();
}

#[test]
fn st_example() {
	const JSON_FILENAME: &str = "res/ethtests/GeneralStateTests/stExample/";
	let tests_status = run::run_single(JSON_FILENAME, false).unwrap();
	tests_status.print_total();
}

#[test]
fn st_self_balance() {
	const JSON_FILENAME: &str = "res/ethtests/GeneralStateTests/stSelfBalance/";
	let tests_status = run::run_single(JSON_FILENAME, false).unwrap();
	tests_status.print_total();
}

#[test]
fn st_s_load_test() {
	const JSON_FILENAME: &str = "res/ethtests/GeneralStateTests/stSLoadTest/";
	let tests_status = run::run_single(JSON_FILENAME, false).unwrap();
	tests_status.print_total();
}

#[test]
fn vm_arithmetic_test() {
	const JSON_FILENAME: &str = "res/ethtests/GeneralStateTests/VMTests/vmArithmeticTest/";
	let tests_status = run::run_single(JSON_FILENAME, false).unwrap();
	tests_status.print_total();
}

#[test]
fn vm_bitwise_logic_operation() {
	const JSON_FILENAME: &str = "res/ethtests/GeneralStateTests/VMTests/vmBitwiseLogicOperation/";
	let tests_status = run::run_single(JSON_FILENAME, false).unwrap();
	tests_status.print_total();
}

#[test]
fn vm_io_and_flow_operations() {
	const JSON_FILENAME: &str = "res/ethtests/GeneralStateTests/VMTests/vmIOandFlowOperations/";
	let tests_status = run::run_single(JSON_FILENAME, false).unwrap();
	tests_status.print_total();
}

#[test]
fn vm_log_test() {
	const JSON_FILENAME: &str = "res/ethtests/GeneralStateTests/VMTests/vmLogTest/";
	let tests_status = run::run_single(JSON_FILENAME, false).unwrap();
	tests_status.print_total();
}

#[test]
fn vm_tests() {
	const JSON_FILENAME: &str = "res/ethtests/GeneralStateTests/VMTests/vmTests/";
	let tests_status = run::run_single(JSON_FILENAME, false).unwrap();
	tests_status.print_total();
}
