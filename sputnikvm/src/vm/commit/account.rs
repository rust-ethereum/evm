//! Account commitment managment

use std::collections::hash_map::{self, HashMap};
use utils::address::Address;
use utils::bigint::{M256, U256};

use vm::Storage;
use vm::errors::{RequireError, CommitError};

#[derive(Debug, Clone)]
/// A single account commitment.
pub enum AccountCommitment {
    /// Full account commitment. The client that committed account
    /// should not change the account in other EVMs if it decides to
    /// accept the result.
    Full {
        nonce: M256,
        address: Address,
        balance: U256,
        code: Vec<u8>,
    },
    /// Commit only code of the account. The client can keep changing
    /// it in other EVMs if the code remains unchanged.
    Code {
        address: Address,
        code: Vec<u8>,
    },
    /// Commit a storage. Must be used given a full account.
    Storage {
        address: Address,
        index: M256,
        value: M256,
    },
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
        }
    }
}

#[derive(Debug, Clone)]
/// Represents an account. This is usually returned by the EVM.
pub enum Account {
    /// A full account. The client is expected to replace its own account state with this.
    Full {
        nonce: M256,
        address: Address,
        balance: U256,
        changing_storage: Storage,
        code: Vec<u8>,
    },
    /// Only balance is changed, and it is increasing for this address.
    IncreaseBalance(Address, U256),
    /// Only balance is changed, and it is decreasing for this address.
    DecreaseBalance(Address, U256),
    /// Remove an account, the client is expected to handle only
    /// removal without any ether transfer.
    Remove(Address),
    /// Create a new account.
    Create {
        nonce: M256,
        address: Address,
        balance: U256,
        storage: Storage,
        code: Vec<u8>,
    },
}

impl Account {
    /// Address of this account.
    pub fn address(&self) -> Address {
        match self {
            &Account::Full {
                address,
                ..
            } => address,
            &Account::IncreaseBalance(address, _) => address,
            &Account::DecreaseBalance(address, _) => address,
            &Account::Remove(address) => address,
            &Account::Create {
                address,
                ..
            } => address,
        }
    }
}

#[derive(Debug, Clone)]
/// A struct that manages the current account state for one EVM.
pub struct AccountState {
    accounts: HashMap<Address, Account>,
    codes: HashMap<Address, Vec<u8>>,
}

impl Default for AccountState {
    fn default() -> Self {
        Self {
            accounts: HashMap::new(),
            codes: HashMap::new(),
        }
    }
}

impl AccountState {
    /// Returns all accounts right now in this account state.
    pub fn accounts(&self) -> hash_map::Values<Address, Account> {
        self.accounts.values()
    }

    /// Returns Ok(()) if a full account is in this account
    /// state. Otherwise raise a `RequireError`.
    pub fn require(&self, address: Address) -> Result<(), RequireError> {
        match self.accounts.get(&address) {
            Some(&Account::Full { .. }) => return Ok(()),
            Some(&Account::Create { .. }) => return Ok(()),
            Some(&Account::Remove(_)) => panic!(),
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
            Some(&Account::Full { .. }) => return Ok(()),
            Some(&Account::Create { .. }) => return Ok(()),
            Some(&Account::Remove(_)) => panic!(),
            _ => return Err(RequireError::AccountCode(address)),
        }
    }

