use utils::gas::Gas;
use utils::address::Address;
use utils::bigint::U256;

use super::errors::RequireError;
use super::{Context, ContextVM, AccountState, BlockhashState, Patch, BlockHeader, Memory};

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
