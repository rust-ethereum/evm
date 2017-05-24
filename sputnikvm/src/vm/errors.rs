//! VM errors

use utils::address::Address;
use utils::bigint::M256;

#[derive(Debug, Clone)]
/// Errors returned by an EVM memory.
pub enum MemoryError {
    /// The index is too large for the implementation of the VM to
    /// handle.
    IndexNotSupported,
}

impl From<MemoryError> for MachineError {
    fn from(val: MemoryError) -> MachineError {
        MachineError::Memory(val)
    }
}

impl From<MemoryError> for EvalError {
    fn from(val: MemoryError) -> EvalError {
        EvalError::Machine(MachineError::Memory(val))
    }
}

#[derive(Debug, Clone)]
/// Errors returned by an EVM stack.
pub enum StackError {
    /// Stack is overflowed (pushed more than 1024 items to the
    /// stack).
    Overflow,
    /// Stack is underflowed (poped an empty stack).
    Underflow,
}

impl From<StackError> for EvalError {
    fn from(val: StackError) -> EvalError {
        EvalError::Machine(MachineError::Stack(val))
    }
}

#[derive(Debug, Clone)]
/// Errors returned by an EVM PC.
pub enum PCError {
    /// The opcode is invalid and the PC is not able to convert it to
    /// an instruction.
    InvalidOpcode,
    /// The index is too large for the implementation of the VM to
    /// handle.
    IndexNotSupported,
    /// PC jumped to an invalid jump destination.
    BadJumpDest,
    /// PC overflowed (tries to read the next opcode which is already
    /// the end of the code).
    Overflow,
}

impl From<PCError> for EvalError {
    fn from(val: PCError) -> EvalError {
        EvalError::Machine(MachineError::PC(val))
    }
}

#[derive(Debug, Clone)]
/// Errors returned when trying to step the instruction.
pub enum EvalError {
    /// A runtime error. Non-recoverable.
    Machine(MachineError),
    /// The VM requires account of blockhash information to be
    /// committed. Recoverable after the required information is
    /// committed.
    Require(RequireError),
}

#[derive(Debug, Clone)]
/// Errors returned by the a single machine of the VM.
pub enum MachineError {
    /// VM memory error.
    Memory(MemoryError),
    /// VM stack error.
    Stack(StackError),
    /// VM PC error.
    PC(PCError),

    /// Call stack is too large that it exceeds the limit.
    CallstackOverflow,
    /// For instruction that requires reading a range, it is invalid.
    InvalidRange,
    /// Not enough gas to continue.
    EmptyGas,
}

impl From<MachineError> for EvalError {
    fn from(val: MachineError) -> EvalError {
        EvalError::Machine(val)
    }
}

#[derive(Debug, Clone)]
/// Errors returned by the VM.
pub enum VMError {
    /// VM runtime error.
    Machine(MachineError),
}

impl From<MachineError> for VMError {
    fn from(val: MachineError) -> VMError {
        VMError::Machine(val)
    }
}

#[derive(Debug, Clone)]
/// Errors stating that the VM requires additional information to
/// continue running.
pub enum RequireError {
    Account(Address),
    AccountCode(Address),
    AccountStorage(Address, M256),
    Blockhash(M256),
}

impl From<RequireError> for EvalError {
    fn from(val: RequireError) -> EvalError {
        EvalError::Require(val)
    }
}

#[derive(Debug, Clone)]
/// Errors returned when committing a new information.
pub enum CommitError {
    InvalidCommitment,
    AlreadyCommitted,
}
