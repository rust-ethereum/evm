use utils::bigint::M256;
use utils::gas::Gas;
use super::commit::{AccountState, BlockhashState};
use super::errors::{RequireError, MachineError};
use super::{Stack, Context, BlockHeader, Patch, PC, Storage, Memory};

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
    pub fn step(&mut self) -> Result<(), RequireError> {
        unimplemented!()
    }
}