    /// Returns Ok(()) if the storage exists in the VM. Otherwise
    /// raise a `RequireError`.
    pub fn require_storage(&self, address: Address, index: M256) -> Result<(), RequireError> {
        self.storage(address)?.read(index).and_then(|_| Ok(()))
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
                        Account::Full { .. } => return Err(CommitError::AlreadyCommitted),
                        Account::Create { .. } => return Err(CommitError::AlreadyCommitted),
                        Account::Remove(address) => Account::Remove(address),
                        Account::IncreaseBalance(address, topup) => {
                            Account::Full {
                                nonce,
                                address,
                                balance: balance + topup,
                                changing_storage: Storage::new(address, true),
                                code,
                            }
                        },
                        Account::DecreaseBalance(address, withdraw) => {
                            Account::Full {
                                nonce,
                                address,
                                balance: balance - withdraw,
                                changing_storage: Storage::new(address, true),
                                code,
                            }
                        },
                    }
                } else {
                    Account::Full {
                        nonce,
                        address,
                        balance,
                        changing_storage: Storage::new(address, true),
                        code,
                    }
                };

                self.accounts.insert(address, account);
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
                    Some(&mut Account::Full {
                        ref mut changing_storage,
                        ..
                    }) => {
                        changing_storage.commit(index, value)?;
                    },
                    _ => {
                        return Err(CommitError::InvalidCommitment);
                    },
                }
            }
        }
        Ok(())
    }

    /// Find code by its address in this account state. If the search
    /// failed, returns a `RequireError`.
    pub fn code(&self, address: Address) -> Result<&[u8], RequireError> {
        if self.codes.contains_key(&address) {
            return Ok(self.codes.get(&address).unwrap().as_slice());
        }

        if self.accounts.contains_key(&address) {
            match self.accounts.get(&address).unwrap() {
                &Account::Full {
                    ref code,
                    ..
                } => return Ok(code.as_slice()),
                &Account::Create {
                    ref code,
                    ..
                } => return Ok(code.as_slice()),
                &Account::Remove(_) => panic!(),
                &Account::IncreaseBalance(address, _) => return Err(RequireError::Account(address)),
                &Account::DecreaseBalance(address, _) => return Err(RequireError::Account(address)),
            }
        }

        return Err(RequireError::AccountCode(address));
    }

    /// Find nonce by its address in this account state. If the search
    /// failed, returns a `RequireError`.
    pub fn nonce(&self, address: Address) -> Result<M256, RequireError> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get(&address).unwrap() {
                &Account::Full {
                    nonce,
                    ..
                } => return Ok(nonce),
                &Account::Create {
                    nonce,
                    ..
                } => return Ok(nonce),
                &Account::Remove(_) => panic!(),
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
                &Account::Full {
                    balance,
                    ..
                } => return Ok(balance),
                &Account::Create {
                    balance,
                    ..
                } => return Ok(balance),
                &Account::Remove(_) => return Ok(U256::zero()),
                _ => (),
            }
        }

        return Err(RequireError::Account(address));
    }

    /// Returns the storage of an account. If the account is not yet
    /// committed, returns a `RequireError`.
    pub fn storage(&self, address: Address) -> Result<&Storage, RequireError> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get(&address).unwrap() {
                &Account::Full {
                    ref changing_storage,
                    ..
                } => return Ok(changing_storage),
                &Account::Create {
                    ref storage,
                    ..
                } => return Ok(storage),
                &Account::Remove(_) => panic!(),
                _ => (),
            }
        }

        return Err(RequireError::Account(address));
    }

    /// Returns the mutable storage of an account. If the account is
    /// not yet committed. returns a `RequireError`.
    pub fn storage_mut(&mut self, address: Address) -> Result<&mut Storage, RequireError> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get_mut(&address).unwrap() {
                &mut Account::Full {
                    ref mut changing_storage,
                    ..
                } => return Ok(changing_storage),
                &mut Account::Create {
                    ref mut storage,
                    ..
                } => return Ok(storage),
                &mut Account::Remove(_) => panic!(),
                _ => (),
            }
        }

        return Err(RequireError::Account(address));
    }

    /// Create a new account (that should not yet have existed
    /// before).
    pub fn create(&mut self, address: Address, balance: U256) {
        let account = if self.accounts.contains_key(&address) {
            match self.accounts.remove(&address).unwrap() {
                Account::Full { .. } => panic!(),
                Account::Create { .. } => panic!(),
                Account::Remove(address) => {
                    Account::Create {
                        address, balance, code: Vec::new(), nonce: M256::zero(),
                        storage: Storage::new(address, false)
                    }
                },
                Account::IncreaseBalance(address, topup) => {
                    Account::Create {
                        address, code: Vec::new(), nonce: M256::zero(),
                        balance: balance + topup, storage: Storage::new(address, false)
                    }
                },
                Account::DecreaseBalance(address, withdraw) => {
                    Account::Create {
                        address, code: Vec::new(), nonce: M256::zero(),
                        balance: balance - withdraw, storage: Storage::new(address, false)
                    }
                },
            }
        } else {
            Account::Create {
                address, balance, code: Vec::new(), nonce: M256::zero(),
                storage: Storage::new(address, false)
            }
        };

        self.accounts.insert(address, account);
    }

    /// Deposit code in to a created account.
    pub fn code_deposit(&mut self, address: Address, new_code: &[u8]) {
        match self.accounts.get_mut(&address).unwrap() {
            &mut Account::Create { ref mut code, .. } => {
                *code = new_code.into();
            },
            _ => panic!(),
        }
    }

    /// Increase the balance of an account.
    pub fn increase_balance(&mut self, address: Address, topup: U256) {
        if topup == U256::zero() { return; }
        let account = match self.accounts.remove(&address) {
            Some(Account::Full {
                address,
                balance,
                changing_storage,
                code,
                nonce,
            }) => {
                Some(Account::Full {
                    address,
                    balance: balance + topup,
                    changing_storage,
                    code,
                    nonce,
                })
            },
            Some(Account::IncreaseBalance(address, balance)) => {
                Some(Account::IncreaseBalance(address, balance + topup))
            },
            Some(Account::DecreaseBalance(address, balance)) => {
                if balance == topup {
                    None
                } else if balance > topup {
                    Some(Account::DecreaseBalance(address, balance - topup))
                } else {
                    Some(Account::IncreaseBalance(address, topup - balance))
                }
            },
            Some(Account::Remove(_)) => {
                panic!()
            },
            Some(Account::Create {
                address,
                balance,
                storage,
                code,
                nonce,
            }) => {
                Some(Account::Create {
                    address,
                    balance: balance + topup,
                    storage,
                    code,
                    nonce,
                })
            },
            None => {
                Some(Account::IncreaseBalance(address, topup))
            },
        };
        if account.is_some() {
            self.accounts.insert(address, account.unwrap());
        }
    }

    /// Decrease the balance of an account.
    pub fn decrease_balance(&mut self, address: Address, withdraw: U256) {
        if withdraw == U256::zero() { return; }
        let account = match self.accounts.remove(&address) {
            Some(Account::Full {
                address,
                balance,
                changing_storage,
                code,
                nonce,
            }) => {
                Some(Account::Full {
                    address,
                    balance: balance - withdraw,
                    changing_storage,
                    code,
                    nonce,
                })
            },
            Some(Account::DecreaseBalance(address, balance)) => {
                Some(Account::DecreaseBalance(address, balance + withdraw))
            },
            Some(Account::IncreaseBalance(address, balance)) => {
                if balance == withdraw {
                    None
                } else if balance > withdraw {
                    Some(Account::IncreaseBalance(address, balance - withdraw))
                } else {
                    Some(Account::DecreaseBalance(address, withdraw - balance))
                }
            },
            Some(Account::Remove(_)) => {
                panic!()
            },
            Some(Account::Create {
                address,
                balance,
                storage,
                code,
                nonce,
            }) => {
                Some(Account::Create {
                    address,
                    balance: balance - withdraw,
                    storage,
                    code,
                    nonce,
                })
            },
            None => {
                Some(Account::DecreaseBalance(address, withdraw))
            },
        };
        if account.is_some() {
            self.accounts.insert(address, account.unwrap());
        }
    }

    /// Set nonce of an account. If the account is not already
    /// commited, returns a `RequireError`.
    pub fn set_nonce(&mut self, address: Address, new_nonce: M256) -> Result<(), RequireError> {
        match self.accounts.get_mut(&address) {
            Some(&mut Account::Full {
                ref mut nonce,
                ..
            }) => {
                *nonce = new_nonce;
                Ok(())
            },
            Some(&mut Account::Create {
                ref mut nonce,
                ..
            }) => {
                *nonce = new_nonce;
                Ok(())
            },
            Some(&mut Account::Remove(_)) => panic!(),
            _ => {
                Err(RequireError::Account(address))
            },
        }
    }

    /// Delete an account from this account state. The account is set
    /// to null. If the account is not already commited, returns a
    /// `RequireError`.
    pub fn remove(&mut self, address: Address) -> Result<(), RequireError> {
        let account = match self.accounts.remove(&address) {
            Some(Account::Full {
                address,
                ..
            }) => {
                Account::Remove(address)
            },
            Some(Account::DecreaseBalance(address, _)) => {
                return Err(RequireError::Account(address));
            },
            Some(Account::IncreaseBalance(address, _)) => {
                return Err(RequireError::Account(address));
            },
            Some(Account::Remove(_)) => {
                panic!()
            },
            Some(Account::Create {
                address,
                ..
            }) => {
                Account::Remove(address)
            },
            None => {
                return Err(RequireError::Account(address));
            },
        };
        self.accounts.insert(address, account);
        Ok(())
    }
}
