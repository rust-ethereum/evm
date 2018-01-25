extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate evm;
extern crate evm_stateful;
extern crate block;
extern crate trie;
extern crate rand;
extern crate sha3;
extern crate bigint;
extern crate evm_network_classic;

use sha3::{Digest, Keccak256};
use bigint::{H256, U256, Address, Gas};
use evm::{ValidTransaction, Storage, AccountChange, VM, SeqTransactionVM, HeaderParams, VMStatus, AccountPatch};
use evm_network_classic::{MainnetEIP160Patch, EIP160Patch};
use evm_stateful::{MemoryStateful, LiteralAccount};
use block::TransactionAction;
use trie::{Database, MemoryDatabase};
use std::collections::HashMap;
use std::str::FromStr;
use std::rc::Rc;
use rand::Rng;

#[derive(Serialize, Deserialize, Debug)]
struct JSONAccount {
    balance: String,
}

lazy_static! {
    static ref GENESIS_ACCOUNTS: HashMap<String, JSONAccount> =
        serde_json::from_str(include_str!("../res/genesis.json")).unwrap();
}

lazy_static! {
    static ref MORDEN_ACCOUNTS: HashMap<String, JSONAccount> =
        serde_json::from_str(include_str!("../res/morden.json")).unwrap();
}

#[test]
fn secure_trie() {
    let mut database = MemoryDatabase::new();
    let mut trie = database.create_empty();

    trie.insert_raw(Keccak256::digest("doe".as_bytes()).as_slice().into(),
                    "reindeer".as_bytes().into());
    trie.insert_raw(Keccak256::digest("dog".as_bytes()).as_slice().into(),
                    "puppy".as_bytes().into());
    trie.insert_raw(Keccak256::digest("dogglesworth".as_bytes()).as_slice().into(),
                    "cat".as_bytes().into());

    assert_eq!(trie.root(), H256::from_str("0xd4cd937e4a4368d7931a9cf51686b7e10abb3dce38a39000fd7902a092b64585").unwrap());
}

#[test]
fn morden_state_root() {
    let database = MemoryDatabase::default();
    let mut stateful = MemoryStateful::empty(&database);
    let mut rng = rand::thread_rng();

    let mut accounts: Vec<(&String, &JSONAccount)> = MORDEN_ACCOUNTS.iter().collect();
    rng.shuffle(&mut accounts);

    for (key, value) in accounts {
        struct MordenAccountPatch;
        impl AccountPatch for MordenAccountPatch {
            fn initial_nonce() -> U256 { U256::from(2u64.pow(20)) }
            fn initial_create_nonce() -> U256 { Self::initial_nonce() }
            fn empty_considered_exists() -> bool { true }
        }

        let address = Address::from_str(key).unwrap();
        let balance = U256::from_dec_str(&value.balance).unwrap();
        let empty_input = Rc::new(Vec::new());

        let vm: SeqTransactionVM<EIP160Patch<MordenAccountPatch>> = stateful.execute(ValidTransaction {
            caller: None,
            gas_price: Gas::zero(),
            gas_limit: Gas::from(100000u64),
            action: TransactionAction::Call(address),
            value: balance,
            input: empty_input.clone(),
            nonce: U256::zero(),
        }, HeaderParams {
            beneficiary: Address::default(),
            timestamp: 0,
            number: U256::zero(),
            difficulty: U256::zero(),
            gas_limit: Gas::max_value()
        }, &[]);
        match vm.status() {
            VMStatus::ExitedOk => (),
            _ => panic!(),
        }
    }

    assert_eq!(stateful.root(), H256::from("0xf3f4696bbf3b3b07775128eb7a3763279a394e382130f27c21e70233e04946a9"));
}

#[test]
fn genesis_state_root() {
    let database = MemoryDatabase::default();
    let mut stateful = MemoryStateful::empty(&database);
    let mut rng = rand::thread_rng();

    let mut accounts: Vec<(&String, &JSONAccount)> = GENESIS_ACCOUNTS.iter().collect();
    rng.shuffle(&mut accounts);
    let empty_input = Rc::new(Vec::new());

    for (key, value) in accounts {
        let address = Address::from_str(key).unwrap();
        let balance = U256::from_dec_str(&value.balance).unwrap();

        let vm: SeqTransactionVM<MainnetEIP160Patch> = stateful.execute(ValidTransaction {
            caller: None,
            gas_price: Gas::zero(),
            gas_limit: Gas::from(100000u64),
            action: TransactionAction::Call(address),
            value: balance,
            input: empty_input.clone(),
            nonce: U256::zero(),
        }, HeaderParams {
            beneficiary: Address::default(),
            timestamp: 0,
            number: U256::zero(),
            difficulty: U256::zero(),
            gas_limit: Gas::max_value()
        }, &[]);
        match vm.status() {
            VMStatus::ExitedOk => (),
            _ => panic!(),
        }
    }

    assert_eq!(stateful.root(), H256::from("0xd7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544"));
}
