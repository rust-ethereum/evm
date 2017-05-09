mod stack;
mod pc;
mod memory;
mod params;
mod account;
mod storage;
mod cost;
mod run;
mod transaction;

pub use self::memory::{Memory, SeqMemory};
pub use self::stack::Stack;
pub use self::pc::PC;
pub use self::params::{BlockHeader, Context, Patch, Log};
pub use self::account::{Account, AccountCommitment};
pub use self::storage::{Storage, HashMapStorage};
pub use self::transaction::{Transaction, MessageCall, ContractCreation,
                            MessageCallMachine, ContractCreationMachine};

use self::account::AccountState;
use self::cost::{gas_cost, gas_refund, gas_stipend, CostAggregrator};
use self::run::run_opcode;
use std::collections::hash_map;
use utils::gas::Gas;
use utils::address::Address;
use utils::bigint::{M256, U256};

#[derive(Debug, Clone)]
pub enum ExecutionError {
    EmptyGas,
    EmptyBalance,
    StackUnderflow,
    StackOverflow,
    CallOverflow,
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

impl ExecutionError {
    pub fn is_require(&self) -> bool {
        match self {
            &ExecutionError::RequireAccount(_) |
            &ExecutionError::RequireAccountCode(_) |
            &ExecutionError::RequireBlockhash(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum CommitError {
    AlreadyCommitted,
    StateChanged,
    Invalid,
}

pub type ExecutionResult<T> = ::std::result::Result<T, ExecutionError>;
pub type CommitResult<T> = ::std::result::Result<T, CommitError>;

pub type SeqMachine = Machine<SeqMemory, HashMapStorage>;

pub trait VM<S: Storage> {
    fn step(&mut self) -> ExecutionResult<()>;
    fn peek_cost(&self) -> ExecutionResult<Gas>;
    fn fire(&mut self) -> ExecutionResult<()> {
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

    fn commit_account(&mut self, commitment: AccountCommitment<S>) -> CommitResult<()>;
    fn commit_blockhash(&mut self, number: M256, hash: M256) -> CommitResult<()>;
    fn accounts(&self) -> hash_map::Values<Address, Account<S>>;
    fn transactions(&self) -> &[Transaction];
    fn appending_logs(&self) -> &[Log];
    fn available_gas(&self) -> Gas;

    fn patch(&self) -> Patch;
}

pub struct Machine<M, S> {
    pc: PC,
    memory: M,
    stack: Stack,
    cost_aggregrator: CostAggregrator,
    return_values: Vec<u8>,

    context: Context,
    block: BlockHeader,

    account_state: AccountState<S>,
    blockhashes: hash_map::HashMap<M256, M256>,
    appending_logs: Vec<Log>,
    transactions: Vec<Transaction>,

    used_gas: Gas,
    refunded_gas: Gas,

    patch: Patch,
}

impl<M: Memory + Default, S: Storage + Default> Machine<M, S> {
    pub fn new(context: Context, block: BlockHeader) -> Self {
        Self {
            pc: PC::new(context.code.as_slice()),
            memory: M::default(),
            stack: Stack::default(),
            cost_aggregrator: CostAggregrator::default(),
            return_values: Vec::new(),

            context: context,
            block: block,

            account_state: AccountState::default(),
            blockhashes: hash_map::HashMap::new(),
            appending_logs: Vec::new(),
            transactions: Vec::new(),

            used_gas: Gas::zero(),
            refunded_gas: Gas::zero(),

            patch: Patch::None,
        }
    }
}

impl<M: Memory + Default, S: Storage + Default> VM<S> for Machine<M, S> {
    fn peek_cost(&self) -> ExecutionResult<Gas> {
        if self.context.depth >= 1024 {
            return Err(ExecutionError::CallOverflow);
        }

        let opcode = self.pc.peek_opcode()?;
        let aggregrator = self.cost_aggregrator;
        let (gas, agg) = gas_cost(opcode, &self, aggregrator)?;
        Ok(gas)
    }

    fn step(&mut self) -> ExecutionResult<()> {
        if self.context.depth >= 1024 {
            return Err(ExecutionError::CallOverflow);
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
        let stipend = trr!(gas_stipend(opcode, self), __);

        if gas > self.available_gas() {
            trr!(Err(ExecutionError::EmptyGas), __);
        }

        trr!(run_opcode(opcode, self, stipend, available_gas - gas + stipend), __);

        self.cost_aggregrator = agg;
        self.used_gas = self.used_gas + gas - stipend;
        self.refunded_gas = self.refunded_gas + refunded;

        end_rescuable!(__);
        Ok(())
    }

    fn commit_account(&mut self, commitment: AccountCommitment<S>) -> CommitResult<()> {
        self.account_state.commit(commitment)
    }

    fn commit_blockhash(&mut self, number: M256, hash: M256) -> CommitResult<()> {
        if self.blockhashes.contains_key(&number) {
            return Err(CommitError::AlreadyCommitted);
        }

        self.blockhashes.insert(number, hash);
        Ok(())
    }

    fn accounts(&self) -> hash_map::Values<Address, Account<S>> {
        self.account_state.accounts()
    }

    fn transactions(&self) -> &[Transaction] {
        self.transactions.as_slice()
    }

    fn appending_logs(&self) -> &[Log] {
        self.appending_logs.as_slice()
    }

    fn available_gas(&self) -> Gas {
        self.context.gas_limit - self.used_gas
    }

    fn patch(&self) -> Patch {
        self.patch
    }
}

impl<M: Memory + Default, S: Storage + Default> Machine<M, S> {
    pub fn pc(&self) -> &PC {
        &self.pc
    }

    pub fn memory(&self) -> &M {
        &self.memory
    }

    pub fn stack(&self) -> &Stack {
        &self.stack
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn block(&self) -> &BlockHeader {
        &self.block
    }

    pub fn accounts(&self) -> hash_map::Values<Address, Account<S>> {
        self.account_state.accounts()
    }

    pub fn return_values(&self) -> &[u8] {
        self.return_values.as_slice()
    }

    pub fn active_memory_len(&self) -> M256 {
        self.cost_aggregrator.active_memory_len()
    }


    fn blockhash(&mut self, number: M256) -> ExecutionResult<M256> {
        match self.blockhashes.get(&number) {
            Some(val) => Ok(*val),
            None => Err(ExecutionError::RequireBlockhash(number)),
        }
    }

    fn append_log(&mut self, log: Log) {
        self.appending_logs.push(log);
    }

    fn fire_sub<V: VM<S>>(&self, submachine: &mut V) -> ExecutionResult<()> {
        loop {
            let result = submachine.fire();
            match result {
                Err(ExecutionError::RequireAccount(address)) => {
                    submachine.commit_account(AccountCommitment::Full {
                        nonce: self.account_state.nonce(address)?,
                        balance: self.account_state.balance(address)?,
                        storage: self.account_state.storage(address)?.derive(),
                        code: self.account_state.code(address)?.into(),
                        address: address,
                    });
                },
                Err(ExecutionError::RequireAccountCode(address)) => {
                    submachine.commit_account(AccountCommitment::Code {
                        code: self.account_state.code(address)?.into(),
                        address: address,
                    });
                },
                val => return val,
            }
        }
    }

    fn merge_sub(&mut self, submachine: &Machine<M, S>) {
        for account in submachine.accounts() {
            self.account_state.merge(account);
        }
        for transaction in submachine.transactions() {
            self.transactions.push(transaction.clone());
        }
    }
}
