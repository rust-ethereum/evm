// In EVM we implement the code execution model as far as the
// transaction level. Blockchain, blocks and accounts should be
// handled by the invoking Ethereum Classic client, as copying the
// world state every time the EVM gets invoked might be expensive.

pub trait Transaction {
    fn gas_price(&self) -> U256;
    fn gas_limit(&self) -> U256;
    fn sender(&self) -> Address;
    fn callee(&self) -> Address;
    fn originator(&self) -> Address;
    fn value(&self) -> U256;
    fn data(&self) -> Option<&[u8]>; // Only for message call transaction.
    fn init(&self) -> Option<&[u8]>; // Only for account initialization procedure.
}

pub struct FakeTransaction {
    gas_price: U256,
    gas_limit: U256,
    sender: Address,
    callee: Address,
    originator: Address,
    value: U256,
    data: Option<Vec<u8>>,
    init: Option<Vec<u8>>,
}

impl FakeTransaction {
    pub fn message_call(value: U256, data: &[u8]) {
        FakeTransaction {
            gas_price: U256::zero(),
            gas_limit: U256::zero(),
            sender: Address::default(),
            callee: Address::default(),
            originator: Address::default(),
            value: value,
            data: Some(data.into()),
            init: None
        }
    }
}

impl Transaction for FakeTransaction {
    type A = FakeAccount;

    fn gas_price(&self) -> U256 {
        self.gas_price
    }

    fn gas_limit(&self) -> U256 {
        self.gas_limit
    }

    fn sender(&self) -> &FakeAccount {
        &self.sender
    }

    fn callee(&self) -> &FakeAccount {
        &self.callee
    }

    fn callee_mut(&mut self) -> &mut FakeAccount {
        &mut self.callee
    }

    fn originator(&self) -> &FakeAccount {
        if self.originator.is_none() {
            self.sender()
        } else {
            self.originator.as_ref().unwrap()
        }
    }

    fn value(&self) -> U256 {
        self.value
    }

    fn data(&self) -> Option<&[u8]> {
        self.data.map(|s| s.into())
    }

    fn init(&self) -> Option<&[u8]> {
        self.init.map(|s| s.into())
    }
}
