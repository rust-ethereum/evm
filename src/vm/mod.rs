mod stack;
mod opcode;
mod pc;
mod gas;
mod memory;
mod machine;

pub use self::opcode::Opcode;
pub use self::memory::{Memory, VectorMemory};
pub use self::stack::{Stack, VectorStack};
pub use self::pc::PC;
pub use self::machine::{Machine, VectorMachine};
pub use self::gas::Gas;

#[derive(Debug)]
pub enum Error {
    EmptyGas,
}

pub type Result<T> = ::std::result::Result<T, Error>;
