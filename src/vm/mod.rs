macro_rules! try_restore {
    ($e:expr, $r:expr) => (match $e {
        Ok(val) => val,
        Err(err) => {
            $r;
            return Err(err);
        }
    });
}

mod stack;
mod opcode;
mod pc;
mod memory;
mod machine;
mod cost;

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
    PCOverflow,
    PCTooLarge, // The current implementation only support code size with usize::maximum.
    DataTooLarge,
    CodeTooLarge,
    Stopped
}

pub type Result<T> = ::std::result::Result<T, Error>;
