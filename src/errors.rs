//! VM errors

use bigint::{Address, U256};

#[derive(Debug, Clone)]
/// Errors when trying to validate the transaction.
pub enum PreExecutionError {
    /// The caller is invalid.
    InvalidCaller,
    /// Nonce of the caller does not equal.
    InvalidNonce,
    /// Balance from the caller is insufficient.
    InsufficientBalance,
    /// Gas limit is smaller than the intrinsic gas required.
    InsufficientGasLimit,
}

#[derive(Debug, Clone)]
/// Errors that can be written on chain.
pub enum OnChainError {
    /// Stack is overflowed (pushed more than 1024 items to the
    /// stack).
    StackOverflow,
    /// Stack is underflowed (poped an empty stack).
    StackUnderflow,
    /// The opcode is invalid and the PC is not able to convert it to
    /// an instruction.
    InvalidOpcode,
    /// PC jumped to an invalid jump destination.
    BadJumpDest,
    /// PC overflowed (tries to read the next opcode which is already
    /// the end of the code). In Yellow Paper, this is categorized the
    /// same as InvalidOpcode.
    PCOverflow,
    /// Not enough gas to continue.
    EmptyGas,
    /// For instruction that requires reading a range, it is
    /// invalid. This in the Yellow Paper is covered by EmptyGas.
    InvalidRange,
    /// In static context but does mutation.
    NotStatic,
    /// Invoked by REVERT opcode.
    Revert,
}

impl From<OnChainError> for RuntimeError {
    fn from(val: OnChainError) -> RuntimeError {
        RuntimeError::OnChain(val)
    }
}

impl From<OnChainError> for EvalOnChainError {
    fn from(val: OnChainError) -> EvalOnChainError {
        EvalOnChainError::OnChain(val)
    }
}

impl From<OnChainError> for EvalError {
    fn from(val: OnChainError) -> EvalError {
        EvalError::OnChain(val)
    }
}

#[derive(Debug, Clone)]
/// Errors when the VM detects that it does not support certain
/// operations.
pub enum NotSupportedError {
    /// The memory index is too large for the implementation of the VM to
    /// handle.
    MemoryIndexNotSupported,
    /// A particular precompiled contract is not supported.
    PrecompiledNotSupported,
}

impl From<NotSupportedError> for RuntimeError {
    fn from(val: NotSupportedError) -> RuntimeError {
        RuntimeError::NotSupported(val)
    }
}

impl From<NotSupportedError> for EvalError {
    fn from(val: NotSupportedError) -> EvalError {
        EvalError::NotSupported(val)
    }
}

#[derive(Debug, Clone)]
/// Runtime error. Can either be an on-chain error or a not-supported
/// error.
pub enum RuntimeError {
    /// On chain error.
    OnChain(OnChainError),
    /// Off chain error due to VM not supported.
    NotSupported(NotSupportedError),
}

impl From<RuntimeError> for EvalError {
    fn from(val: RuntimeError) -> EvalError {
        match val {
            RuntimeError::OnChain(err) =>
                EvalError::OnChain(err),
            RuntimeError::NotSupported(err) =>
                EvalError::NotSupported(err),
        }
    }
}

#[derive(Debug, Clone)]
/// Eval on-chain error. Can either be an on-chain error or a require
/// error.
pub enum EvalOnChainError {
    /// On chain error.
    OnChain(OnChainError),
    /// Require error for additional accounts.
    Require(RequireError),
}

impl From<EvalOnChainError> for EvalError {
    fn from(val: EvalOnChainError) -> EvalError {
        match val {
            EvalOnChainError::OnChain(err) =>
                EvalError::OnChain(err),
            EvalOnChainError::Require(err) =>
                EvalError::Require(err),
        }
    }
}

#[derive(Debug, Clone)]
/// Eval error. On-chain error, not-supported error or require error.
pub enum EvalError {
    /// On chain error.
    OnChain(OnChainError),
    /// Off chain error due to VM not supported.
    NotSupported(NotSupportedError),
    /// Require error for additional accounts.
    Require(RequireError),
}

#[derive(Debug, Clone)]
/// Errors stating that the VM requires additional information to
/// continue running.
pub enum RequireError {
    /// Requires the account at address for the VM to continue
    /// running, this should usually be dealt by
    /// `vm.commit_account(AccountCommitment::Full { .. })` or
    /// `vm.commit_account(AccountCommitment::Nonexist(..))`.
    Account(Address),
    /// Requires the account code at address for the VM to continue
    /// running, this should usually be dealt by
    /// `vm.commit_account(AccountCommitment::Code { .. })`.
    AccountCode(Address),
    /// Requires the current value of the storage for the VM to
    /// continue running, this should usually be dealt by
    /// `vm.commit_account(AccountCommitment::Storage { .. }`.
    AccountStorage(Address, U256),
    /// Requires the blockhash for the VM to continue running, this
    /// should usually be dealt by `vm.commit_blockhash(..)`.
    Blockhash(U256),
}

impl From<RequireError> for EvalError {
    fn from(val: RequireError) -> EvalError {
        EvalError::Require(val)
    }
}

impl From<RequireError> for EvalOnChainError {
    fn from(val: RequireError) -> EvalOnChainError {
        EvalOnChainError::Require(val)
    }
}

#[derive(Debug, Clone)]
/// Errors returned when committing a new information.
pub enum CommitError {
    /// The commitment is invalid.
    InvalidCommitment,
    /// The commitment has already been committed.
    AlreadyCommitted,
}
