mod stack;
mod opcode;
mod pc;
mod memory;
mod machine;

pub use self::opcode::Opcode;
pub use self::memory::{Memory, VectorMemory};
pub use self::stack::{Stack, VectorStack};
pub use self::pc::{PC, VectorPC};
pub use self::machine::{Machine, VectorMachine, FakeVectorMachine};

#[derive(Debug)]
pub enum Error {
    EmptyGas,
    StackUnderflow,
    InvalidOpcode,
    Stopped
}

pub type Result<T> = ::std::result::Result<T, Error>;
