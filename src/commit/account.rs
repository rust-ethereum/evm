//! Account commitment managment

#[cfg(not(feature = "std"))]
use alloc::Vec;

#[cfg(feature = "std")] use std::collections::{HashSet as Set, HashMap as Map, hash_map as map};
#[cfg(feature = "std")] use std::marker::PhantomData;
#[cfg(not(feature = "std"))] use alloc::{BTreeSet as Set, BTreeMap as Map, btree_map as map};
#[cfg(not(feature = "std"))] use core::marker::PhantomData;
use bigint::{M256, U256, Address};
use patch::AccountPatch;

#[cfg(not(feature = "std"))] use alloc::rc::Rc;
#[cfg(feature = "std")] use std::rc::Rc;

use errors::{RequireError, CommitError};

/// Internal representation of an account storage. It will return a
/// `RequireError` if trying to access non-existing storage.
#[derive(Debug, Clone)]
pub struct Storage {
    partial: bool,
    address: Address,
    storage: Map<U256, M256>,
}

impl Into<Map<U256, M256>> for Storage {
    fn into(self) -> Map<U256, M256> {
        self.storage
    }
}

impl Storage {
    /// Create a new storage.
    fn new(address: Address, partial: bool) -> Self {
        Storage {
            partial: partial,
            address: address,
            storage: Map::new(),
        }
    }

    /// Commit a value into the storage.
    fn commit(&mut self, index: U256, value: M256) -> Result<(), CommitError> {
        if !self.partial {
            return Err(CommitError::InvalidCommitment);
        }

        if self.storage.contains_key(&index) {
            return Err(CommitError::AlreadyCommitted);
        }

        self.storage.insert(index, value);
        Ok(())
    }

    /// Read a value from the storage.
    pub fn read(&self, index: U256) -> Result<M256, RequireError> {
        match self.storage.get(&index) {
            Some(&v) => Ok(v),
            None => if self.partial {
                Err(RequireError::AccountStorage(self.address, index))
            } else {
                Ok(M256::zero())
            }
        }
    }

    /// Write a value into the storage.
    pub fn write(&mut self, index: U256, value: M256) -> Result<(), RequireError> {
        if !self.storage.contains_key(&index) && self.partial {
            return Err(RequireError::AccountStorage(self.address, index));
        }
        self.storage.insert(index, value);
        Ok(())
    }

    /// Return the number of changed/full items in storage.
    pub fn len(&self) -> usize {
        self.storage.len()
    }
}

#[derive(Debug, Clone)]
/// A single account commitment.
pub enum AccountCommitment {
    /// Full account commitment. The client that committed account
    /// should not change the account in other EVMs if it decides to
    /// accept the result.
    Full {
        /// Nonce of the account.
        nonce: U256,
        /// Account address.
        address: Address,
        /// Account balance.
        balance: U256,
        /// Code associated with this account.
        code: Rc<Vec<u8>>,
    },
    /// Commit only code of the account. The client can keep changing
    /// it in other EVMs if the code remains unchanged.
    Code {
        /// Account address.
        address: Address,
        /// Code associated with this account.
        code: Rc<Vec<u8>>,
    },
    /// Commit a storage. Must be used given a full account.
    Storage {
        /// Account address.
        address: Address,
        /// Account storage index.
        index: U256,
        /// Value at the given account storage index.
        value: M256,
    },
    /// Indicate that an account does not exist, or is a suicided
    /// account.
    Nonexist(Address),
}

impl AccountCommitment {
    /// Address of this account commitment.
    pub fn address(&self) -> Address {
        match self {
            &AccountCommitment::Full {
                address,
                ..
            } => address,
            &AccountCommitment::Code {
                address,
                ..
            } => address,
            &AccountCommitment::Storage {
                address,
                ..
            } => address,
            &AccountCommitment::Nonexist(address) => address,
        }
    }
}

#[derive(Debug, Clone)]
/// Represents an account. This is usually returned by the EVM.
pub enum AccountChange {
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
        code: Rc<Vec<u8>>,
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
        code: Rc<Vec<u8>>
    },
    /// The account should remain nonexist, or should be deleted if
    /// exists.
    Nonexist(Address)
}

