mod mock;
use evm::uint::{H160, H256, U256, U256Ext};
use evm::{
	backend::{OverlayedBackend, RuntimeBaseBackend},
	interpreter::{
		ExitError,
		etable::{Chained, Single},
	},
	standard::{
		Config, DispatchEtable, EtableResolver, Invoker, TransactArgs, TransactArgsCallCreate,
		TransactValue, TransactValueCallCreate,
	},
};
use mock::{MockAccount, MockBackend};

const SIMPLE_CONTRACT_INITCODE: &str = include_str!("./contract/simple_contract_bytecode.txt");
const DEPLOY_AND_DESTROY_INITCODE: &str = include_str!("./contract/deploy_and_destroy_init_code");

fn transact(
	args: TransactArgs,
	overlayed_backend: &mut OverlayedBackend<MockBackend>,
) -> Result<TransactValue, ExitError> {
	let gas_etable = Single::new(evm::standard::eval_gasometer);
	let exec_etable = DispatchEtable::runtime();
	let etable = Chained(gas_etable, exec_etable);
	let resolver = EtableResolver::new(&(), &etable);
	let invoker = Invoker::new(&resolver);

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
			nonce: U256::ONE,
			storage: Default::default(),
			transient_storage: Default::default(),
		},
	);
	let config = Config::shanghai();
	let mut overlayed_backend = OverlayedBackend::new(backend, &config.runtime);

	let init_code = hex::decode(SIMPLE_CONTRACT_INITCODE.trim_end()).unwrap();
	let args = TransactArgs {
		call_create: TransactArgsCallCreate::Create {
			init_code,
			salt: Some(H256::from_low_u64_be(4)),
		},
		caller: H160::from_low_u64_be(1),
		value: U256::ZERO,
		gas_limit: U256::from(400_000),
		gas_price: U256::from(1).into(),
		access_list: vec![],
		config: &config,
	};

	// Create simple contract
	let contract_address = match transact(args, &mut overlayed_backend) {
		Ok(TransactValue {
			call_create: TransactValueCallCreate::Create { address, .. },
			..
		}) => address,
		_ => panic!("Failed to create contract"),
	};

	// Verify contract creation
	assert!(!overlayed_backend.code(contract_address).is_empty());
	assert_eq!(overlayed_backend.nonce(contract_address), U256::ONE);

	// Apply overlayed changeset
	let (mut backend, changeset) = overlayed_backend.deconstruct();
	backend.apply_overlayed(&changeset);

	// Call Self destruct in another transaction
	let mut overlayed_backend = OverlayedBackend::new(backend, &config.runtime);
	let args = TransactArgs {
		call_create: TransactArgsCallCreate::Call {
			address: contract_address,
			data: hex::decode(
				"00f55d9d00000000000000000000000055c41626c84445180eda39bac564606c633dd980",
			)
			.unwrap(),
		},
		caller: H160::from_low_u64_be(1),
		value: U256::ZERO,
		gas_limit: U256::from(400_000),
		gas_price: U256::ONE.into(),
		access_list: vec![],
		config: &config,
	};

	let result = transact(args, &mut overlayed_backend);
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
			nonce: U256::ONE,
			storage: Default::default(),
			transient_storage: Default::default(),
		},
	);
	let config = Config::cancun();
	let mut overlayed_backend = OverlayedBackend::new(backend, &config.runtime);

	let init_code =
		hex::decode(SIMPLE_CONTRACT_INITCODE.trim_end()).expect("Failed to decode contract");
	let args = TransactArgs {
		call_create: TransactArgsCallCreate::Create {
			init_code,
			salt: Some(H256::from_low_u64_be(4)),
		},
		caller: H160::from_low_u64_be(1),
		value: U256::ZERO,
		gas_limit: U256::from(400_000),
		gas_price: U256::from(1).into(),
		access_list: vec![],
		config: &config,
	};

	// Create simple contract
	let contract_address = match transact(args, &mut overlayed_backend) {
		Ok(TransactValue {
			call_create: TransactValueCallCreate::Create { address, .. },
			..
		}) => address,
		_ => panic!("Failed to create contract"),
	};

	// Verify contract creation
	assert!(!overlayed_backend.code(contract_address).is_empty());
	assert_eq!(overlayed_backend.nonce(contract_address), U256::ONE);

	// Apply overlayed changeset
	let (mut backend, changeset) = overlayed_backend.deconstruct();
	backend.apply_overlayed(&changeset);

	let mut overlayed_backend = OverlayedBackend::new(backend, &config.runtime);
	// Self destruct contract in another transaction
	let args = TransactArgs {
		call_create: TransactArgsCallCreate::Call {
			address: contract_address,
			data: hex::decode(
				"00f55d9d00000000000000000000000055c41626c84445180eda39bac564606c633dd980",
			)
			.unwrap(),
		},
		caller: H160::from_low_u64_be(1),
		value: U256::ZERO,
		gas_limit: U256::from(400_000),
		gas_price: U256::ONE.into(),
		access_list: vec![],
		config: &config,
	};

	let result = transact(args, &mut overlayed_backend);
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
			nonce: U256::ONE,
			storage: Default::default(),
			transient_storage: Default::default(),
		},
	);
	let config = Config::cancun();
	let mut overlayed_backend = OverlayedBackend::new(backend, &config.runtime);

	let init_code =
		hex::decode(DEPLOY_AND_DESTROY_INITCODE.trim_end()).expect("Failed to decode contract");
	let args = TransactArgs {
		call_create: TransactArgsCallCreate::Create {
			init_code,
			salt: Some(H256::from_low_u64_be(4)),
		},
		caller: H160::from_low_u64_be(1),
		value: U256::ZERO,
		gas_limit: U256::from(400_000),
		gas_price: U256::from(1).into(),
		access_list: vec![],
		config: &config,
	};

	// Create deploy and destroy contract
	let result = transact(args, &mut overlayed_backend);
	assert!(result.is_ok());

	// Verify contract was deleted
	assert!(!overlayed_backend.deconstruct().1.deletes.is_empty());
}
