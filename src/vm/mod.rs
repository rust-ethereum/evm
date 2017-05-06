mod stack;
mod pc;
mod memory;
mod params;
mod account;
mod cost;
mod run;

pub use self::memory::{Memory, SeqMemory};
pub use self::stack::Stack;
pub use self::pc::PC;
pub use self::params::{BlockHeader, Transaction};
pub use self::account::{Account, Storage, HashMapStorage, Log};

use self::cost::{gas_cost, gas_refund, CostAggregrator};
use self::run::run_opcode;
use std::collections::hash_map;
use utils::gas::Gas;
use utils::address::Address;
use utils::bigint::{M256, U256};

#[derive(Debug)]
pub enum ExecutionError {
    EmptyGas,
    StackUnderflow,
    StackOverflow,
    InvalidOpcode,
    PCOverflow,
    PCBadJumpDest,
    PCTooLarge, // The current implementation only support code size with usize::maximum.
    MemoryTooLarge,
    DataTooLarge,
    CodeTooLarge,
    RequireAccount(Address),
    RequireAccountCode(Address),
    RequireBlockhash(M256),
    Stopped
}

#[derive(Debug)]
pub enum CommitError {
    AlreadyCommitted,
    StateChanged
}

pub type ExecutionResult<T> = ::std::result::Result<T, ExecutionError>;

pub type SeqMachine = Machine<SeqMemory, HashMapStorage>;

pub trait Machine {
    step,
    peek_cost,
    fire
}

pub struct TransactionMachine<M, S> {
    transaction: Transaction,
    machine: Option<ContextMachine<M, S>>
}

pub struct ContextMachine<M, S> {
    pc: PC,
    memory: M,
    stack: Stack,
    cost_aggregrator: CostAggregrator,
    return_values: Vec<u8>,

    context: ExecutionContext,
    block: BlockHeader,

    account_state: AccountState,
    blockhashes: hash_map::HashMap<M256, M256>,

    used_gas: Gas,
    refunded_gas: Gas,

    homestead: bool,
    eip150: bool,
    eip160: bool,
}

impl<M: Memory + Default, S: Storage> ContextMachine<M, S> {
    pub fn new(transaction: Transaction, block: BlockHeader) -> Self {
        Machine::with_depth(transaction, block, 1)
    }

    pub fn with_depth(transaction: Transaction, block: BlockHeader, depth: usize) -> Self {
        Self {
            pc: PC::default(),
            memory: M::default(),
            stack: Stack::default(),
            transaction: transaction,
            block: block,
            cost_aggregrator: CostAggregrator::default(),
            return_values: Vec::new(),
            accounts: hash_map::HashMap::new(),
            blockhashes: hash_map::HashMap::new(),
            valid_pc: false,
            used_gas: Gas::zero(),
            refunded_gas: Gas::zero(),
            depth: depth,

            homestead: false,
            eip150: false,
            eip160: false,
        }
    }
}

impl<M: Memory + Default, S: Storage + Default> Machine<M, S> {
    pub fn pc(&self) -> Option<&PC> {
        if self.valid_pc {
            Some(&self.pc)
        } else {
            None
        }
    }

    pub fn memory(&self) -> &M {
        &self.memory
    }

    pub fn stack(&self) -> &Stack {
        &self.stack
    }

    pub fn transaction(&self) -> &Transaction {
        &self.transaction
    }

    pub fn block(&self) -> &BlockHeader {
        &self.block
    }

    pub fn accounts(&self) -> hash_map::Values<Address, Account<S>> {
        self.accounts.values()
    }

    pub fn return_values(&self) -> &[u8] {
        self.return_values.as_slice()
    }

    pub fn active_memory_len(&self) -> M256 {
        self.cost_aggregrator.active_memory_len()
    }

    pub fn owner(&self) -> ExecutionResult<Address> {
        Ok(match self.transaction {
            Transaction::MessageCall {
                to: to,
                ..
            } => to,
            Transaction::ContractCreation {
                ..
            } => unimplemented!(),
        })
    }

    pub fn available_gas(&self) -> Gas {
        self.transaction.gas_limit() - self.used_gas
    }


    pub fn homestead(&self) -> bool {
        self.homestead
    }

