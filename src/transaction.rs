// In EVM we implement the code execution model as far as the
// transaction level. Blockchain, blocks and accounts should be
// handled by the invoking Ethereum Classic client, as copying the
// world state every time the EVM gets invoked might be expensive.

use utils::bigint::M256;
use utils::address::Address;
use utils::gas::Gas;

pub trait Transaction {
    fn gas_price(&self) -> Gas;
    fn gas_limit(&self) -> Gas;
    fn sender(&self) -> Address;
    fn callee(&self) -> Address;
    fn originator(&self) -> Address;
    fn value(&self) -> M256;
    fn data(&self) -> Option<&[u8]>; // Only for message call transaction.
    fn init(&self) -> Option<&[u8]>; // Only for account initialization procedure.
}

pub struct VectorTransaction {
    gas_price: Gas,
    gas_limit: Gas,
    sender: Address,
    callee: Address,
    originator: Address,
    value: M256,
    data: Option<Vec<u8>>,
    init: Option<Vec<u8>>,
}

impl VectorTransaction {
    pub fn message_call(from: Address, to: Address,
                        value: M256, data: &[u8], gas_limit: Gas) -> VectorTransaction {
        VectorTransaction {
            gas_price: Gas::zero(),
            gas_limit: gas_limit,
            sender: from,
            callee: to,
            originator: from,
            value: value,
            data: Some(data.into()),
            init: None
        }
    }
}

impl Transaction for VectorTransaction {
    fn gas_price(&self) -> Gas {
        self.gas_price
    }

    fn gas_limit(&self) -> Gas {
        self.gas_limit
    }

    fn sender(&self) -> Address {
        self.sender
    }

    fn callee(&self) -> Address {
        self.callee
    }

    fn originator(&self) -> Address {
        self.originator
    }

    fn value(&self) -> M256 {
        self.value
    }

    fn data(&self) -> Option<&[u8]> {
        if self.data.is_some() {
            Some(self.data.as_ref().unwrap().as_ref())
        } else {
            None
        }
    }

    fn init(&self) -> Option<&[u8]> {
        if self.init.is_some() {
            Some(self.init.as_ref().unwrap().as_ref())
        } else {
            None
        }
    }
}
