mod memory;
mod stack;
mod pc;
mod storage;
mod params;
mod eval;
mod commit;
pub mod errors;

pub use self::memory::{Memory, SeqMemory};
pub use self::stack::Stack;
pub use self::pc::{PC, Instruction};
pub use self::storage::{Storage, HashMapStorage};
pub use self::params::{Context, BlockHeader, Log, Patch};
pub use self::eval::{State, Machine, Status};
pub use self::commit::{AccountCommitment, Account};

use self::errors::RequireError;

pub struct VM<M, S>(Vec<Machine<M, S>>);

impl<M: Memory + Default, S: Storage + Default + Clone> VM<M, S> {
    pub fn step(&mut self) -> Result<(), RequireError> {
        unimplemented!()
    }
}
