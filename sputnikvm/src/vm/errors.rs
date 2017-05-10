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

#[derive(Debug, Clone)]
pub enum StackError {
    Overflow,
    Underflow,
}

#[derive(Debug, Clone)]
pub enum PCError {
    InvalidOpcode,
    IndexNotSupported,
    BadJumpDest,
    Overflow,
}

#[derive(Debug, Clone)]
pub enum StorageError {
    IndexNotSupported,
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
}

#[derive(Debug, Clone)]
pub enum RequireError {
    Account(Address),
    AccountCode(Address),
    Blockhash(M256),
}

#[derive(Debug, Clone)]
pub enum CommitError {
    AlreadyCommitted,
}
