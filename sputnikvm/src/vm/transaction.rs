use std::collections::hash_map;
use utils::gas::Gas;
use utils::address::Address;
use utils::bigint::{U256, M256};
use rlp::RlpStream;
use tiny_keccak::Keccak;

use super::errors::{RequireError, CommitError};
use super::{Context, ContextVM, VM, AccountState, BlockhashState, Patch, BlockHeader, Memory,
            VMStatus, AccountCommitment, Log, Account};

const G_TXDATAZERO: usize = 4;
const G_TXDATANONZERO: usize = 68;
const G_TRANSACTION: usize = 21000;

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
    pub fn intrinsic_gas(&self, patch: &'static Patch) -> Gas {
        let mut gas = Gas::from(G_TRANSACTION);
        match self {
            &Transaction::MessageCall {
                ref data, ..
            } => {
                for d in data {
                    if *d == 0 {
                        gas = gas + Gas::from(G_TXDATAZERO);
                    } else {
                        gas = gas + Gas::from(G_TXDATANONZERO);
                    }
                }
            },
            &Transaction::ContractCreation {
                ref init, ..
            } => {
                gas = gas + Gas::from(patch.gas_transaction_create);
                for d in init {
                    if *d == 0 {
                        gas = gas + Gas::from(G_TXDATAZERO);
                    } else {
                        gas = gas + Gas::from(G_TXDATANONZERO);
                    }
                }
            }
        }
        return gas;
    }

    pub fn into_context(self, upfront: Gas, origin: Option<Address>,
                        account_state: &AccountState) -> Result<Context, RequireError> {
        match self {
            Transaction::MessageCall {
                address, caller, gas_price, gas_limit, value, data
            } => {
                Ok(Context {
                    address, caller, data, gas_price, value,
                    gas_limit: gas_limit - upfront,
                    code: account_state.code(address)?.into(),
                    origin: origin.unwrap_or(caller),
                })
            },
            Transaction::ContractCreation {
                caller, gas_price, gas_limit, value, init,
            } => {
                let nonce = account_state.nonce(caller)?;
                let mut rlp = RlpStream::new();
                rlp.begin_list(2);
                rlp.append(&caller);
                rlp.append(&nonce);
                let mut address_array = [0u8; 32];
                let mut sha3 = Keccak::new_keccak256();
                sha3.update(rlp.out().as_slice());
                sha3.finalize(&mut address_array);
                let address = Address::from(M256::from(address_array));

                Ok(Context {
                    address, caller, gas_price, value,
                    gas_limit: gas_limit - upfront,
                    data: Vec::new(),
                    code: init,
                    origin: origin.unwrap_or(caller),
                })
            }
        }
    }

    pub fn gas_limit(&self) -> Gas {
        match self {
            &Transaction::MessageCall { gas_limit, .. } => gas_limit,
            &Transaction::ContractCreation { gas_limit, .. } => gas_limit,
        }
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

                cgas = Some(transaction.intrinsic_gas(patch));
                ccontext = Some(transaction.clone().into_context(cgas.unwrap(), None, account_state)?);
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
            TransactionVMState::Running { ref vm, intrinsic_gas } => vm.used_gas() + intrinsic_gas,
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