    pub fn set_homestead(&mut self, val: bool) {
        self.homestead = val;
    }

    pub fn eip150(&self) -> bool {
        self.eip150
    }

    pub fn set_eip150(&mut self, val: bool) {
        self.eip150 = val;
    }

    pub fn eip160(&self) -> bool {
        self.eip160
    }

    pub fn set_eip160(&mut self, val: bool) {
        self.eip160 = val;
    }


    pub fn commit_blockhash(&mut self, number: M256, hash: M256) -> Result<(), CommitError> {

    }

    pub fn commit_account(&mut self, commitment: AccountCommitment<S>) -> Result<(), CommitError> {
        match commitment {
            Commitment::Full {
                address: address,
                balance: balance,
                storage: storage,
                code: code,
                nonce: nonce,
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
                    appending_logs: Vec::new(),
                });
            },
            Commitment::Code {
                address: address,
                code: code,
            } => {
                if self.accounts.contains_key(&address) {
                    return Err(CommitError::AlreadyCommitted);
                }

                self.accounts.insert(address, Account::Code {
                    address: address,
                    code: code
                });
            },
            Commitment::Blockhash {
                number: number,
                hash: hash,
            } => {
                if self.blockhashes.contains_key(&number) {
                    return Err(CommitError::AlreadyCommitted);
                }

                self.blockhashes.insert(number, hash);
            },
        }

        let owner = self.owner();
        if !self.valid_pc && owner.is_ok() {
            let owner = owner.ok().unwrap();
            match self.accounts.get(&owner) {
                Some(&Account::Full {
                    code: ref code,
                    ..
                }) => {
                    self.pc = PC::new(code.as_slice());
                    self.valid_pc = true;
                },
                Some(&Account::Code {
                    code: ref code,
                    ..
                }) => {
                    self.pc = PC::new(code.as_slice());
                    self.valid_pc = true;
                },
                _ => (),
            }
        }
        Ok(())
    }

    fn account_log(&mut self, address: Address, data: &[u8], topics: &[M256]) -> ExecutionResult<()> {
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

    fn account_code(&self, address: Address) -> ExecutionResult<&[u8]> {
        match self.accounts.get(&address) {
            Some(&Account::Full {
                code: ref code,
                ..
            }) => {
                Ok(code.as_slice())
            },
            Some(&Account::Code {
                code: ref code,
                ..
            }) => {
                Ok(code.as_slice())
            },
            _ => {
                Err(ExecutionError::RequireAccountCode(address))
            }
        }
    }

    fn account_nonce(&self, address: Address) -> ExecutionResult<M256> {
        match self.accounts.get(&address) {
            Some(&Account::Full {
                nonce: nonce,
                ..
            }) => {
                Ok(nonce)
            },
            _ => {
                Err(ExecutionError::RequireAccount(address))
            }
        }
    }

    fn account_nonce_inc(&mut self, address: Address) -> ExecutionResult<()> {
        let account = match self.accounts.remove(&address) {
            Some(Account::Full {
                address: address,
                balance: balance,
                storage: storage,
                code: code,
                appending_logs: appending_logs,
                nonce: nonce,
            }) => {
                Account::Full {
                    address: address,
                    balance: balance,
                    storage: storage,
                    code: code,
                    appending_logs: appending_logs,
                    nonce: nonce + M256::from(1u64),
                }
            },
            Some(Account::Remove(address)) => panic!(),
            _ => {
                return Err(ExecutionError::RequireAccount(address));
            }
        };
        self.accounts.insert(address, account);
        Ok(())
    }

    fn account_nonce_dec(&mut self, address: Address) -> ExecutionResult<()> {
        let account = match self.accounts.remove(&address) {
            Some(Account::Full {
                address: address,
                balance: balance,
                storage: storage,
                code: code,
                appending_logs: appending_logs,
                nonce: nonce,
            }) => {
                Account::Full {
                    address: address,
                    balance: balance,
                    storage: storage,
                    code: code,
                    appending_logs: appending_logs,
                    nonce: nonce - M256::from(1u64),
                }
            },
            Some(Account::Remove(address)) => panic!(),
            _ => {
                return Err(ExecutionError::RequireAccount(address));
            }
        };
        self.accounts.insert(address, account);
        Ok(())
    }

    fn account_balance(&self, address: Address) -> ExecutionResult<U256> {
        match self.accounts.get(&address) {
            Some(&Account::Full {
                balance: balance,
                ..
            }) => {
                Ok(balance)
            },
            _ => {
                Err(ExecutionError::RequireAccount(address))
            }
        }
    }

    fn account_balance_topup(&mut self, address: Address, topup: U256) -> ExecutionResult<()> {
        let account = match self.accounts.remove(&address) {
            Some(Account::Full {
                address: address,
                balance: balance,
                storage: storage,
                code: code,
                appending_logs: appending_logs,
                nonce: nonce,
            }) => {
                Account::Full {
                    address: address,
                    balance: balance + topup,
                    storage: storage,
                    code: code,
                    appending_logs: appending_logs,
                    nonce: nonce,
                }
            },
            Some(Account::Code {
                ..
            }) => {
                return Err(ExecutionError::RequireAccount(address));
            }
            Some(Account::Remove(address)) => {
                Account::Full {
                    nonce: M256::zero(),
                    address: address,
                    balance: topup,
                    storage: S::default(),
                    code: Vec::new(),
                    appending_logs: Vec::new(),
                }
            },
            Some(Account::Topup(address, old_topup)) => {
                Account::Topup(address, old_topup + topup)
            },
            None => {
                Account::Topup(address, topup)
            },
        };
        self.accounts.insert(address, account);
        Ok(())
    }

    fn account_remove(&mut self, address: Address) {
        self.accounts.insert(address, Account::Remove(address));
    }

    fn account_storage(&self, address: Address) -> ExecutionResult<&S> {
        match self.accounts.get(&address) {
            Some(&Account::Full {
                storage: ref storage,
                ..
            }) => {
                Ok(storage)
            },
            _ => {
                Err(ExecutionError::RequireAccount(address))
            }
        }
    }

    fn account_storage_mut(&mut self, address: Address) -> ExecutionResult<&mut S> {
        match self.accounts.get_mut(&address) {
            Some(&mut Account::Full {
                storage: ref mut storage,
                ..
            }) => {
                Ok(storage)
            },
            _ => {
                Err(ExecutionError::RequireAccount(address))
            }
        }
    }

    fn blockhash(&mut self, number: M256) -> ExecutionResult<M256> {
        match self.blockhashes.get(&number) {
            Some(val) => Ok(*val),
            None => Err(ExecutionError::RequireBlockhash(number)),
        }
    }


    pub fn peek_cost(&self) -> ExecutionResult<Gas> {
        if !self.valid_pc {
            return Err(ExecutionError::RequireAccount(self.owner()?));
        }

        let opcode = self.pc.peek_opcode()?;
        let aggregrator = self.cost_aggregrator;
        let (gas, agg) = gas_cost(opcode, &self, aggregrator)?;
        Ok(gas)
    }

    pub fn step(&mut self) -> ExecutionResult<()> {
        if !self.valid_pc {
            return Err(ExecutionError::RequireAccount(self.owner()?));
        }

        begin_rescuable!(self, &mut Self, __);
        if self.pc.stopped() {
            trr!(Err(ExecutionError::Stopped), __);
        }

        let position = self.pc.position();
        on_rescue!(|machine| {
            machine.pc.jump_unchecked(position);
        }, __);

        let opcode = trr!(self.pc.read_opcode(), __);
        let available_gas = self.available_gas();
        let cost_aggregrator = self.cost_aggregrator;
        let (gas, agg) = trr!(gas_cost(opcode, self, cost_aggregrator), __);
        let refunded = trr!(gas_refund(opcode, self), __);

        if gas > self.available_gas() {
            trr!(Err(ExecutionError::EmptyGas), __);
        }

        trr!(run_opcode(opcode, self, available_gas - gas), __);

        self.cost_aggregrator = agg;
        self.used_gas = self.used_gas + gas;
        self.refunded_gas = self.refunded_gas + refunded;

        end_rescuable!(__);
        Ok(())
    }

    pub fn fire(&mut self) -> ExecutionResult<()> {
        loop {
            let result = self.step();

            if result.is_err() {
                match result.err().unwrap() {
                    ExecutionError::Stopped => return Ok(()),
                    err => return Err(err),
                }
            }
        }
    }
}
