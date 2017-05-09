use utils::bigint::{M256, U256};
use utils::address::Address;
use utils::gas::Gas;
use utils::rlp::WriteRLP;

use std::collections::hash_map;
use crypto::sha3::Sha3;
use crypto::digest::Digest;
use super::{VM, Machine, Patch, Log, Account, CommitResult, CommitError,
            ExecutionResult, ExecutionError, AccountCommitment, Storage, Memory,
            BlockHeader, Context};

#[derive(Debug, Clone)]
pub enum Transaction {
    MessageCall(MessageCall),
    ContractCreation(ContractCreation),
}

#[derive(Debug, Clone)]
pub struct ContractCreation {
    pub gas_price: Gas,
    pub gas_limit: Gas,
    pub origin: Address,
    pub caller: Address,
    pub value: U256,
    pub init: Vec<u8>,
}

impl ContractCreation {
    pub fn into_context(self, nonce: M256, depth: usize) -> Context {
        let mut sha3 = Sha3::keccak256();
        let mut rlp: Vec<u8> = Vec::new();
        let mut ret = [0u8; 32];
        self.caller.write_rlp(&mut rlp);
        nonce.write_rlp(&mut rlp);
        sha3.input(rlp.as_slice());
        sha3.result(&mut ret);
        let address = Address::from(M256::from(ret));

        Context {
            address: address,
            caller: self.caller,
            code: self.init,
            data: Vec::new(),
            gas_limit: self.gas_limit,
            gas_price: self.gas_price,
            origin: self.origin,
            value: self.value,
            depth: depth,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageCall {
    pub gas_price: Gas,
    pub gas_limit: Gas,
    pub to: Address,
    pub origin: Address,
    pub caller: Address,
    pub value: U256,
    pub data: Vec<u8>,
}

impl MessageCall {
    pub fn into_context(self, code: Vec<u8>, depth: usize) -> Context {
        Context {
            address: self.to,
            caller: self.caller,
            code: code,
            data: self.data,
            gas_limit: self.gas_limit,
            gas_price: self.gas_price,
            origin: self.origin,
            value: self.value,
            depth: depth,
        }
    }
}

pub struct ContractCreationMachine<M, S> {
    machine: Option<Machine<M, S>>,
    transaction: ContractCreation,
    block: BlockHeader,
    depth: usize,

    _empty_logs: Vec<Log>,
    _empty_accounts: hash_map::HashMap<Address, Account<S>>,
    _initial_transactions: [Transaction; 1],
}

impl<M, S> Into<Machine<M, S>> for ContractCreationMachine<M, S> {
    fn into(self) -> Machine<M, S> {
        self.machine.unwrap()
    }
}

impl<M: Memory + Default, S: Storage + Default> ContractCreationMachine<M, S> {
    pub fn new(transaction: ContractCreation, block: BlockHeader, depth: usize) -> Self {
        Self {
            machine: None,
            transaction: transaction.clone(),
            block: block,
            depth: depth,

            _empty_logs: Vec::new(),
            _empty_accounts: hash_map::HashMap::new(),
            _initial_transactions: [Transaction::ContractCreation(transaction)],
        }
    }
}

impl<M: Memory + Default, S: Storage + Default> VM<S> for ContractCreationMachine<M, S> {
    fn peek_cost(&self) -> ExecutionResult<Gas> {
        if self.machine.is_none() {
            return Err(ExecutionError::RequireAccount(self.transaction.caller));
        }
        self.machine.as_ref().unwrap().peek_cost()
    }

    fn step(&mut self) -> ExecutionResult<()> {
        if self.machine.is_none() {
            return Err(ExecutionError::RequireAccount(self.transaction.caller));
        }
        self.machine.as_mut().unwrap().step()
    }

    fn commit_account(&mut self, commitment: AccountCommitment<S>) -> CommitResult<()> {
        if self.machine.is_none() {
            match commitment {
                AccountCommitment::Full {
                    nonce: nonce,
                    address: address,
                    balance: balance,
                    storage: storage,
                    code: code,
                } => {
                    if address != self.transaction.caller {
                        return Err(CommitError::Invalid);
                    }

                    let context = self.transaction.clone().into_context(nonce, self.depth);
                    let mut machine = Machine::new(context, self.block.clone());
                    machine.commit_account(AccountCommitment::Full {
                        nonce: nonce,
                        address: address,
                        balance: balance,
                        storage: storage,
                        code: code
                    });
                    machine.transactions.push(Transaction::ContractCreation(self.transaction.clone()));
                    self.machine = Some(machine);
                },
                _ => return Err(CommitError::Invalid),
            }
            return Ok(());
        }
        self.machine.as_mut().unwrap().commit_account(commitment)
    }

    fn commit_blockhash(&mut self, number: M256, hash: M256) -> CommitResult<()> {
        if self.machine.is_none() {
            return Err(CommitError::Invalid);
        }
        self.machine.as_mut().unwrap().commit_blockhash(number, hash)
    }

    fn accounts(&self) -> hash_map::Values<Address, Account<S>> {
        if self.machine.is_none() {
            return self._empty_accounts.values();
        }
        self.machine.as_ref().unwrap().accounts()
    }

    fn transactions(&self) -> &[Transaction] {
        if self.machine.is_none() {
            return &self._initial_transactions;
        }
        self.machine.as_ref().unwrap().transactions()
    }

    fn appending_logs(&self) -> &[Log] {
        if self.machine.is_none() {
            return self._empty_logs.as_slice();
        }
        self.machine.as_ref().unwrap().appending_logs()
    }

    fn patch(&self) -> Patch {
        self.machine.as_ref().unwrap().patch()
    }
}

pub struct MessageCallMachine<M, S> {
    machine: Option<Machine<M, S>>,
    transaction: MessageCall,
    block: BlockHeader,
    depth: usize,

    _empty_logs: Vec<Log>,
    _empty_accounts: hash_map::HashMap<Address, Account<S>>,
    _initial_transactions: [Transaction; 1],
}

impl<M, S> Into<Machine<M, S>> for MessageCallMachine<M, S> {
    fn into(self) -> Machine<M, S> {
        self.machine.unwrap()
    }
}

impl<M: Memory + Default, S: Storage + Default> MessageCallMachine<M, S> {
    pub fn new(transaction: MessageCall, block: BlockHeader, depth: usize) -> Self {
        Self {
            machine: None,
            transaction: transaction.clone(),
            block: block,
            depth: depth,

            _empty_logs: Vec::new(),
            _empty_accounts: hash_map::HashMap::new(),
            _initial_transactions: [Transaction::MessageCall(transaction)],
        }
    }
}

impl<M: Memory + Default, S: Storage + Default> VM<S> for MessageCallMachine<M, S> {
    fn peek_cost(&self) -> ExecutionResult<Gas> {
        if self.machine.is_none() {
            return Err(ExecutionError::RequireAccount(self.transaction.caller));
        }
        self.machine.as_ref().unwrap().peek_cost()
    }

    fn step(&mut self) -> ExecutionResult<()> {
        if self.machine.is_none() {
            return Err(ExecutionError::RequireAccount(self.transaction.caller));
        }
        self.machine.as_mut().unwrap().step()
    }

    fn commit_account(&mut self, commitment: AccountCommitment<S>) -> CommitResult<()> {
        if self.machine.is_none() {
            match commitment {
                AccountCommitment::Full {
                    nonce: nonce,
                    address: address,
                    balance: balance,
                    storage: storage,
                    code: code,
                } => {
                    if address != self.transaction.caller {
                        return Err(CommitError::Invalid);
                    }

                    let context = self.transaction.clone().into_context(code.clone(), self.depth);
                    let mut machine = Machine::new(context, self.block.clone());
                    machine.commit_account(AccountCommitment::Full {
                        nonce: nonce,
                        address: address,
                        balance: balance,
                        storage: storage,
                        code: code
                    });
                    machine.transactions.push(Transaction::MessageCall(self.transaction.clone()));
                    self.machine = Some(machine);
                },
                _ => return Err(CommitError::Invalid),
            }
            return Ok(());
        }
        self.machine.as_mut().unwrap().commit_account(commitment)
    }

    fn commit_blockhash(&mut self, number: M256, hash: M256) -> CommitResult<()> {
        if self.machine.is_none() {
            return Err(CommitError::Invalid);
        }
        self.machine.as_mut().unwrap().commit_blockhash(number, hash)
    }

    fn transactions(&self) -> &[Transaction] {
        if self.machine.is_none() {
            return &self._initial_transactions;
        }
        self.machine.as_ref().unwrap().transactions()
    }

    fn accounts(&self) -> hash_map::Values<Address, Account<S>> {
        if self.machine.is_none() {
            return self._empty_accounts.values();
        }
        self.machine.as_ref().unwrap().accounts()
    }

    fn appending_logs(&self) -> &[Log] {
        if self.machine.is_none() {
            return self._empty_logs.as_slice();
        }
        self.machine.as_ref().unwrap().appending_logs()
    }

    fn patch(&self) -> Patch {
        self.machine.as_ref().unwrap().patch()
    }
}
