mod stack;
mod pc;
mod memory;
mod params;
mod account;
mod cost;
mod run;

pub use self::opcode::Opcode;
pub use self::memory::{Memory, SeqMemory};
pub use self::stack::Stack;
pub use self::pc::PC;
pub use self::params::{Block, Transaction};
pub use self::account::{Commitment, Account, Storage, MapStorage, Log};

use self::cost::{gas_cost, gas_refund, CostAggregrator};
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
    Stopped
}

#[derive(Debug)]
pub enum CommitError {
    AlreadyCommitted,
    StateChanged
}

pub type ExecutionResult<T> = ::std::result::Result<T, ExecutionError>;

pub struct Machine<M, S> {
    pc: PC,
    memory: M,
    stack: Stack,
    transaction: Transaction,
    block: Block,
    cost_aggregrator: CostAggregrator,
    return_values: Vec<u8>,
    accounts: hash_map::HashMap<Address, Account>,
    valid_pc: bool,

    homestead: bool,
    eip150: bool,
    eip160: bool,
}

impl<M: Memory + Default, S: Storage> Machine<M, S> {
    pub fn new(transaction: Transaction, block: Block) -> Self {
        Self {
            pc: PC::default(),
            memory: M::default(),
            stack: Stack::default(),
            transaction: transaction,
            block: block,
            cost_aggregrator: CostAggregrator::default(),
            return_values: Vec::new(),
            accounts: hash_map::HashMap::new(),
            valid_pc: false,

            homestead: false,
            eip150: false,
            eip160: false,
        }
    }
}

impl<M: Memory, S: Storage> Machine<M, S> {
    pub fn pc(&self) -> ExecutionResult<&PC> {
        &self.pc
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

    pub fn block(&self) -> &Block {
        &self.block
    }

    pub fn accounts(&self) -> hash_map::Iter<Address, Account> {

    }

    pub fn return_values(&self) -> &[u8] {

    }

    pub fn active_memory_len(&self) -> M256 {
        self.cost_aggregrator.active_memoty_len()
    }

    pub fn owner(&self) -> Address {

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


    pub fn commit(&mut self, commitment: Commitment<S>) -> Result<(), CommitError> {
        let account: Account<S> = commitment.into();
        let address = account.address();
        if self.accounts.contains_key(&address) {
            return Err(CommitError::AlreadyCommitted);
        }

        if address == self.owner() {
            match account {
                Account::Full {
                    address: _,
                    code: ref code,
                    balance: _,
                    storage: _,
                    appending_logs: _,
                } => {
                    self.pc = PC::new(code.as_slice());
                    self.valid_pc = true;
                }
            }
        }

        self.accounts.insert(address, account);
        Ok(())
    }

    fn account_log(&mut self, address: Address, data: &[u8], topics: &[M256]) -> ExecutionResult<()> {
        match self.accounts.get(&address) {
            Some(&Account::Full {
                address: _,
                balance: _,
                storage: _,
                code: _,
                appending_logs: ref appending_logs,
            }) => {
                appending_logs.push(Log {
                    data: data.into(),
                    topics: topics.into(),
                });
                Ok(())
            },
            _ => {
                Err(ExecutionResult::RequireAccount(address))
            }
        }
    }

    fn account_code(&self, address: Address) -> ExecutionResult<&[u8]> {
        match self.accounts_get(&address) {
            Some(&Account::Full {
                address: _,
                balance: _,
                storage: _,
                code: ref code,
                appending_logs: _,
            }) => {
                Ok(code.as_ref().unwrap().as_slice())
            },
            Some(&Account::Code {
                code: ref code,
            }) => {
                Ok(code.as_ref().unwrap().as_slice())
            },
            _ => {
                Err(ExecutionResult::RequireAccountCode(address))
            }
        }
    }

    fn account_balance(&self, address: Address) -> ExecutionResult<M256> {
        match self.accounts_get(&address) {
            Some(&Account::Full {
                address: _,
                balance: balance,
                storage: _,
                code: _,
                appending_logs: _,
            }) => {
                Ok(balance)
            },
            _ => {
                Err(ExecutionResult::RequireAccount(address))
            }
        }
    }


    pub fn peek_cost(&self) -> ExecutionResult<Gas> {
        if !self.valid_pc {
            return Err(ExecutionError::RequireAccount(self.owner()));
        }

        let opcode = self.pc.peek_opcode()?;
        let aggregrator = self.cost_aggregrator;
        let (gas, agg) = gas_cost(opcode, &self, aggregrator);
        Ok(gas)
    }

    pub fn step(&mut self) -> Result<()> {
        if !self.valid_pc {
            return Err(ExecutionError::RequireAccount(self.owner()));
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
        let cost_aggregrator = self.state.cost_aggregrator();
        let (gas, agg) = trr!(gas_cost(opcode, &mut self.state,
                                       available_gas, cost_aggregrator), __);
        let refunded = trr!(gas_refund(opcode, &mut self.state), __);

        if gas > self.available_gas() {
            trr!(Err(ExecutionError::EmptyGas), __);
        }

        trr!(opcode.run(&mut self.state, available_gas - gas), __);

        self.cost_aggregrator = agg;
        self.used_gas = self.used_gas + gas;
        self.refunded_gas = self.refunded_gas + refunded;

        end_rescuable!(__);
        Ok(())
    }

    pub fn fire(&mut self) -> Result<()> {
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
