#![deny(unused_import_braces, unused_imports,
        unused_comparisons, unused_must_use,
        unused_variables, non_shorthand_field_patterns,
        unreachable_code)]

extern crate trie;
extern crate evm;
extern crate sha3;
extern crate block;
extern crate rlp;
extern crate bigint;

use bigint::{H256, U256, M256, Address};
use evm::{ValidTransaction, HeaderParams, Memory, TransactionVM, VM,
          AccountCommitment, Patch, AccountState, AccountChange};
use evm::errors::{PreExecutionError, RequireError};
use sha3::{Keccak256, Digest};
use trie::{FixedSecureTrie, DatabaseGuard, MemoryDatabase, Database, DatabaseOwned};
use block::{Account, Transaction};
use std::collections::HashMap;
use std::cmp::min;
use std::rc::Rc;
use std::ops::Deref;

pub struct LiteralAccount {
    pub nonce: U256,
    pub balance: U256,
    pub storage: HashMap<U256, M256>,
    pub code: Vec<u8>,
}

#[derive(Debug)]
pub struct Stateful<'a, D: 'a> {
    database: &'a D,
    root: H256,
}

impl<'a, D: 'a> Clone for Stateful<'a, D> {
    fn clone(&self) -> Self {
        Self {
            database: self.database.clone(),
            root: self.root.clone(),
        }
    }
}

impl<'a, D> Stateful<'a, D> {
    pub fn new(database: &'a D, root: H256) -> Self {
        Self {
            database,
            root
        }
    }

    pub fn empty(database: &'a D) -> Self {
        Self::new(database, MemoryDatabase::new().create_empty().root())
    }
}

impl<'b, D: DatabaseOwned> Stateful<'b, D> {
    fn is_empty_hash(hash: H256) -> bool {
        hash == H256::from(Keccak256::digest(&[]).as_slice())
    }

