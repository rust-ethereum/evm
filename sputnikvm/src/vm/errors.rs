use utils::address::Address;
use utils::bigint::M256;

#[derive(Debug, Clone)]
pub enum MemoryError {
    IndexNotSupported,
}

#[derive(Debug, Clone)]
pub enum StackError {
    Overflow,
    Underflow,
}

#[derive(Debug, Clone)]
pub enum PCError {
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
