mod mock;
use evm::{
	backend::{OverlayedBackend, RuntimeBaseBackend},
	interpreter::{
		error::ExitError,
		etable::{Chained, Single},
	},
	standard::{Config, Etable, EtableResolver, Invoker, TransactArgs, TransactValue},
};
use mock::{MockAccount, MockBackend};
use primitive_types::{H160, H256, U256};

const SIMPLE_CONTRACT_INITCODE: &str = include_str!("./contract/simple_contract_bytecode.txt");
const DEPLOY_AND_DESTROY_INITCODE: &str = include_str!("./contract/deploy_and_destroy_init_code");

fn transact(
	config: &Config,
	args: TransactArgs,
	overlayed_backend: &mut OverlayedBackend<MockBackend>,
) -> Result<TransactValue, ExitError> {
	let gas_etable = Single::new(evm::standard::eval_gasometer);
	let exec_etable = Etable::runtime();
	let etable = Chained(gas_etable, exec_etable);
	let resolver = EtableResolver::new(config, &(), &etable);
	let invoker = Invoker::new(config, &resolver);

	evm::transact(args.clone(), Some(4), overlayed_backend, &invoker)
}

#[test]
fn self_destruct_before_cancun() {
	let mut backend = MockBackend::default();
	backend.state.insert(
		H160::from_low_u64_be(1),
		MockAccount {
			balance: U256::from(1_000_000_000),
			code: vec![],
			nonce: U256::one(),
			storage: Default::default(),
			transient_storage: Default::default(),
		},
	);
	let config = Config::shanghai();
	let mut overlayed_backend = OverlayedBackend::new(backend, Default::default(), &config);

	let init_code = hex::decode(SIMPLE_CONTRACT_INITCODE.trim_end()).unwrap();
	let args = TransactArgs::Create {
		caller: H160::from_low_u64_be(1),
		value: U256::zero(),
		init_code,
		salt: Some(H256::from_low_u64_be(4)),
		gas_limit: U256::from(400_000),
		gas_price: U256::from(1),
		access_list: vec![],
	};

	// Create simple contract
	let contract_address = match transact(&config, args, &mut overlayed_backend) {
		Ok(TransactValue::Create { address, .. }) => address,
		_ => panic!("Failed to create contract"),
	};

	// Verify contract creation
	assert!(!overlayed_backend.code(contract_address).is_empty());
	assert_eq!(overlayed_backend.nonce(contract_address), U256::one());

	// Apply overlayed changeset
	let (mut backend, changeset) = overlayed_backend.deconstruct();
	backend.apply_overlayed(&changeset);

	// Call Self destruct in anothor transaction
	let mut overlayed_backend = OverlayedBackend::new(backend, Default::default(), &config);
	let args = TransactArgs::Call {
		caller: H160::from_low_u64_be(1),
		address: contract_address,
		value: U256::zero(),
		data: hex::decode(
			"00f55d9d00000000000000000000000055c41626c84445180eda39bac564606c633dd980",
		)
		.unwrap(),
		gas_limit: U256::from(400_000),
		gas_price: U256::one(),
		access_list: vec![],
	};

	let result = transact(&config, args, &mut overlayed_backend);
	let changeset = overlayed_backend.deconstruct().1;

	assert!(result.is_ok());
	assert!(changeset.deletes.contains(&contract_address));
}

#[test]
fn self_destruct_cancun() {
	let mut backend = MockBackend::default();
	backend.state.insert(
		H160::from_low_u64_be(1),
		MockAccount {
			balance: U256::from(1_000_000_000),
			code: vec![],
			nonce: U256::one(),
			storage: Default::default(),
			transient_storage: Default::default(),
		},
	);
	let config = Config::cancun();
	let mut overlayed_backend = OverlayedBackend::new(backend, Default::default(), &config);

	let init_code =
		hex::decode(SIMPLE_CONTRACT_INITCODE.trim_end()).expect("Failed to decode contract");
	let args = TransactArgs::Create {
		caller: H160::from_low_u64_be(1),
		value: U256::zero(),
		init_code,
		salt: Some(H256::from_low_u64_be(4)),
		gas_limit: U256::from(400_000),
		gas_price: U256::from(1),
		access_list: vec![],
	};

	// Create simple contract
	let contract_address = match transact(&config, args, &mut overlayed_backend) {
		Ok(TransactValue::Create { address, .. }) => address,
		_ => panic!("Failed to create contract"),
	};

	// Verify contract creation
	assert!(!overlayed_backend.code(contract_address).is_empty());
	assert_eq!(overlayed_backend.nonce(contract_address), U256::one());

	// Apply overlayed changeset
	let (mut backend, changeset) = overlayed_backend.deconstruct();
	backend.apply_overlayed(&changeset);

	let mut overlayed_backend = OverlayedBackend::new(backend, Default::default(), &config);
	// Self destruct contract in another transaction
	let args = TransactArgs::Call {
		caller: H160::from_low_u64_be(1),
		address: contract_address,
		value: U256::zero(),
		data: hex::decode(
			"00f55d9d00000000000000000000000055c41626c84445180eda39bac564606c633dd980",
		)
		.unwrap(),
		gas_limit: U256::from(400_000),
		gas_price: U256::one(),
		access_list: vec![],
	};

	let result = transact(&config, args, &mut overlayed_backend);
	let changeset = overlayed_backend.deconstruct().1;

	assert!(result.is_ok());
	assert!(!changeset.deletes.contains(&contract_address));
}

#[test]
fn self_destruct_same_tx_cancun() {
	let mut backend = MockBackend::default();
	backend.state.insert(
		H160::from_low_u64_be(1),
		MockAccount {
			balance: U256::from(1_000_000_000),
			code: vec![],
			nonce: U256::one(),
			storage: Default::default(),
			transient_storage: Default::default(),
		},
	);
	let config = Config::cancun();
	let mut overlayed_backend = OverlayedBackend::new(backend, Default::default(), &config);

	let init_code =
		hex::decode(DEPLOY_AND_DESTROY_INITCODE.trim_end()).expect("Failed to decode contract");
	let args = TransactArgs::Create {
		caller: H160::from_low_u64_be(1),
		value: U256::zero(),
		init_code,
		salt: Some(H256::from_low_u64_be(4)),
		gas_limit: U256::from(400_000),
		gas_price: U256::from(1),
		access_list: vec![],
	};

	// Create deploy and destroy contract
	let result = transact(&config, args, &mut overlayed_backend);
	assert!(result.is_ok());

	// Verify contract was deleted
	assert!(!overlayed_backend.deconstruct().1.deletes.is_empty());
}
