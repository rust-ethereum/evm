//! Transaction related functionality.

use std::collections::hash_map;
use std::cmp::min;
use std::str::FromStr;
use util::gas::Gas;
use util::address::Address;
use util::bigint::{U256, H256, M256};
use rlp::RlpStream;
use sha3::{Digest, Keccak256};

use super::errors::{RequireError, CommitError, PreExecutionError};
use super::{Context, ContextVM, VM, AccountState, BlockhashState, Patch, HeaderParams, Memory,
            VMStatus, AccountCommitment, Log, Account, MachineStatus};
use block::{Transaction, TransactionAction};

const G_TXDATAZERO: usize = 4;
const G_TXDATANONZERO: usize = 68;
const G_TRANSACTION: usize = 21000;

macro_rules! system_address {
    () => {
        Address::from_str("0xffffffffffffffffffffffffffffffffffffffff").unwrap()
    }
}

/// ## About SYSTEM transaction
///
/// SYSTEM transaction in Ethereum is something that cannot be
/// executed by the user, and is enforced by the blockchain rules. The
/// SYSTEM transaction does not have a caller. When executed in EVM,
/// however, the CALLER opcode would return
/// 0xffffffffffffffffffffffffffffffffffffffff. As a result, when
/// executing a message call or a contract creation, nonce are not
/// changed. A SYSTEM transaction must have gas_price set to zero.

#[derive(Debug, Clone)]
/// Represents an Ethereum transaction.
pub struct ValidTransaction {
    /// Caller of this transaction. If caller is None, then this is a
    /// SYSTEM transaction.
    pub caller: Option<Address>,
    /// Gas price of this transaction.
    pub gas_price: Gas,
    /// Gas limit of this transaction.
    pub gas_limit: Gas,
    /// Transaction action.
    pub action: TransactionAction,
    /// Value of this transaction.
    pub value: U256,
    /// Data or init associated with this transaction.
    pub input: Vec<u8>,
}

impl ValidTransaction {
    /// Create a valid transaction from a block transaction. Caller is
    /// always Some.
    pub fn from_transaction(
        transaction: &Transaction, account_state: &AccountState, patch: &'static Patch
    ) -> Result<Result<ValidTransaction, PreExecutionError>, RequireError> {
        let caller = match transaction.caller() {
            Ok(val) => val,
            Err(_) => return Ok(Err(PreExecutionError::InvalidCaller)),
        };

        let nonce = account_state.nonce(caller)?;
        if nonce != transaction.nonce {
            return Ok(Err(PreExecutionError::InvalidNonce));
        }

        let valid = ValidTransaction {
            caller: Some(caller),
            gas_price: transaction.gas_price,
            gas_limit: transaction.gas_limit,
            action: transaction.action.clone(),
            value: transaction.value,
            input: transaction.input.clone(),
        };

        if valid.gas_limit < valid.intrinsic_gas(patch) {
            return Ok(Err(PreExecutionError::InsufficientGasLimit));
        }

        let balance = account_state.balance(caller)?;
        if balance < valid.preclaimed_value() {
            return Ok(Err(PreExecutionError::InsufficientBalance));
        }

        Ok(Ok(valid))
    }
}

impl ValidTransaction {
    /// To address of the transaction.
    pub fn address(&self, account_state: &AccountState) -> Result<Address, RequireError> {
        match self.action.clone() {
            TransactionAction::Call(address) => Ok(address),
            TransactionAction::Create => {
                let caller = self.caller.unwrap_or(system_address!());
                let nonce = if self.caller.is_some() {
                    account_state.nonce(self.caller.unwrap())?
                } else {
                    U256::zero()
                };
                let mut rlp = RlpStream::new_list(2);
                rlp.append(&caller);
                rlp.append(&nonce);

                let address = Address::from(M256::from(Keccak256::digest(rlp.out().as_slice()).as_slice()));
                Ok(address)
            },
        }
    }

    /// Intrinsic gas to be paid in prior to this transaction
    /// execution.
    pub fn intrinsic_gas(&self, patch: &'static Patch) -> Gas {
        let mut gas = Gas::from(G_TRANSACTION);
        if self.action == TransactionAction::Create {
            gas = gas + Gas::from(patch.gas_transaction_create);
        }
        for d in &self.input {
            if *d == 0 {
                gas = gas + Gas::from(G_TXDATAZERO);
            } else {
                gas = gas + Gas::from(G_TXDATANONZERO);
            }
        }
        return gas;
    }

