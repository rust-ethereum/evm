extern crate trie;
extern crate block;
extern crate hexutil;
extern crate bigint;
extern crate evm;
extern crate evm_stateful;
#[macro_use] extern crate lazy_static;

use hexutil::*;
use block::TransactionAction;
use bigint::{Address, U256, Gas};
use evm::{AccountChange, HeaderParams, SeqTransactionVM, VM, Storage, MainnetEIP160Patch, ValidTransaction};
use trie::MemoryDatabase;
use evm_stateful::{MemoryStateful, LiteralAccount};
use std::thread;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::rc::Rc;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone)]
/// Represents an account. This is usually returned by the EVM.
pub enum SendableAccountChange {
    /// A full account. The client is expected to replace its own account state with this.
    Full {
        /// Account nonce.
        nonce: U256,
        /// Account address.
        address: Address,
        /// Account balance.
        balance: U256,
        /// Change storage with given indexes and values.
        changing_storage: Storage,
        /// Code associated with this account.
        code: Vec<u8>,
    },
    /// Only balance is changed, and it is increasing for this address.
    IncreaseBalance(Address, U256),
    /// Create or delete a (new) account.
    Create {
        /// Account nonce.
        nonce: U256,
        /// Account address.
        address: Address,
        /// Account balance.
        balance: U256,
        /// All storage values of this account, with given indexes and values.
        storage: Storage,
        /// Code associated with this account.
        code: Vec<u8>,
    },
    Nonexist(Address),
}

impl SendableAccountChange {
    /// Address of this account.
    pub fn address(&self) -> Address {
        match self {
            &SendableAccountChange::Full {
                address,
                ..
            } => address,
            &SendableAccountChange::IncreaseBalance(address, _) => address,
            &SendableAccountChange::Create {
                address,
                ..
            } => address,
            &SendableAccountChange::Nonexist(address) => address,
        }
    }
}

impl From<AccountChange> for SendableAccountChange {
    fn from(change: AccountChange) -> Self {
        match change {
            AccountChange::Full { nonce, address, balance, changing_storage, code } => {
                SendableAccountChange::Full {
                    nonce, address, balance, changing_storage,
                    code: code.deref().clone()
                }
            },
            AccountChange::IncreaseBalance(address, balance) =>
                SendableAccountChange::IncreaseBalance(address, balance),
            AccountChange::Create { nonce, address, balance, storage, code } => {
                SendableAccountChange::Create {
                    nonce, address, balance, storage,
                    code: code.deref().clone()
                }
            },
            AccountChange::Nonexist(address) => SendableAccountChange::Nonexist(address),
        }
    }
}

impl Into<AccountChange> for SendableAccountChange {
    fn into(self) -> AccountChange {
        match self {
            SendableAccountChange::Full { nonce, address, balance, changing_storage, code } => {
                AccountChange::Full {
                    nonce, address, balance, changing_storage,
                    code: Rc::new(code),
                }
            },
            SendableAccountChange::IncreaseBalance(address, balance) =>
                AccountChange::IncreaseBalance(address, balance),
            SendableAccountChange::Create { nonce, address, balance, storage, code } => {
                AccountChange::Create {
                    nonce, address, balance, storage,
                    code: Rc::new(code),
                }
            },
            SendableAccountChange::Nonexist(address) => AccountChange::Nonexist(address),
        }
    }
}

pub struct SendableValidTransaction {
    pub caller: Option<Address>,
    pub gas_price: Gas,
    pub gas_limit: Gas,
    pub action: TransactionAction,
    pub value: U256,
    pub input: Vec<u8>,
    pub nonce: U256,
}

impl From<ValidTransaction> for SendableValidTransaction {
    fn from(transaction: ValidTransaction) -> SendableValidTransaction {
        match transaction {
            ValidTransaction { caller, gas_price, gas_limit, action, value, input, nonce } => {
                SendableValidTransaction {
                    caller, gas_price, gas_limit, action, value, nonce,
                    input: input.deref().clone(),
                }
            }
        }
    }
}

impl Into<ValidTransaction> for SendableValidTransaction {
    fn into(self) -> ValidTransaction {
        match self {
            SendableValidTransaction { caller, gas_price, gas_limit, action, value, input, nonce } => {
                ValidTransaction {
                    caller, gas_price, gas_limit, action, value, nonce,
                    input: Rc::new(input),
                }
            }
        }
    }
}

fn is_modified(modified_addresses: &HashSet<Address>, accounts: &[SendableAccountChange]) -> bool {
    for account in accounts {
        if modified_addresses.contains(&account.address()) {
            return true;
        }
    }
    return false;
}

