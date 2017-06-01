use utils::gas::Gas;
use utils::address::Address;
use utils::bigint::U256;

use super::errors::RequireError;
use super::{Context, AccountState};

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
                        account_state: AccountState) -> Result<Context, RequireError> {
        unimplemented!()
    }
}
