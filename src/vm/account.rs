use std::collections::hash_map;
use utils::gas::Gas;
use utils::address::Address;
use utils::bigint::{M256, U256};

use super::{ExecutionResult, ExecutionError, CommitResult, CommitError, Storage};

#[derive(Debug, Clone)]
pub enum AccountCommitment<S> {
    Full {
        nonce: M256,
        address: Address,
        balance: U256,
        storage: S,
        code: Vec<u8>,
    },
    Code {
        address: Address,
        code: Vec<u8>,
    },
}

impl<S: Storage> AccountCommitment<S> {
    pub fn address(&self) -> Address {
        match self {
            &AccountCommitment::Full {
                address: address,
                ..
            } => address,
            &AccountCommitment::Code {
                address: address,
                ..
            } => address,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Account<S> {
    Full {
        nonce: M256,
        address: Address,
        balance: U256,
        storage: S,
        code: Vec<u8>,
    },
    IncreaseBalance(Address, U256),
    DecreaseBalance(Address, U256),
}

impl<S: Storage> Account<S> {
    pub fn address(&self) -> Address {
        match self {
            &Account::Full {
                address: address,
                ..
            } => address,
            &Account::IncreaseBalance(address, _) => address,
            &Account::DecreaseBalance(address, _) => address,
        }
    }
}

pub struct AccountState<S> {
    accounts: hash_map::HashMap<Address, Account<S>>,
    codes: hash_map::HashMap<Address, Vec<u8>>,
}

impl<S: Storage> Default for AccountState<S> {
    fn default() -> Self {
        Self {
            accounts: hash_map::HashMap::new(),
            codes: hash_map::HashMap::new(),
        }
    }
}

impl<S: Storage + Default> AccountState<S> {
    pub fn accounts(&self) -> hash_map::Values<Address, Account<S>> {
        self.accounts.values()
    }

    pub fn commit(&mut self, commitment: AccountCommitment<S>) -> CommitResult<()> {
        match commitment {
            AccountCommitment::Full {
                nonce: nonce,
                address: address,
                balance: balance,
                storage: storage,
                code: code
            } => {
                if self.accounts.contains_key(&address) {
                    return Err(CommitError::AlreadyCommitted);
                }

                self.accounts.insert(address, Account::Full {
                    nonce: nonce,
                    address: address,
                    balance: balance,
                    storage: storage,
                    code: code,
                });
            },
            AccountCommitment::Code {
                address: address,
                code: code,
            } => {
                if self.accounts.contains_key(&address) || self.codes.contains_key(&address) {
                    return Err(CommitError::AlreadyCommitted);
                }

                self.codes.insert(address, code);
            }
        }
        Ok(())
    }

    pub fn code(&self, address: Address) -> ExecutionResult<&[u8]> {
        if self.codes.contains_key(&address) {
            return Ok(self.codes.get(&address).unwrap().as_slice());
        }

        if self.accounts.contains_key(&address) {
            match self.accounts.get(&address).unwrap() {
                &Account::Full {
                    code: ref code,
                    ..
                } => return Ok(code.as_slice()),
                _ => (),
            }
        }

        return Err(ExecutionError::RequireAccountCode(address));
    }

    pub fn nonce(&self, address: Address) -> ExecutionResult<M256> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get(&address).unwrap() {
                &Account::Full {
                    nonce: nonce,
                    ..
                } => return Ok(nonce),
                _ => (),
            }
        }

        return Err(ExecutionError::RequireAccount(address));
    }

    pub fn balance(&self, address: Address) -> ExecutionResult<U256> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get(&address).unwrap() {
                &Account::Full {
                    balance: balance,
                    ..
                } => return Ok(balance),
                _ => (),
            }
        }

        return Err(ExecutionError::RequireAccount(address));
    }

    pub fn storage(&self, address: Address) -> ExecutionResult<&S> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get(&address).unwrap() {
                &Account::Full {
                    storage: ref storage,
                    ..
                } => return Ok(storage),
                _ => (),
            }
        }

        return Err(ExecutionError::RequireAccount(address));
    }

    pub fn storage_mut(&mut self, address: Address) -> ExecutionResult<&mut S> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get_mut(&address).unwrap() {
                &mut Account::Full {
                    storage: ref mut storage,
                    ..
                } => return Ok(storage),
                _ => (),
            }
        }

        return Err(ExecutionError::RequireAccount(address));
    }

    pub fn increase_balance(&mut self, address: Address, topup: U256) -> ExecutionResult<()> {
        let account = match self.accounts.remove(&address) {
            Some(Account::Full {
                address: address,
                balance: balance,
                storage: storage,
                code: code,
                nonce: nonce,
            }) => {
                Some(Account::Full {
                    address: address,
                    balance: balance + topup,
                    storage: storage,
                    code: code,
                    nonce: nonce,
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
                return Err(ExecutionError::RequireAccount(address));
            },
        };
        if account.is_some() {
            self.accounts.insert(address, account.unwrap());
        }
        Ok(())
    }

    pub fn decrease_balance(&mut self, address: Address, withdraw: U256) -> ExecutionResult<()> {
        let account = match self.accounts.remove(&address) {
            Some(Account::Full {
                address: address,
                balance: balance,
                storage: storage,
                code: code,
                nonce: nonce,
            }) => {
                if balance < withdraw {
                    return Err(ExecutionError::EmptyBalance);
                }
                Some(Account::Full {
                    address: address,
                    balance: balance - withdraw,
                    storage: storage,
                    code: code,
                    nonce: nonce,
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
                return Err(ExecutionError::RequireAccount(address));
            },
        };
        if account.is_some() {
            self.accounts.insert(address, account.unwrap());
        }
        Ok(())
    }

    pub fn set_nonce(&mut self, address: Address, new_nonce: M256) -> ExecutionResult<()> {
        match self.accounts.get_mut(&address) {
            Some(&mut Account::Full {
                nonce: ref mut nonce,
                ..
            }) => {
                *nonce = new_nonce;
                Ok(())
            },
            _ => {
                Err(ExecutionError::RequireAccount(address))
            },
        }
    }

    pub fn remove(&mut self, address: Address) -> ExecutionResult<()> {
        let account = match self.accounts.remove(&address) {
            Some(Account::Full {
                address: address,
                balance: balance,
                storage: storage,
                code: code,
                nonce: nonce,
            }) => {
                Account::Full {
                    address: address,
                    balance: U256::zero(),
                    storage: S::default(),
                    code: Vec::new(),
                    nonce: M256::zero(),
                }
            },
            Some(Account::DecreaseBalance(address, balance)) => {
                return Err(ExecutionError::RequireAccount(address));
            },
            Some(Account::IncreaseBalance(address, balance)) => {
                return Err(ExecutionError::RequireAccount(address));
            },
            None => {
                return Err(ExecutionError::RequireAccount(address));
            },
        };
        self.accounts.insert(address, account);
        Ok(())
    }
}