pub fn parallel_execute(
    stateful: MemoryStateful<'static>, transactions: &[ValidTransaction]
) -> MemoryStateful<'static> {
    let header = HeaderParams {
        beneficiary: Address::zero(),
        timestamp: 0,
        number: U256::zero(),
        difficulty: U256::zero(),
        gas_limit: Gas::max_value(),
    };

    let stateful = Arc::new(stateful);
    let mut threads = Vec::new();

    // Execute all transactions in parallel.
    for transaction in transactions {
        let transaction: SendableValidTransaction = transaction.clone().into();
        let header = header.clone();
        let stateful = stateful.clone();

        threads.push(thread::spawn(move || {
            let vm: SeqTransactionVM<MainnetEIP160Patch> = stateful.call(
                transaction.into(), header, &[]);
            let accounts: Vec<SendableAccountChange> = vm.accounts().map(|v| SendableAccountChange::from(v.clone())).collect();
            (accounts, vm.used_addresses())
        }));
    }

    // Join all results together.
    let mut thread_accounts = Vec::new();
    for thread in threads {
        let accounts = thread.join().unwrap();
        thread_accounts.push(accounts);
    }

    // Now we only have a single reference to stateful, unwrap it and
    // start the state transition.
    let mut stateful = match Arc::try_unwrap(stateful) {
        Ok(val) => val,
        Err(_) => panic!(),
    };
    let mut modified_addresses = HashSet::new();

    for (index, (accounts, used_addresses)) in thread_accounts.into_iter().enumerate() {
        let (accounts, used_addresses) = if is_modified(&modified_addresses, &accounts) {
            // Re-execute the transaction if conflict is detected.
            println!("Transaction index {}: conflict detected, re-execute.", index);
            let vm: SeqTransactionVM<MainnetEIP160Patch> = stateful.call(
                transactions[index].clone(), header.clone(), &[]);
            let accounts: Vec<AccountChange> = vm.accounts().map(|v| v.clone()).collect();
            (accounts, vm.used_addresses())
        } else {
            println!("Transaction index {}: parallel execution successful.", index);
            let accounts: Vec<AccountChange> = accounts.iter().map(|v| v.clone().into()).collect();
            (accounts, used_addresses)
        };

        stateful.transit(&accounts);
        modified_addresses.extend(used_addresses.into_iter());
    }

    stateful
}

lazy_static! {
    static ref DATABASE: MemoryDatabase = MemoryDatabase::default();
}

fn main() {
    let mut stateful = MemoryStateful::empty(&DATABASE);

    let addr1 = Address::from_str("0x0000000000000000000000000000000000001000").unwrap();
    let addr2 = Address::from_str("0x0000000000000000000000000000000000001001").unwrap();
    let addr3 = Address::from_str("0x0000000000000000000000000000000000001002").unwrap();

    // Input some initial accounts.
    stateful.sets(&[
        (addr1, LiteralAccount {
            nonce: U256::zero(),
            balance: U256::from_str("0x1000000000000000000").unwrap(),
            storage: HashMap::new(),
            code: read_hex("0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055").unwrap(),
        }),
        (addr2, LiteralAccount {
            nonce: U256::zero(),
            balance: U256::from_str("0x1000000000000000000").unwrap(),
            storage: HashMap::new(),
            code: read_hex("0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055").unwrap(),
        }),
        (addr3, LiteralAccount {
            nonce: U256::zero(),
            balance: U256::from_str("0x1000000000000000000").unwrap(),
            storage: HashMap::new(),
            code: read_hex("0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055").unwrap(),
        }),
    ]);

    // Execute several crafted transactions.
    let stateful = parallel_execute(stateful, &[
        ValidTransaction {
            caller: Some(addr1),
            action: TransactionAction::Call(addr1),
            gas_price: Gas::zero(),
            gas_limit: Gas::max_value(),
            value: U256::from_str("0x1000").unwrap(),
            input: Rc::new(Vec::new()),
            nonce: U256::zero(),
        },
        ValidTransaction {
            caller: Some(addr2),
            action: TransactionAction::Call(addr3),
            gas_price: Gas::zero(),
            gas_limit: Gas::max_value(),
            value: U256::from_str("0x1000").unwrap(),
            input: Rc::new(Vec::new()),
            nonce: U256::zero(),
        },
        ValidTransaction {
            caller: Some(addr3),
            action: TransactionAction::Call(addr2),
            gas_price: Gas::zero(),
            gas_limit: Gas::max_value(),
            value: U256::from_str("0x1000").unwrap(),
            input: Rc::new(Vec::new()),
            nonce: U256::zero(),
        },
    ]);

    println!("New state root: 0x{:x}", stateful.root());
}