    /// Convert this transaction into a context. Note that this will
    /// change the account state.
    pub fn into_context(self, upfront: Gas, origin: Option<Address>,
                        account_state: &mut AccountState, is_code: bool) -> Result<Context, RequireError> {
        let address = self.address(account_state)?;

        match self.action {
            TransactionAction::Call(_) => {
                if self.caller.is_some() {
                    account_state.require(self.caller.unwrap())?;
                }
                account_state.require_code(address)?;

                if self.caller.is_some() && !is_code {
                    let nonce = account_state.nonce(self.caller.unwrap()).unwrap();
                    account_state.set_nonce(self.caller.unwrap(), nonce + U256::from(1u64)).unwrap();
                }

                Ok(Context {
                    address,
                    caller: self.caller.unwrap_or(system_address!()),
                    data: self.input,
                    gas_price: self.gas_price,
                    value: self.value,
                    gas_limit: self.gas_limit - upfront,
                    code: account_state.code(address).unwrap().into(),
                    origin: origin.unwrap_or(self.caller.unwrap_or(system_address!())),
                    apprent_value: self.value,
                })
            },
            TransactionAction::Create => {
                if self.caller.is_some() {
                    account_state.require(self.caller.unwrap())?;
                    let nonce = account_state.nonce(self.caller.unwrap()).unwrap();
                    account_state.set_nonce(self.caller.unwrap(), nonce + U256::from(1u64)).unwrap();
                }

                Ok(Context {
                    address,
                    caller: self.caller.unwrap_or(system_address!()),
                    gas_price: self.gas_price,
                    value: self.value,
                    gas_limit: self.gas_limit - upfront,
                    data: Vec::new(),
                    code: self.input,
                    origin: origin.unwrap_or(self.caller.unwrap_or(system_address!())),
                    apprent_value: self.value,
                })
            },
        }
    }

    /// When the execution of a transaction begins, this preclaimed
    /// value is deducted from the account.
    pub fn preclaimed_value(&self) -> U256 {
        (self.gas_limit * self.gas_price).into()
    }
}

enum TransactionVMState<M> {
    Running {
        vm: ContextVM<M>,
        intrinsic_gas: Gas,
        preclaimed_value: U256,
        finalized: bool,
        code_deposit: bool,
        fresh_account_state: AccountState,
    },
    Constructing {
        transaction: ValidTransaction,
        block: HeaderParams,
        patch: &'static Patch,

        account_state: AccountState,
        blockhash_state: BlockhashState,
    },
}

/// A VM that executes using a transaction and block information.
pub struct TransactionVM<M>(TransactionVMState<M>);

impl<M: Memory + Default> TransactionVM<M> {
    /// Create a new VM using the given transaction, block header and
    /// patch. This VM runs at the transaction level.
    pub fn new(transaction: ValidTransaction, block: HeaderParams, patch: &'static Patch) -> Self {
        TransactionVM(TransactionVMState::Constructing {
            transaction: transaction,
            block: block,
            patch: patch,

            account_state: AccountState::default(),
            blockhash_state: BlockhashState::default(),
        })
    }

    /// Create a new VM with the result of the previous VM. This is
    /// usually used by transaction for chaining them.
    pub fn with_previous(transaction: ValidTransaction, block: HeaderParams, patch: &'static Patch,
                         vm: &TransactionVM<M>) -> Self {
        TransactionVM(TransactionVMState::Constructing {
            transaction: transaction,
            block: block,
            patch: patch,

            account_state: match vm.0 {
                TransactionVMState::Constructing { ref account_state, .. } =>
                    account_state.clone(),
                TransactionVMState::Running { ref vm, .. } =>
                    vm.machines[0].state().account_state.clone(),
            },
            blockhash_state: match vm.0 {
                TransactionVMState::Constructing { ref blockhash_state, .. } =>
                    blockhash_state.clone(),
                TransactionVMState::Running { ref vm, .. } =>
                    vm.machines[0].state().blockhash_state.clone(),
            },
        })
    }

    /// Returns the real used gas by the transaction. This is what is
    /// recorded in the transaction receipt.
    pub fn real_used_gas(&self) -> Gas {
        match self.0 {
            TransactionVMState::Running { ref vm, intrinsic_gas, .. } => {
                match vm.machines[0].status() {
                    MachineStatus::ExitedErr(_) =>
                        vm.machines[0].state().context.gas_limit + intrinsic_gas,
                    MachineStatus::ExitedOk => {
                        let total_used = vm.machines[0].state().memory_gas() + vm.machines[0].state().used_gas + intrinsic_gas;
                        let refund_cap = total_used / Gas::from(2u64);
                        let refunded = min(refund_cap, vm.machines[0].state().refunded_gas);
                        total_used - refunded
                    }
                    _ => Gas::zero(),
                }
            }
            TransactionVMState::Constructing { .. } => Gas::zero(),
        }
    }
}

impl<M: Memory + Default> VM for TransactionVM<M> {
    fn commit_account(&mut self, commitment: AccountCommitment) -> Result<(), CommitError> {
        match self.0 {
            TransactionVMState::Running { ref mut vm, .. } => vm.commit_account(commitment),
            TransactionVMState::Constructing { ref mut account_state, .. } => account_state.commit(commitment),
        }
    }

