use utils::address::Address;
use utils::bigint::M256;

#[derive(Debug, Clone)]
pub enum MemoryError {
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
pub enum StackError {
    Overflow,
    Underflow,
}

impl From<StackError> for EvalError {
    fn from(val: StackError) -> EvalError {
        EvalError::Machine(MachineError::Stack(val))
    }
}

#[derive(Debug, Clone)]
pub enum PCError {
    InvalidOpcode,
    IndexNotSupported,
    BadJumpDest,
    Overflow,
}

impl From<PCError> for EvalError {
    fn from(val: PCError) -> EvalError {
        EvalError::Machine(MachineError::PC(val))
    }
}

#[derive(Debug, Clone)]
pub enum StorageError {
    IndexNotSupported,
}

impl From<StorageError> for EvalError {
    fn from(val: StorageError) -> EvalError {
        EvalError::Machine(MachineError::Storage(val))
    }
}

#[derive(Debug, Clone)]
pub enum EvalError {
    Machine(MachineError),
    Require(RequireError),
}

#[derive(Debug, Clone)]
pub enum MachineError {
    Memory(MemoryError),
    Stack(StackError),
    PC(PCError),
    Storage(StorageError),

    InvalidRange,
    EmptyGas,
    EmptyBalance,
    CallstackOverflow,
}

impl From<MachineError> for EvalError {
    fn from(val: MachineError) -> EvalError {
        EvalError::Machine(val)
    }
}

#[derive(Debug, Clone)]
pub enum VMError {
    Machine(MachineError),
}

impl From<MachineError> for VMError {
    fn from(val: MachineError) -> VMError {
        VMError::Machine(val)
    }
}

#[derive(Debug, Clone)]
pub enum RequireError {
    Account(Address),
    AccountCode(Address),
    Blockhash(M256),
}

impl From<RequireError> for EvalError {
    fn from(val: RequireError) -> EvalError {
        EvalError::Require(val)
    }
}

#[derive(Debug, Clone)]
pub enum CommitError {
    AlreadyCommitted,
}
