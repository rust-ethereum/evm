#![deny(unused_import_braces,
        unused_comparisons, unused_must_use,
        unused_variables, non_shorthand_field_patterns,
        unreachable_code)]

extern crate trie;
extern crate sputnikvm;
extern crate sha3;
extern crate block;
extern crate rlp;

use sputnikvm::{H256, U256, M256, Address};
use sputnikvm::vm::{self, ValidTransaction, HeaderParams, Memory, TransactionVM, VM,
                    AccountCommitment, Patch, SeqMemory};
use sputnikvm::vm::errors::RequireError;
use sha3::{Keccak256, Digest};
use trie::{Trie, SecureTrie, FixedSecureTrie, DatabaseGuard, MemoryDatabase, MemoryDatabaseGuard, Database, DatabaseOwned};
use block::Account;
use std::collections::HashMap;
use std::cmp::min;

pub struct Stateful<G, D> {
    database: D,
    code_hashes: G,
    root: H256,
}

impl<G, D> Stateful<G, D> {
    pub fn new(database: D, code_hashes: G, root: H256) -> Self {
        Self {
            database,
            code_hashes,
            root
        }
    }
}

impl<G: Default, D: Default> Default for Stateful<G, D> {
    fn default() -> Self {
        Self {
            database: D::default(),
            code_hashes: G::default(),
            root: MemoryDatabase::new().create_empty().root(),
        }
    }
}

impl<G: DatabaseGuard, D: DatabaseOwned> Stateful<G, D> {
    fn is_empty_hash(hash: H256) -> bool {
        hash == H256::from(Keccak256::digest(&[]).as_slice())
    }

    pub fn call<M: Memory + Default>(
        &self, transaction: ValidTransaction, block: HeaderParams,
        patch: &'static Patch, most_recent_block_hashes: &[H256]
    ) -> TransactionVM<M> {
        assert!(U256::from(most_recent_block_hashes.len()) >=
                min(block.number, U256::from(256)));

        let mut vm = TransactionVM::new(transaction, block.clone(), patch);
        let state = self.database.create_fixed_secure_trie(self.root);

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
                                self.code_hashes.get(account.code_hash).unwrap()
                            };

                            vm.commit_account(AccountCommitment::Full {
                                nonce: account.nonce,
                                address: address,
                                balance: account.balance,
                                code: code,
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
                                self.code_hashes.get(account.code_hash).unwrap()
                            };

                            vm.commit_account(AccountCommitment::Code {
                                address: address,
                                code: code,
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

    pub fn transit(
        &mut self, accounts: &[vm::Account]
    ) {
        let mut state = self.database.create_fixed_secure_trie(self.root);

        for account in accounts {
            match account.clone() {
                vm::Account::Full {
                    nonce, address, balance, changing_storage, code
                } => {
                    let changing_storage: HashMap<U256, M256> = changing_storage.into();

                    let mut account: Account = state.get(&address).unwrap();

                    let mut storage_trie = self.database.create_fixed_secure_trie(account.storage_root);
                    for (key, value) in changing_storage {
                        storage_trie.insert(H256::from(key), value);
                    }

                    account.balance = balance;
                    account.nonce = nonce;
                    account.storage_root = storage_trie.root();
                    assert!(account.code_hash == H256::from(Keccak256::digest(&code).as_slice()));

                    state.insert(address, account);
                },
                vm::Account::IncreaseBalance(address, value) => {
                    let mut account: Account = state.get(&address).unwrap();

                    account.balance = account.balance + value;

                    state.insert(address, account);
                },
                vm::Account::DecreaseBalance(address, value) => {
                    let mut account: Account = state.get(&address).unwrap();

                    account.balance = account.balance - value;

                    state.insert(address, account);
                },
                vm::Account::Create {
                    nonce, address, balance, storage, code, exists
                } => {
                    if !exists {
                        state.remove(&address);
                    } else {
                        let storage: HashMap<U256, M256> = storage.into();

                        let mut storage_trie = self.database.create_fixed_secure_empty();
                        for (key, value) in storage {
                            storage_trie.insert(H256::from(key), value);
                        }

                        let code_hash = H256::from(Keccak256::digest(&code).as_slice());
                        self.code_hashes.set(code_hash, code);

                        let account = Account {
                            nonce: nonce,
                            balance: balance,
                            storage_root: storage_trie.root(),
                            code_hash
                        };

                        state.insert(address, account);
                    }
                },
            }
        }

        self.root = state.root();
    }

    pub fn execute<M: Memory + Default>(
        &mut self, transaction: ValidTransaction, block: HeaderParams,
        patch: &'static Patch, most_recent_block_hashes: &[H256]
    ) -> TransactionVM<M> {
        let vm = self.call(transaction, block, patch, most_recent_block_hashes);
        let mut accounts = Vec::new();
        for account in vm.accounts() {
            accounts.push(account.clone());
        }
        self.transit(&accounts);
        vm
    }

    pub fn root(&self) -> H256 {
        self.root
    }

    pub fn state<'a>(&'a self) -> FixedSecureTrie<<D as Database<'a>>::Guard, Address, Account> {
        self.database.create_fixed_secure_trie::<Address, Account>(self.root())
    }
}

pub type MemoryStateful = Stateful<HashMap<H256, Vec<u8>>, MemoryDatabase>;
