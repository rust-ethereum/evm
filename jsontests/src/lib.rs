pub mod error;
pub mod hash;
pub mod in_memory;
pub mod run;
pub mod types;

macro_rules! general_state_tests {
	( $name:ident, $folder:expr $(,$extra_meta:meta)? ) => {
		$(#[$extra_meta])?
		#[test]
		fn $name() {
			const JSON_FILENAME: &str = concat!("res/ethtests/GeneralStateTests/", $folder, "/");
			println!("name: {}", JSON_FILENAME);
			let tests_status = run::run_single(JSON_FILENAME, false, None).unwrap();
			tests_status.print_total();
		}
	}
}

general_state_tests!(st_args_zero_one_balance, "stArgsZeroOneBalance");
general_state_tests!(st_attack_test, "stAttackTest");
general_state_tests!(st_bad_opcode, "stBadOpcode");
general_state_tests!(st_bugs, "stBugs");
general_state_tests!(st_call_codes, "stCallCodes");
general_state_tests!(st_call_create_call_code_test, "stCallCreateCallCodeTest");
general_state_tests!(
	st_call_delegate_codes_call_code_homestead,
	"stCallDelegateCodesCallCodeHomestead"
);
general_state_tests!(
	st_call_delegate_codes_homestead,
	"stCallDelegateCodesHomestead"
);
general_state_tests!(st_chain_id, "stChainId");
general_state_tests!(st_code_copy_test, "stCodeCopyTest");
general_state_tests!(st_code_size_limit, "stCodeSizeLimit");
general_state_tests!(st_create2, "stCreate2");
general_state_tests!(st_create_test, "stCreateTest");
general_state_tests!(
	st_delegatecall_test_homestead,
	"stDelegatecallTestHomestead"
);
general_state_tests!(
	st_eip150_single_code_gas_prices,
	"stEIP150singleCodeGasPrices"
);
general_state_tests!(st_eip150_specific, "stEIP150Specific");
general_state_tests!(st_eip1559, "stEIP1559", ignore);
general_state_tests!(st_eip158_specific, "stEIP158Specific");
general_state_tests!(st_eip2930, "stEIP2930", ignore);
general_state_tests!(st_eip3607, "stEIP3607", ignore);
general_state_tests!(st_example, "stExample");
general_state_tests!(st_ext_code_hash, "stExtCodeHash");
general_state_tests!(st_homestead_specific, "stHomesteadSpecific");
general_state_tests!(st_init_code_test, "stInitCodeTest");
general_state_tests!(st_log_tests, "stLogTests");
general_state_tests!(st_mem_expanding_eip150_calls, "stMemExpandingEIP150Calls");
general_state_tests!(st_memory_stress_test, "stMemoryStressTest");
general_state_tests!(st_memory_test, "stMemoryTest");
general_state_tests!(st_non_zero_calls_test, "stNonZeroCallsTest");
general_state_tests!(st_precompiled_contracts, "stPreCompiledContracts");
general_state_tests!(st_precompiled_contracts2, "stPreCompiledContracts2");
general_state_tests!(st_quadratic_complexity_test, "stQuadraticComplexityTest");
general_state_tests!(st_random, "stRandom");
general_state_tests!(st_random2, "stRandom2");
general_state_tests!(st_recursive_create, "stRecursiveCreate");
general_state_tests!(st_refund_test, "stRefundTest");
general_state_tests!(st_return_data_test, "stReturnDataTest");
general_state_tests!(st_revert_test, "stRevertTest");
general_state_tests!(st_self_balance, "stSelfBalance");
general_state_tests!(st_shift, "stShift");
general_state_tests!(st_sload_test, "stSLoadTest");
general_state_tests!(st_solidity_test, "stSolidityTest");
general_state_tests!(st_special_test, "stSpecialTest");
general_state_tests!(st_sstore_test, "stSStoreTest", ignore);
general_state_tests!(st_stack_tests, "stStackTests", ignore);
general_state_tests!(st_static_call, "stStaticCall");
general_state_tests!(st_static_flag_enabled, "stStaticFlagEnabled");
general_state_tests!(st_system_operations_test, "stSystemOperationsTest", ignore);
general_state_tests!(st_time_consuming, "stTimeConsuming");
general_state_tests!(st_transaction_test, "stTransactionTest");
general_state_tests!(st_transition_test, "stTransitionTest");
general_state_tests!(st_wallet_test, "stWalletTest");
general_state_tests!(st_zero_calls_revert, "stZeroCallsRevert");
general_state_tests!(st_zero_calls_test, "stZeroCallsTest");
general_state_tests!(st_zero_knowledge, "stZeroKnowledge");
general_state_tests!(st_zero_knowledge2, "stZeroKnowledge2");

general_state_tests!(vm_tests_arithmetic_test, "VMTests/vmArithmeticTest");
general_state_tests!(
	vm_tests_bitwise_logic_operation,
	"VMTests/vmBitwiseLogicOperation"
);
general_state_tests!(
	vm_tests_io_and_flow_operations,
	"VMTests/vmIOandFlowOperations"
);
general_state_tests!(vm_tests_log_test, "VMTests/vmLogTest");
general_state_tests!(vm_tests_performance, "VMTests/vmPerformance");
general_state_tests!(vm_tests_tests, "VMTests/vmTests");
