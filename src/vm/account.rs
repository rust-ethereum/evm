use std::collections::hash_map;
use utils::gas::Gas;
use utils::address::Address;
use utils::bigint::{M256, U256};

use super::{ExecutionResult, ExecutionError};

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
            &Account::Full {
                address: address,
                ..
            } => address,
            &Account::Code {
                address: address,
                ..
            } => address,
        }
    }
}

pub enum Account<S> {
    Full {
        nonce: M256,
        address: Address,
        balance: U256,
        storage: S,
        code: Vec<u8>,
        appending_logs: Vec<u8>,
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
            &IncreaseBalance(address, _) => address,
            &DecreaseBalance(address, _) => address,
        }
    }
}

pub struct AccountState<S> {
    accounts: hash_map::HashMap<Address, Account<S>>,
    codes: hash_map::HashMap<Address, Vec<u8>>,
}

impl<S: Storage> AccountState<S> {
    pub fn commit(commitment: AccountCommitment<S>) -> CommitResult<()> {
        match commitment {
            Full {
                nonce: nonce,
                address: address,
                balance: balance,
                storage: storage,
                code: code
            } => {
                if self.accounts.contain_keys(&address) {
                    return Err(CommitError::AlreadyCommitted);
                }

                self.accounts.insert(address, Account::Full {
                    nonce: nonce,
                    address: address,
                    balance: balance,
                    storage: storage,
                    code: code,
                    appending_logs: Vec::new(),
                });
            },
            Code {
                address: address,
                code: code,
            } => {
                if self.accounts.contain_keys(&address) || self.codes.contain_keys(&address) {
                    return Err(CommitError::AlreadyCommitted);
                }

                self.codes.insert(address, code);
            }
        }
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
                    nonce: nonce
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
                    balance: balance
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
                    storage: ref storage
                    ..
                } => return Ok(storage),
                _ => (),
            }
        }

        return Err(ExecutionError::RequireAccount(address));
    }

    pub fn storage_mut(&mut self, address: Address) -> ExecutionResult<&mut S> {
        if self.accounts.contains_key(&address) {
            match self.accounts.get(&address).unwrap() {
                &Account::Full {
                    storage: ref mut storage
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
                appending_logs: appending_logs,
                nonce: nonce,
            }) => {
                Some(Account::Full {
                    address: address,
                    balance: balance + topup,
                    storage: storage,
                    code: code,
                    appending_logs: appending_logs,
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
                appending_logs: appending_logs,
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
                    appending_logs: appending_logs,
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

    pub fn set_nonce(&mut self, address: Address, new_nonce: nonce) -> ExecutionResult<()> {
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

    pub fn append_log(&mut self, address: Address, data: &[u8], topics: &[M256]) -> ExecutionResult<()> {
        match self.accounts.get_mut(&address) {
            Some(&mut Account::Full {
                appending_logs: ref mut appending_logs,
                ..
            }) => {
                appending_logs.push(Log {
                    data: data.into(),
                    topics: topics.into(),
                });
                Ok(())
            },
            _ => {
                Err(ExecutionError::RequireAccount(address))
            }
        }
    }
}
