use std::collections::hash_map;
use std::cmp::min;
use utils::gas::Gas;
use utils::address::Address;
use utils::bigint::{U256, M256};
use rlp::RlpStream;
use tiny_keccak::Keccak;

use super::errors::{RequireError, CommitError};
use super::{Context, ContextVM, VM, AccountState, BlockhashState, Patch, BlockHeader, Memory,
            VMStatus, AccountCommitment, Log, Account, MachineStatus};

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
    pub fn caller(&self) -> Address {
        match self {
            &Transaction::MessageCall { caller, .. } => caller,
            &Transaction::ContractCreation { caller, .. } => caller,
        }
    }

    pub fn address(&self, account_state: &AccountState) -> Result<Address, RequireError> {
        match self {
            &Transaction::MessageCall { address, .. } => Ok(address),
            &Transaction::ContractCreation { caller, .. } => {
                let nonce = account_state.nonce(caller)?;
                let mut rlp = RlpStream::new_list(2);
                rlp.append(&caller);
                rlp.append(&nonce);
                let mut address_array = [0u8; 32];
                let mut sha3 = Keccak::new_keccak256();
                sha3.update(rlp.out().as_slice());
                sha3.finalize(&mut address_array);

                Ok(Address::from(M256::from(address_array)))
            },
        }
    }

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
                        account_state: &mut AccountState, is_code: bool) -> Result<Context, RequireError> {
        let address = self.address(account_state)?;

        match self {
            Transaction::MessageCall {
                caller, gas_price, gas_limit, value, data, ..
            } => {
                account_state.require(caller)?;
                account_state.require_code(address)?;

                if !is_code {
                    let nonce = account_state.nonce(caller).unwrap();
                    account_state.set_nonce(caller, nonce + M256::from(1u64)).unwrap();
                }

                Ok(Context {
                    address, caller, data, gas_price, value,
                    gas_limit: gas_limit - upfront,
                    code: account_state.code(address).unwrap().into(),
                    origin: origin.unwrap_or(caller),
                })
            },
            Transaction::ContractCreation {
                caller, gas_price, gas_limit, value, init,
            } => {
                account_state.require(caller)?;

                let nonce = account_state.nonce(caller).unwrap();
                account_state.set_nonce(caller, nonce + M256::from(1u64)).unwrap();

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

    pub fn gas_price(&self) -> Gas {
        match self {
            &Transaction::MessageCall { gas_price, .. } => gas_price,
            &Transaction::ContractCreation { gas_price, .. } => gas_price,
        }
    }

    pub fn preclaimed_value(&self) -> U256 {
        (self.gas_limit() * self.gas_price()).into()
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

    pub fn with_previous(transaction: Transaction, block: BlockHeader, patch: &'static Patch,
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

    fn commit_blockhash(&mut self, number: M256, hash: M256) -> Result<(), CommitError> {
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
        let mut cblock: Option<BlockHeader> = None;
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
                            vm.machines[0].code_deposit()?;
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

                ccode_deposit = Some(match transaction {
                    &Transaction::MessageCall { .. } => false,
                    &Transaction::ContractCreation { .. } => true,
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
            TransactionVMState::Constructing { ref transaction, .. } => transaction.gas_limit(),
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