    fn commit_blockhash(&mut self, number: U256, hash: H256) -> Result<(), CommitError> {
        match self.0 {
            TransactionVMState::Running { ref mut vm, .. } => vm.commit_blockhash(number, hash),
            TransactionVMState::Constructing { ref mut blockhash_state, .. } => blockhash_state.commit(number, hash),
        }
    }

    fn status(&self) -> VMStatus {
        match self.0 {
            TransactionVMState::Running { ref vm, finalized, .. } => {
                if !finalized {
                    VMStatus::Running
                } else {
                    vm.status()
                }
            },
            TransactionVMState::Constructing { .. } => VMStatus::Running,
        }
    }

    fn step(&mut self) -> Result<(), RequireError> {
        let mut cgas: Option<Gas> = None;
        let mut ccontext: Option<Context> = None;
        let mut cblock: Option<HeaderParams> = None;
        let mut cpatch: Option<&'static Patch> = None;
        let mut caccount_state: Option<AccountState> = None;
        let mut cblockhash_state: Option<BlockhashState> = None;
        let mut ccode_deposit: Option<bool> = None;
        let mut cpreclaimed_value: Option<U256> = None;

        let real_used_gas = self.real_used_gas();

        match self.0 {
            TransactionVMState::Running {
                ref mut vm,
                ref mut finalized,
                ref mut code_deposit,
                ref fresh_account_state,
                preclaimed_value,
                ..
            } => {
                match vm.status() {
                    VMStatus::Running => {
                        return vm.step();
                    },
                    _ => {
                        if *code_deposit {
                            vm.machines[0].code_deposit();
                            *code_deposit = false;
                            return Ok(());
                        }

                        if !*finalized {
                            vm.machines[0].finalize(real_used_gas, preclaimed_value,
                                                    fresh_account_state)?;
                            *finalized = true;
                            return Ok(());
                        }

                        return vm.step();
                    },
                }
            }
            TransactionVMState::Constructing {
                ref transaction, ref block, ref patch,
                ref mut account_state, ref blockhash_state } => {

                let address = transaction.address(account_state)?;
                account_state.require(address)?;

                ccode_deposit = Some(match transaction.action {
                    TransactionAction::Call(_) => false,
                    TransactionAction::Create => true,
                });
                cgas = Some(transaction.intrinsic_gas(patch));
                cpreclaimed_value = Some(transaction.preclaimed_value());
                ccontext = Some(transaction.clone().into_context(cgas.unwrap(), None, account_state, false)?);
                cblock = Some(block.clone());
                cpatch = Some(patch);
                caccount_state = Some(account_state.clone());
                cblockhash_state = Some(blockhash_state.clone());
            }
        }

        let account_state = caccount_state.unwrap();
        let mut vm = ContextVM::with_states(ccontext.unwrap(), cblock.unwrap(), cpatch.unwrap(),
                                            account_state.clone(),
                                            cblockhash_state.unwrap());

        if ccode_deposit.unwrap() {
            vm.machines[0].initialize_create(cpreclaimed_value.unwrap()).unwrap();
        } else {
            vm.machines[0].initialize_call(cpreclaimed_value.unwrap());
        }

        self.0 = TransactionVMState::Running {
            fresh_account_state: account_state,
            vm,
            intrinsic_gas: cgas.unwrap(),
            finalized: false,
            code_deposit: ccode_deposit.unwrap(),
            preclaimed_value: cpreclaimed_value.unwrap(),
        };

        Ok(())
    }

    fn accounts(&self) -> hash_map::Values<Address, Account> {
        match self.0 {
            TransactionVMState::Running { ref vm, .. } => vm.accounts(),
            TransactionVMState::Constructing { ref account_state, .. } => account_state.accounts(),
        }
    }

    fn out(&self) -> &[u8] {
        match self.0 {
            TransactionVMState::Running { ref vm, .. } => vm.out(),
            TransactionVMState::Constructing { .. } => &[],
        }
    }

    fn available_gas(&self) -> Gas {
        match self.0 {
            TransactionVMState::Running { ref vm, .. } => vm.available_gas(),
            TransactionVMState::Constructing { ref transaction, .. } => transaction.gas_limit,
        }
    }

    fn refunded_gas(&self) -> Gas {
        match self.0 {
            TransactionVMState::Running { ref vm, .. } => vm.refunded_gas(),
            TransactionVMState::Constructing { .. } => Gas::zero(),
        }
    }

    fn logs(&self) -> &[Log] {
        match self.0 {
            TransactionVMState::Running { ref vm, .. } => vm.logs(),
            TransactionVMState::Constructing { .. } => &[],
        }
    }

    fn removed(&self) -> &[Address] {
        match self.0 {
            TransactionVMState::Running { ref vm, .. } => vm.removed(),
            TransactionVMState::Constructing { .. } => &[],
        }
    }
}