impl AccountChange {
    /// Address of this account.
    pub fn address(&self) -> Address {
        match self {
            &AccountChange::Full {
                address,
                ..
            } => address,
            &AccountChange::IncreaseBalance(address, _) => address,
            &AccountChange::Create {
                address,
                ..
            } => address,
            &AccountChange::Nonexist(address) => address,
        }
    }
}

#[derive(Debug)]
/// A struct that manages the current account state for one EVM.
pub struct AccountState<A: AccountPatch> {
    accounts: Map<Address, AccountChange>,
    codes: Map<Address, Rc<Vec<u8>>>,
    _marker: PhantomData<A>,
}

impl<A: AccountPatch> Default for AccountState<A> {
    fn default() -> Self {
        Self {
            accounts: Map::new(),
            codes: Map::new(),
            _marker: PhantomData,
        }
    }
}

impl<A: AccountPatch> Clone for AccountState<A> {
    fn clone(&self) -> Self {
        Self {
            accounts: self.accounts.clone(),
            codes: self.codes.clone(),
            _marker: PhantomData,
        }
    }
}

fn is_empty(nonce: U256, balance: U256, code: &[u8]) -> bool {
    nonce == U256::zero() && balance == U256::zero() && code == &[]
}

impl<A: AccountPatch> AccountState<A> {
    fn insert_account(&mut self, account: AccountChange) {
        match account {
            AccountChange::Full { nonce, address, balance, changing_storage, code } => {
                if (!A::empty_considered_exists()) && is_empty(nonce, balance, &code)
                {
                    self.accounts.insert(address, AccountChange::Nonexist(address));
                } else {
                    self.accounts.insert(address, AccountChange::Full {
                        nonce, address, balance, changing_storage, code
                    });
                }
            },
            AccountChange::Create { nonce, address, balance, storage, code } => {
                if (!A::empty_considered_exists()) && is_empty(nonce, balance, &code)
                {
                    self.accounts.insert(address, AccountChange::Nonexist(address));
                } else {
                    self.accounts.insert(address, AccountChange::Create {
                        nonce, address, balance, storage, code
                    });
                }
            },
            AccountChange::Nonexist(address) => {
                self.accounts.insert(address, AccountChange::Nonexist(address));
            },
            AccountChange::IncreaseBalance(address, balance) => {
                if A::allow_partial_change() {
                    self.accounts.insert(address, AccountChange::IncreaseBalance(address, balance));
                } else {
                    panic!()
                }
            },
        }
    }

    /// Returns all fetched or modified addresses.
    pub fn used_addresses(&self) -> Set<Address> {
        let mut set = Set::new();
        for account in self.accounts() {
            set.insert(account.address());
        }
        for (address, _) in &self.codes {
            set.insert(*address);
        }
        set
    }

    /// Returns all accounts right now in this account state.
    pub fn accounts(&self) -> map::Values<Address, AccountChange> {
        self.accounts.values()
    }

    /// Returns Ok(()) if a full account is in this account
    /// state. Otherwise raise a `RequireError`.
    pub fn require(&self, address: Address) -> Result<(), RequireError> {
        match self.accounts.get(&address) {
            Some(&AccountChange::Full { .. }) => return Ok(()),
            Some(&AccountChange::Create { .. }) => return Ok(()),
            Some(&AccountChange::Nonexist(_)) => return Ok(()),
            _ => return Err(RequireError::Account(address)),
        }
    }

    /// Returns Ok(()) if either a full account or a partial code
    /// account is in this account state. Otherwise raise a
    /// `RequireError`.
    pub fn require_code(&self, address: Address) -> Result<(), RequireError> {
        if self.codes.contains_key(&address) {
            return Ok(());
        }
        match self.accounts.get(&address) {
            Some(&AccountChange::Full { .. }) => return Ok(()),
            Some(&AccountChange::Create { .. }) => return Ok(()),
            Some(&AccountChange::Nonexist(_)) => return Ok(()),
            _ => return Err(RequireError::AccountCode(address)),
        }
    }

    /// Returns Ok(()) if the storage exists in the VM. Otherwise
    /// raise a `RequireError`.
    pub fn require_storage(&self, address: Address, index: U256) -> Result<(), RequireError> {
        self.storage_read(address, index).and_then(|_| Ok(()))
    }

