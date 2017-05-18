//! Account commitment managment

use std::collections::hash_map::{self, HashMap};
use utils::address::Address;
use utils::bigint::{M256, U256};

use vm::Storage;
use vm::errors::{RequireError, CommitError};

#[derive(Debug, Clone)]
/// A single account commitment.
pub enum AccountCommitment<S> {
    /// Full account commitment. The client that committed account
    /// should not change the account in other EVMs if it decides to
    /// accept the result.
    Full {
        nonce: M256,
        address: Address,
        balance: U256,
        storage: S,
        code: Vec<u8>,
    },
    /// Commit only code of the account. The client can keep changing
    /// it in other EVMs if the code remains unchanged.
    Code {
        address: Address,
        code: Vec<u8>,
    },
}

impl<S: Storage> AccountCommitment<S> {
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
        }
    }
}

#[derive(Debug, Clone)]
/// Represents an account. This is usually returned by the EVM.
pub enum Account<S> {
    /// A full account. The client is expected to replace its own account state with this.
    Full {
        nonce: M256,
        address: Address,
        balance: U256,
        storage: S,
        code: Vec<u8>,
    },
    /// Only balance is changed, and it is increasing for this address.
    IncreaseBalance(Address, U256),
    /// Only balance is changed, and it is decreasing for this address.
    DecreaseBalance(Address, U256),
}

impl<S: Storage> Account<S> {
    /// Address of this account.
    pub fn address(&self) -> Address {
        match self {
            &Account::Full {
                address,
                ..
            } => address,
            &Account::IncreaseBalance(address, _) => address,
            &Account::DecreaseBalance(address, _) => address,
        }
    }
}

#[derive(Debug, Clone)]
/// A struct that manages the current account state for one EVM.
pub struct AccountState<S> {
    accounts: HashMap<Address, Account<S>>,
    codes: HashMap<Address, Vec<u8>>,
}

impl<S: Storage> Default for AccountState<S> {
    fn default() -> Self {
        Self {
            accounts: HashMap::new(),
            codes: HashMap::new(),
        }
    }
}

impl<S: Storage + Default + Clone> AccountState<S> {
    /// Returns all accounts right now in this account state.
    pub fn accounts(&self) -> hash_map::Values<Address, Account<S>> {
        self.accounts.values()
    }

    /// Returns Ok(()) if a full account is in this account
    /// state. Otherwise raise a `RequireError`.
    pub fn require(&self, address: Address) -> Result<(), RequireError> {
        match self.accounts.get(&address) {
            Some(&Account::Full { .. }) => return Ok(()),
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
            _ => return Err(RequireError::AccountCode(address)),
        }
    }

    /// Commit an account commitment into this account state.
    pub fn commit(&mut self, commitment: AccountCommitment<S>) -> Result<(), CommitError> {
        match commitment {
            AccountCommitment::Full {
                nonce,
                address,
                balance,
                storage,
                code
            } => {
                if self.accounts.contains_key(&address) {
                    return Err(CommitError::AlreadyCommitted);
                }

                self.accounts.insert(address, Account::Full {
                    nonce,
                    address,
                    balance,
                    storage,
                    code,
                });
            },
            AccountCommitment::Code {
                address,
                code,
            } => {
                if self.accounts.contains_key(&address) || self.codes.contains_key(&address) {
                    return Err(CommitError::AlreadyCommitted);
                }

                self.codes.insert(address, code);
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
                _ => (),
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
                _ => (),
            }
        }

        return Err(RequireError::Account(address));
    }

    /// Returns the storage of an account. If the account is not yet
    /// committed, returns a `RequireError`.
    pub fn storage(&self, address: Address) -> Result<&S, RequireError> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get(&address).unwrap() {
                &Account::Full {
                    ref storage,
                    ..
                } => return Ok(storage),
                _ => (),
            }
        }

        return Err(RequireError::Account(address));
    }

    /// Returns the mutable storage of an account. If the account is
    /// not yet committed. returns a `RequireError`.
    pub fn storage_mut(&mut self, address: Address) -> Result<&mut S, RequireError> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get_mut(&address).unwrap() {
                &mut Account::Full {
                    ref mut storage,
                    ..
                } => return Ok(storage),
                _ => (),
            }
        }

        return Err(RequireError::Account(address));
    }

    /// Create a new account (that should not yet have existed
    /// before).
    pub fn create(&mut self, address: Address, balance: U256, code: &[u8]) {
        self.accounts.insert(address, Account::Full {
            address, balance, storage: S::default(), code: code.into(), nonce: M256::zero(),
        });
    }

    /// Increase the balance of an account.
    pub fn increase_balance(&mut self, address: Address, topup: U256) {
        if topup == U256::zero() { return; }
        let account = match self.accounts.remove(&address) {
            Some(Account::Full {
                address,
                balance,
                storage,
                code,
                nonce,
            }) => {
                Some(Account::Full {
                    address,
                    balance: balance + topup,
                    storage,
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
                storage,
                code,
                nonce,
            }) => {
                Some(Account::Full {
                    address,
                    balance: balance - withdraw,
                    storage,
                    code,
                    nonce,
                })
            },
            Some(Account::DecreaseBalance(address, balance)) => {
                Some(Account::DecreaseBalance(address, balance - withdraw))
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
                Account::Full {
                    address,
                    balance: U256::zero(),
                    storage: S::default(),
                    code: Vec::new(),
                    nonce: M256::zero(),
                }
            },
            Some(Account::DecreaseBalance(address, _)) => {
                return Err(RequireError::Account(address));
            },
            Some(Account::IncreaseBalance(address, _)) => {
                return Err(RequireError::Account(address));
            },
            None => {
                return Err(RequireError::Account(address));
            },
        };
        self.accounts.insert(address, account);
        Ok(())
    }
}
