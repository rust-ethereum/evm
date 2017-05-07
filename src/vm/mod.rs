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
    EmptyBalance,
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
pub type CommitResult<T> = ::std::result::Result<T, CommitError>;

pub type SeqMachine = Machine<SeqMemory, HashMapStorage>;

pub trait VM<S> {
    fn step(&mut self) -> ExecutionResult<()>,
    fn peek_cost(&self) -> ExecutionResult<Gas>,
    fn fire(&mut self) -> ExecutionResult<()>,

    fn commit_account(commitment: AccountCommitment<S>) -> CommitResult<()>;
    fn commit_blockhash(number: M256, hash: M256) -> CommitResult<()>;
}

pub struct Machine<M, S> {
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
    pub fn new(context: Context, block: BlockHeader) -> Self {
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
        if self.blockhashes.contains_key(&number) {
            return Err(CommitError::AlreadyCommitted);
        }

        self.blockhashes.insert(number, hash);
        Ok(())
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