    /// Commit an account commitment into this account state.
    pub fn commit(&mut self, commitment: AccountCommitment) -> Result<(), CommitError> {
        match commitment {
            AccountCommitment::Full {
                nonce,
                address,
                balance,
                code
            } => {
                let account = if self.accounts.contains_key(&address) {
                    match self.accounts.remove(&address).unwrap() {
                        AccountChange::Full { .. } => return Err(CommitError::AlreadyCommitted),
                        AccountChange::Create { .. } => return Err(CommitError::AlreadyCommitted),
                        AccountChange::Nonexist(_) => return Err(CommitError::AlreadyCommitted),
                        AccountChange::IncreaseBalance(address, topup) => {
                            AccountChange::Full {
                                nonce,
                                address,
                                balance: balance + topup,
                                changing_storage: Storage::new(address, true),
                                code,
                            }
                        },
                    }
                } else {
                    AccountChange::Full {
                        nonce,
                        address,
                        balance,
                        changing_storage: Storage::new(address, true),
                        code,
                    }
                };

                self.insert_account(account);
                self.codes.remove(&address);
            },
            AccountCommitment::Code {
                address,
                code,
            } => {
                if self.accounts.contains_key(&address) || self.codes.contains_key(&address) {
                    return Err(CommitError::AlreadyCommitted);
                }

                self.codes.insert(address, code);
            },
            AccountCommitment::Storage {
                address,
                index,
                value
            } => {
                match self.accounts.get_mut(&address) {
                    Some(&mut AccountChange::Full {
                        ref mut changing_storage,
                        ..
                    }) => {
                        changing_storage.commit(index, value)?;
                    },
                    _ => {
                        return Err(CommitError::InvalidCommitment);
                    },
                }
            },
            AccountCommitment::Nonexist(address) => {
                let account = if self.accounts.contains_key(&address) {
                    match self.accounts.remove(&address).unwrap() {
                        AccountChange::Full { .. } => return Err(CommitError::AlreadyCommitted),
                        AccountChange::Create { .. } => return Err(CommitError::AlreadyCommitted),
                        AccountChange::Nonexist(_) => return Err(CommitError::AlreadyCommitted),
                        AccountChange::IncreaseBalance(address, topup) => {
                            AccountChange::Create {
                                nonce: A::initial_nonce(),
                                address,
                                balance: topup,
                                storage: Storage::new(address, false),
                                code: Rc::new(Vec::new())
                            }
                        },
                    }
                } else {
                    AccountChange::Nonexist(address)
                };

                self.insert_account(account);
                self.codes.remove(&address);
            }
        }
        Ok(())
    }

    /// Test whether an account at given address is considered
    /// existing.
    pub fn exists(&self, address: Address) -> Result<bool, RequireError> {
        match self.accounts.get(&address) {
            Some(&AccountChange::Create { nonce, balance, ref code, .. }) => Ok(
                if A::empty_considered_exists() {
                    true
                } else {
                    is_empty(nonce, balance, code)
                }),
            Some(&AccountChange::Full { nonce, balance, ref code, .. }) => Ok(
                if A::empty_considered_exists() {
                    true
                } else {
                    is_empty(nonce, balance, code)
                }),
            Some(&AccountChange::Nonexist(_)) => Ok(false),
            Some(&AccountChange::IncreaseBalance(address, topup)) =>
                if A::empty_considered_exists() {
                    Ok(true)
                } else if topup > U256::zero() {
                    Ok(true)
                } else {
                    Err(RequireError::Account(address))
                },
            _ => Err(RequireError::Account(address)),
        }
    }

