use utils::bigint::M256;
use utils::gas::Gas;
use super::commit::{AccountState, BlockhashState};
use super::errors::{RequireError, MachineError, CommitError};
use super::{Stack, Context, BlockHeader, Patch, PC, Storage, Memory, AccountCommitment};

pub mod cost;
pub mod run;

/// A VM state without PC.
pub struct State<M, S> {
    pub memory: M,
    pub stack: Stack,

    pub context: Context,
    pub block: BlockHeader,
    pub patch: Patch,

    pub memory_gas: Gas,
    pub used_gas: Gas,
    pub refuneded_gas: Gas,

    pub account_state: AccountState<S>,
    pub blockhash_state: BlockhashState,
}

/// A VM state with PC.
pub struct Machine<M, S> {
    state: State<M, S>,
    pc: PC,
    status: Status,
}

pub enum Status {
    Running,
    ExitedOk(Vec<u8>),
    ExitedError(MachineError),
    InvokeCall(Context, (M256, M256)),
}

impl<M: Memory + Default, S: Storage + Default + Clone> Machine<M, S> {
    pub fn commit_account(&mut self, commitment: AccountCommitment<S>) -> Result<(), CommitError> {
        self.state.account_state.commit(commitment)
    }

    pub fn commit_blockhash(&mut self, number: M256, hash: M256) -> Result<(), CommitError> {
        self.state.blockhash_state.commit(number, hash)
    }

    pub fn step(&mut self) -> Result<(), RequireError> {
        unimplemented!()
    }
}
