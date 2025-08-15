use jsontests_proc_macro::statetest_folder;

statetest_folder!(
	"legacytests_constaninople",
	"../res/legacytests/Constantinople/GeneralStateTests"
);
// statetest_folder!("legacytests_constaninople_vmtests", "../res/legacytests/Constantinople/VMTests");
statetest_folder!(
	"legacytests_cancun",
	"../res/legacytests/Cancun/GeneralStateTests"
);