    /// Find code by its address in this account state. If the search
    /// failed, returns a `RequireError`.
    pub fn code(&self, address: Address) -> Result<Rc<Vec<u8>>, RequireError> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get(&address).unwrap() {
                &AccountChange::Full { ref code, .. } => return Ok(code.clone()),
                &AccountChange::Create { ref code, .. } => return Ok(code.clone()),
                &AccountChange::Nonexist(_) => return Ok(Rc::new(Vec::new())),
                &AccountChange::IncreaseBalance(_, _) => (),
            }
        }

        if self.codes.contains_key(&address) {
            return Ok(self.codes.get(&address).unwrap().clone());
        } else {
            return Err(RequireError::AccountCode(address));
        }
    }

    /// Find nonce by its address in this account state. If the search
    /// failed, returns a `RequireError`.
    pub fn nonce(&self, address: Address) -> Result<U256, RequireError> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get(&address).unwrap() {
                &AccountChange::Full {
                    nonce,
                    ..
                } => return Ok(nonce),
                &AccountChange::Create {
                    nonce,
                    ..
                } => return Ok(nonce),
                &AccountChange::Nonexist(_) => {
                    return Ok(A::initial_nonce());
                },
                _ => (),
            }
        }

        return Err(RequireError::Account(address));
    }

    /// Find balance by its address in this account state. If the
    /// search failed, returns a `RequireError`.
    pub fn balance(&self, address: Address) -> Result<U256, RequireError> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get(&address).unwrap() {
                &AccountChange::Full {
                    balance,
                    ..
                } => return Ok(balance),
                &AccountChange::Create {
                    balance,
                    ..
                } => return Ok(balance),
                &AccountChange::Nonexist(_) => {
                    return Ok(U256::zero());
                },
                _ => (),
            }
        }

        return Err(RequireError::Account(address));
    }

    /// Read a value from an account storage.
    pub fn storage_read(&self, address: Address, index: U256) -> Result<M256, RequireError> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get(&address).unwrap() {
                &AccountChange::Full {
                    ref changing_storage,
                    ..
                } => return changing_storage.read(index),
                &AccountChange::Create {
                    ref storage,
                    ..
                } => return storage.read(index),
                &AccountChange::Nonexist(_) => return Ok(M256::zero()),
                _ => (),
            }
        }

        return Err(RequireError::Account(address));
    }

    /// Write a value from an account storage. The account will be
    /// created if it is nonexist.
    pub fn storage_write(&mut self, address: Address, index: U256, value: M256) -> Result<(), RequireError> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get_mut(&address).unwrap() {
                &mut AccountChange::Full {
                    ref mut changing_storage,
                    ..
                } => return changing_storage.write(index, value),
                &mut AccountChange::Create {
                    ref mut storage,
                    ..
                } => return storage.write(index, value),
                val => {
                    let is_nonexist;
                    match val {
                        &mut AccountChange::Nonexist(_) => { is_nonexist = true; }
                        _ => { is_nonexist = false; }
                    }
                    if is_nonexist {
                        let mut storage = Storage::new(address, false);
                        let ret = storage.write(index, value);
                        *val = AccountChange::Create {
                            nonce: A::initial_nonce(),
                            address,
                            balance: U256::zero(),
                            storage,
                            code: Rc::new(Vec::new())
                        };
                        return ret;
                    }
                }
            }
        }

        return Err(RequireError::Account(address));
    }

    /// Create a new account (that should not yet have existed
    /// before).
    pub fn create(&mut self, address: Address, topup: U256) -> Result<(), RequireError> {
        let account = if self.accounts.contains_key(&address) {
            match self.accounts.remove(&address).unwrap() {
                AccountChange::Full { balance, .. } => {
                    AccountChange::Create {
                        address, code: Rc::new(Vec::new()), nonce: A::initial_create_nonce(),
                        balance: balance + topup, storage: Storage::new(address, false),
                    }
                },
                AccountChange::Create { balance, .. } => {
                    AccountChange::Create {
                        address, code: Rc::new(Vec::new()), nonce: A::initial_create_nonce(),
                        balance: balance + topup, storage: Storage::new(address, false),
                    }
                },
                AccountChange::Nonexist(_) => {
                    AccountChange::Create {
                        address, code: Rc::new(Vec::new()), nonce: A::initial_create_nonce(),
                        balance: topup, storage: Storage::new(address, false),
                    }
                },
                _ => {
                    // Although creation will clean up storage and
                    // code, the balance will be added if it was
                    // existing in prior. So if it is IncreaseBalance
                    // or DecreaseBalance, we need to ask for the
                    // account first.
                    return Err(RequireError::Account(address));
                },
            }
        } else {
            return Err(RequireError::Account(address));
        };

        self.codes.remove(&address);
        self.insert_account(account);

        Ok(())
    }

    /// Deposit code in to a created account. Only usable in a newly
    /// created account.
    pub fn code_deposit(&mut self, address: Address, new_code: Rc<Vec<u8>>) {
        match self.accounts.get_mut(&address).unwrap() {
            &mut AccountChange::Create { ref mut code, .. } => {
                *code = new_code;
            },
            _ => panic!(),
        }
    }

    /// Increase the balance of an account. The account will be
    /// created if it is nonexist in the beginning.
    pub fn increase_balance(&mut self, address: Address, topup: U256) {
        let account = match self.accounts.remove(&address) {
            Some(AccountChange::Full {
                address,
                balance,
                changing_storage,
                code,
                nonce,
            }) => {
                AccountChange::Full {
                    address,
                    balance: balance + topup,
                    changing_storage,
                    code,
                    nonce,
                }
            },
            Some(AccountChange::IncreaseBalance(address, balance)) => {
                AccountChange::IncreaseBalance(address, balance + topup)
            },
            Some(AccountChange::Create {
                address,
                balance,
                storage,
                code,
                nonce,
            }) => {
                AccountChange::Create {
                    address,
                    balance: balance + topup,
                    storage,
                    code,
                    nonce,
                }
            },
            Some(AccountChange::Nonexist(address)) => {
                AccountChange::Create {
                    nonce: A::initial_nonce(),
                    address,
                    balance: topup,
                    storage: Storage::new(address, false),
                    code: Rc::new(Vec::new())
                }
            },
            None => {
                AccountChange::IncreaseBalance(address, topup)
            },
        };
        self.insert_account(account);
    }

    /// Decrease the balance of an account. The account will be
    /// created if it is nonexist in the beginning.
    pub fn decrease_balance(&mut self, address: Address, withdraw: U256) {
        let account = match self.accounts.remove(&address) {
            Some(AccountChange::Full {
                address,
                balance,
                changing_storage,
                code,
                nonce,
            }) => {
                AccountChange::Full {
                    address,
                    balance: balance - withdraw,
                    changing_storage,
                    code,
                    nonce,
                }
            },
            Some(AccountChange::Create {
                address,
                balance,
                storage,
                code,
                nonce,
            }) => {
                AccountChange::Create {
                    address,
                    balance: balance - withdraw,
                    storage,
                    code,
                    nonce,
                }
            },
            Some(AccountChange::IncreaseBalance(_, _)) => panic!(),
            Some(AccountChange::Nonexist(_)) => panic!(),
            None => panic!(),
        };
        self.insert_account(account);
    }

    /// Set nonce of an account. If the account is not already
    /// commited, returns a `RequireError`. The account will be
    /// created if it is nonexist in the beginning.
    pub fn set_nonce(&mut self, address: Address, new_nonce: U256) -> Result<(), RequireError> {
        match self.accounts.get_mut(&address) {
            Some(&mut AccountChange::Full {
                ref mut nonce,
                ..
            }) => {
                *nonce = new_nonce;
                Ok(())
            },
            Some(&mut AccountChange::Create {
                ref mut nonce,
                ..
            }) => {
                *nonce = new_nonce;
                Ok(())
            },
            Some(val) => {
                let is_nonexist;
                match val {
                    &mut AccountChange::Nonexist(_) => { is_nonexist = true; }
                    _ => { is_nonexist = false; }
                }
                if is_nonexist {
                    *val = AccountChange::Create {
                        nonce: new_nonce,
                        address,
                        balance: U256::zero(),
                        storage: Storage::new(address, false),
                        code: Rc::new(Vec::new())
                    };
                    Ok(())
                } else {
                    Err(RequireError::Account(address))
                }
            }
            None => {
                Err(RequireError::Account(address))
            },
        }
    }

    /// Delete an account from this account state. The account is set
    /// to null.
    pub fn remove(&mut self, address: Address) -> Result<(), RequireError> {
        self.codes.remove(&address);
        self.insert_account(AccountChange::Nonexist(address));
        Ok(())
    }
}
