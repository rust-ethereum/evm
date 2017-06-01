use std::collections::hash_map;
use utils::gas::Gas;
use utils::address::Address;
use utils::bigint::{U256, M256};

use super::errors::{RequireError, CommitError};
use super::{Context, ContextVM, VM, AccountState, BlockhashState, Patch, BlockHeader, Memory,
            VMStatus, AccountCommitment, Log, Account};

#[derive(Debug, Clone)]
pub enum Transaction {
    MessageCall {
        address: Address,
        caller: Address,
        gas_price: Gas,
        gas_limit: Gas,
        value: U256,
        data: Vec<u8>,
    },
    ContractCreation {
        caller: Address,
        gas_price: Gas,
        gas_limit: Gas,
        value: U256,
        init: Vec<u8>,
    },
}

impl Transaction {
    #[allow(unused_variables)]
    pub fn intrinsic_gas(&self) -> Gas {
        unimplemented!()
    }

    #[allow(unused_variables)]
    pub fn into_context(self, origin: Option<Address>,
                        account_state: &AccountState) -> Result<Context, RequireError> {
        unimplemented!()
    }

    #[allow(unused_variables)]
    pub fn gas_limit(&self) -> Gas {
        unimplemented!()
    }
}

enum TransactionVMState<M> {
    Running {
        vm: ContextVM<M>,
        intrinsic_gas: Gas,
    },
    Constructing {
        transaction: Transaction,
        block: BlockHeader,
        patch: &'static Patch,

        account_state: AccountState,
        blockhash_state: BlockhashState,
    },
}

pub struct TransactionVM<M>(TransactionVMState<M>);

impl<M: Memory + Default> TransactionVM<M> {
    pub fn new(transaction: Transaction, block: BlockHeader, patch: &'static Patch) -> Self {
        TransactionVM(TransactionVMState::Constructing {
            transaction: transaction,
            block: block,
            patch: patch,

            account_state: AccountState::default(),
            blockhash_state: BlockhashState::default(),
        })
    }
}

impl<M: Memory + Default> VM for TransactionVM<M> {
    fn commit_account(&mut self, commitment: AccountCommitment) -> Result<(), CommitError> {
        match self.0 {
            TransactionVMState::Running { ref mut vm, .. } => vm.commit_account(commitment),
            TransactionVMState::Constructing { ref mut account_state, .. } => account_state.commit(commitment),
        }
    }

    fn commit_blockhash(&mut self, number: M256, hash: M256) -> Result<(), CommitError> {
        match self.0 {
            TransactionVMState::Running { ref mut vm, .. } => vm.commit_blockhash(number, hash),
            TransactionVMState::Constructing { ref mut blockhash_state, .. } => blockhash_state.commit(number, hash),
        }
    }

    fn status(&self) -> VMStatus {
        match self.0 {
            TransactionVMState::Running { ref vm, .. } => vm.status(),
            TransactionVMState::Constructing { .. } => VMStatus::Running,
        }
    }

    fn step(&mut self) -> Result<(), RequireError> {
        let mut cgas: Option<Gas> = None;
        let mut ccontext: Option<Context> = None;
        let mut cblock: Option<BlockHeader> = None;
        let mut cpatch: Option<&'static Patch> = None;
        let mut caccount_state: Option<AccountState> = None;
        let mut cblockhash_state: Option<BlockhashState> = None;

        match self.0 {
            TransactionVMState::Running { ref mut vm, .. } => return vm.step(),
            TransactionVMState::Constructing {
                ref transaction, ref block, ref patch,
                ref account_state, ref blockhash_state } => {

                cgas = Some(transaction.intrinsic_gas());
                ccontext = Some(transaction.clone().into_context(None, account_state)?);
                cblock = Some(block.clone());
                cpatch = Some(patch);
                caccount_state = Some(account_state.clone());
                cblockhash_state = Some(blockhash_state.clone());
            }
        }

        self.0 = TransactionVMState::Running {
            vm: ContextVM::with_states(ccontext.unwrap(), cblock.unwrap(), cpatch.unwrap(),
                                       caccount_state.unwrap(), cblockhash_state.unwrap()),
            intrinsic_gas: cgas.unwrap(),
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
            TransactionVMState::Constructing { ref transaction, .. } => transaction.gas_limit(),
        }
    }

    fn used_gas(&self) -> Gas {
        match self.0 {
            TransactionVMState::Running { ref vm, .. } => vm.used_gas(),
            TransactionVMState::Constructing { .. } => Gas::zero(),
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
}