    pub fn database(&self) -> &'b D {
        self.database
    }

    pub fn code(&self, hash: H256) -> Option<Vec<u8>> {
        let code_hashes = self.database.create_guard();

        if Self::is_empty_hash(hash) {
            Some(Vec::new())
        } else {
            code_hashes.get(hash)
        }
    }

    pub fn step<V: VM>(
        &self, vm: &mut V, block_number: U256, most_recent_block_hashes: &[H256]
    ) {
        assert!(U256::from(most_recent_block_hashes.len()) >=
                min(block_number, U256::from(256)));

        let state = self.database.create_fixed_secure_trie(self.root);
        let code_hashes = self.database.create_guard();

        loop {
            match vm.step() {
                Ok(()) => break,
                Err(RequireError::Account(address)) => {
                    let account: Option<Account> = state.get(&address);

                    match account {
                        Some(account) => {
                            let code = if Self::is_empty_hash(account.code_hash) {
                                Vec::new()
                            } else {
                                code_hashes.get(account.code_hash).unwrap()
                            };

                            vm.commit_account(AccountCommitment::Full {
                                nonce: account.nonce,
                                address: address,
                                balance: account.balance,
                                code: Rc::new(code),
                            }).unwrap();
                        },
                        None => {
                            vm.commit_account(AccountCommitment::Nonexist(address)).unwrap();
                        },
                    }
                },
                Err(RequireError::AccountCode(address)) => {
                    let account: Option<Account> = state.get(&address);

                    match account {
                        Some(account) => {
                            let code = if Self::is_empty_hash(account.code_hash) {
                                Vec::new()
                            } else {
                                code_hashes.get(account.code_hash).unwrap()
                            };

                            vm.commit_account(AccountCommitment::Code {
                                address: address,
                                code: Rc::new(code),
                            }).unwrap();
                        },
                        None => {
                            vm.commit_account(AccountCommitment::Nonexist(address)).unwrap();
                        },
                    }
                },
                Err(RequireError::AccountStorage(address, index)) => {
                    let account: Option<Account> = state.get(&address);

                    match account {
                        Some(account) => {
                            let storage = self.database.create_fixed_secure_trie(account.storage_root);
                            let value = storage.get(&H256::from(index)).unwrap_or(M256::zero());

                            vm.commit_account(AccountCommitment::Storage {
                                address: address,
                                index, value
                            }).unwrap();
                        },
                        None => {
                            vm.commit_account(AccountCommitment::Nonexist(address)).unwrap();
                        },
                    }
                },
                Err(RequireError::Blockhash(number)) => {
                    let index = (block_number - number).as_usize();
                    vm.commit_blockhash(number, most_recent_block_hashes[index]).unwrap();
                },
            }
        }
    }

    pub fn require_to_commit(
        &self, require: RequireError, root: Option<H256>
    ) -> AccountCommitment {
        let state = self.database.create_fixed_secure_trie(root.unwrap_or(self.root));

        match require {
            RequireError::Account(address) => {
                let account: Option<Account> = state.get(&address);

                match account {
                    Some(account) => {
                        let code = if Self::is_empty_hash(account.code_hash) {
                            Vec::new()
                        } else {
                            self.code(account.code_hash).unwrap()
                        };

                        AccountCommitment::Full {
                            nonce: account.nonce,
                            address: address,
                            balance: account.balance,
                            code: Rc::new(code),
                        }
                    },
                    None => {
                        AccountCommitment::Nonexist(address)
                    },
                }
            },
            RequireError::AccountCode(address) => {
                let account: Option<Account> = state.get(&address);

                match account {
                    Some(account) => {
                        let code = if Self::is_empty_hash(account.code_hash) {
                            Vec::new()
                        } else {
                            self.code(account.code_hash).unwrap()
                        };

                        AccountCommitment::Code {
                            address: address,
                            code: Rc::new(code),
                        }
                    },
                    None => {
                        AccountCommitment::Nonexist(address)
                    },
                }
            },
            RequireError::AccountStorage(address, index) => {
                let account: Option<Account> = state.get(&address);

                match account {
                    Some(account) => {
                        let storage = self.database.create_fixed_secure_trie(account.storage_root);
                        let value = storage.get(&H256::from(index)).unwrap_or(M256::zero());

                        AccountCommitment::Storage {
                            address: address,
                            index, value
                        }
                    },
                    None => {
                        AccountCommitment::Nonexist(address)
                    },
                }
            },
            RequireError::Blockhash(_) => panic!(),
        }
    }

    pub fn call<M: Memory + Default, P: Patch>(
        &self, transaction: ValidTransaction, block: HeaderParams,
        most_recent_block_hashes: &[H256]
    ) -> TransactionVM<M, P> {
        assert!(U256::from(most_recent_block_hashes.len()) >=
                min(block.number, U256::from(256)));

        let mut vm = TransactionVM::new(transaction, block.clone());
        let state = self.database.create_fixed_secure_trie(self.root);
        let code_hashes = self.database.create_guard();

        loop {
            match vm.fire() {
                Ok(()) => break,
                Err(RequireError::Account(address)) => {
                    let account: Option<Account> = state.get(&address);

                    match account {
                        Some(account) => {
                            let code = if Self::is_empty_hash(account.code_hash) {
                                Vec::new()
                            } else {
                                code_hashes.get(account.code_hash).unwrap()
                            };

                            vm.commit_account(AccountCommitment::Full {
                                nonce: account.nonce,
                                address: address,
                                balance: account.balance,
                                code: Rc::new(code),
                            }).unwrap();
                        },
                        None => {
                            vm.commit_account(AccountCommitment::Nonexist(address)).unwrap();
                        },
                    }
                },
                Err(RequireError::AccountCode(address)) => {
                    let account: Option<Account> = state.get(&address);

                    match account {
                        Some(account) => {
                            let code = if Self::is_empty_hash(account.code_hash) {
                                Vec::new()
                            } else {
                                code_hashes.get(account.code_hash).unwrap()
                            };

                            vm.commit_account(AccountCommitment::Code {
                                address: address,
                                code: Rc::new(code),
                            }).unwrap();
                        },
                        None => {
                            vm.commit_account(AccountCommitment::Nonexist(address)).unwrap();
                        },
                    }
                },
                Err(RequireError::AccountStorage(address, index)) => {
                    let account: Option<Account> = state.get(&address);

                    match account {
                        Some(account) => {
                            let storage = self.database.create_fixed_secure_trie(account.storage_root);
                            let value = storage.get(&H256::from(index)).unwrap_or(M256::zero());

                            vm.commit_account(AccountCommitment::Storage {
                                address: address,
                                index, value
                            }).unwrap();
                        },
                        None => {
                            vm.commit_account(AccountCommitment::Nonexist(address)).unwrap();
                        },
                    }
                },
                Err(RequireError::Blockhash(number)) => {
                    let index = (block.number - number).as_usize();
                    vm.commit_blockhash(number, most_recent_block_hashes[index]).unwrap();
                },
            }
        }

        vm
    }

    pub fn sets(
        &mut self, accounts: &[(Address, LiteralAccount)]
    ) {
        let mut state = self.database.create_fixed_secure_trie(self.root);
        let mut code_hashes = self.database.create_guard();

        for &(address, ref account) in accounts {
            let mut storage_trie = self.database.create_fixed_secure_empty();
            for (key, value) in &account.storage {
                if *value == M256::zero() {
                    storage_trie.remove(&H256::from(*key));
                } else {
                    storage_trie.insert(H256::from(*key), *value);
                }
            }

            let code_hash = H256::from(Keccak256::digest(&account.code).as_slice());
            code_hashes.set(code_hash, account.code.clone());

            let account = Account {
                nonce: account.nonce,
                balance: account.balance,
                storage_root: storage_trie.root(),
                code_hash
            };

            state.insert(address, account);
        }

        self.root = state.root();
    }

    pub fn transit(
        &mut self, accounts: &[AccountChange]
    ) {
        let mut state = self.database.create_fixed_secure_trie(self.root);
        let mut code_hashes = self.database.create_guard();

        for account in accounts {
            match account.clone() {
                AccountChange::Full {
                    nonce, address, balance, changing_storage, code
                } => {
                    let changing_storage: HashMap<U256, M256> = changing_storage.into();

                    let mut account: Account = state.get(&address).unwrap();

                    let mut storage_trie = self.database.create_fixed_secure_trie(account.storage_root);
                    for (key, value) in changing_storage {
                        if value == M256::zero() {
                            storage_trie.remove(&H256::from(key));
                        } else {
                            storage_trie.insert(H256::from(key), value);
                        }
                    }

                    account.balance = balance;
                    account.nonce = nonce;
                    account.storage_root = storage_trie.root();
                    assert!(account.code_hash == H256::from(Keccak256::digest(&code).as_slice()));

                    state.insert(address, account);
                },
                AccountChange::IncreaseBalance(address, value) => {
                    match state.get(&address) {
                        Some(mut account) => {
                            account.balance = account.balance + value;
                            state.insert(address, account);
                        },
                        None => {
                            let account = Account {
                                nonce: U256::zero(),
                                balance: value,
                                storage_root: self.database.create_empty().root(),
                                code_hash: H256::from(Keccak256::digest(&[]).as_slice())
                            };
                            state.insert(address, account);
                        }
                    }
                },
                AccountChange::Create {
                    nonce, address, balance, storage, code
                } => {
                    let storage: HashMap<U256, M256> = storage.into();

                    let mut storage_trie = self.database.create_fixed_secure_empty();
                    for (key, value) in storage {
                        if value == M256::zero() {
                            storage_trie.remove(&H256::from(key));
                        } else {
                            storage_trie.insert(H256::from(key), value);
                        }
                    }

                    let code_hash = H256::from(Keccak256::digest(&code).as_slice());
                    code_hashes.set(code_hash, code.deref().clone());

                    let account = Account {
                        nonce: nonce,
                        balance: balance,
                        storage_root: storage_trie.root(),
                        code_hash
                    };

                    state.insert(address, account);
                },
                AccountChange::Nonexist(address) => {
                    state.remove(&address);
                }
            }
        }

        self.root = state.root();
    }

    pub fn execute<M: Memory + Default, P: Patch>(
        &mut self, transaction: ValidTransaction, block: HeaderParams,
        most_recent_block_hashes: &[H256]
    ) -> TransactionVM<M, P> {
        let vm = self.call::<_, P>(transaction, block, most_recent_block_hashes);
        let mut accounts = Vec::new();
        for account in vm.accounts() {
            accounts.push(account.clone());
        }
        self.transit(&accounts);
        vm
    }

    pub fn to_valid<P: Patch>(
        &self, transaction: Transaction,
    ) -> Result<ValidTransaction, PreExecutionError> {
        let state = self.database.create_fixed_secure_trie(self.root);
        let code_hashes = self.database.create_guard();
        let mut account_state = AccountState::default();

        loop {
            match ValidTransaction::from_transaction::<P>(&transaction, &account_state) {
                Ok(val) => return val,
                Err(RequireError::Account(address)) => {
                    let account: Option<Account> = state.get(&address);

                    match account {
                        Some(account) => {
                            let code = if Self::is_empty_hash(account.code_hash) {
                                Vec::new()
                            } else {
                                code_hashes.get(account.code_hash).unwrap()
                            };

                            account_state.commit(AccountCommitment::Full {
                                nonce: account.nonce,
                                address: address,
                                balance: account.balance,
                                code: Rc::new(code),
                            }).unwrap();
                        },
                        None => {
                            account_state.commit(AccountCommitment::Nonexist(address)).unwrap();
                        },
                    }
                },
                Err(RequireError::AccountCode(address)) => {
                    let account: Option<Account> = state.get(&address);

                    match account {
                        Some(account) => {
                            let code = if Self::is_empty_hash(account.code_hash) {
                                Vec::new()
                            } else {
                                code_hashes.get(account.code_hash).unwrap()
                            };

                            account_state.commit(AccountCommitment::Code {
                                address: address,
                                code: Rc::new(code),
                            }).unwrap();
                        },
                        None => {
                            account_state.commit(AccountCommitment::Nonexist(address)).unwrap();
                        },
                    }
                },
                Err(RequireError::AccountStorage(address, index)) => {
                    let account: Option<Account> = state.get(&address);

                    match account {
                        Some(account) => {
                            let storage = self.database.create_fixed_secure_trie(account.storage_root);
                            let value = storage.get(&H256::from(index)).unwrap_or(M256::zero());

                            account_state.commit(AccountCommitment::Storage {
                                address: address,
                                index, value
                            }).unwrap();
                        },
                        None => {
                            account_state.commit(AccountCommitment::Nonexist(address)).unwrap();
                        },
                    }
                },
                Err(RequireError::Blockhash(_)) => {
                    panic!()
                },
            }
        }
    }

    pub fn root(&self) -> H256 {
        self.root
    }

    pub fn state_of<'a>(&'a self, root: H256) -> FixedSecureTrie<<D as Database<'a>>::Guard, Address, Account> {
        self.database.create_fixed_secure_trie::<Address, Account>(root)
    }

    pub fn state<'a>(&'a self) -> FixedSecureTrie<<D as Database<'a>>::Guard, Address, Account> {
        self.state_of(self.root())
    }

    pub fn storage_state_of<'a>(&'a self, root: H256) -> FixedSecureTrie<<D as Database<'a>>::Guard, H256, M256> {
        self.database.create_fixed_secure_trie::<H256, M256>(root)
    }

    pub fn storage_state<'a>(&'a self, address: Address) -> Option<FixedSecureTrie<<D as Database<'a>>::Guard, H256, M256>> {
        let state = self.state();
        let account = state.get(&address);

        match account {
            Some(account) => {
                Some(self.storage_state_of(account.storage_root))
            },
            None => None,
        }
    }
}

pub type MemoryStateful<'a> = Stateful<'a, MemoryDatabase>;
