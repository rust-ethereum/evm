use utils::u256::U256;
use utils::gas::Gas;
use utils::address::Address;
use utils::hash::H256;

use std::collections::HashMap;

pub trait Block {
    fn account_code(&self, address: Address) -> Option<&[u8]>;
    fn coinbase(&self) -> Address;
    fn balance(&self, address: Address) -> Option<U256>;
    fn timestamp(&self) -> U256;
    fn number(&self) -> U256;
    fn difficulty(&self) -> U256;
    fn gas_limit(&self) -> Gas;
    fn create_account(&mut self, code: &[u8]) -> Option<Address>;
    fn account_storage(&self, address: Address, index: U256) -> U256;
    fn set_account_storage(&mut self, address: Address, index: U256, val: U256);
    fn log(&mut self, address: Address, data: &[u8], topics: &[U256]);
    fn blockhash(&self, n: U256) -> H256;
}

pub struct FakeVectorBlock {
    storages: HashMap<Address, Vec<U256>>,
}

impl FakeVectorBlock {
    pub fn new() -> FakeVectorBlock {
        FakeVectorBlock {
            storages: HashMap::new()
        }
    }
}

impl Block for FakeVectorBlock {
    fn account_code(&self, address: Address) -> Option<&[u8]> {
        None
    }

    fn create_account(&mut self, code: &[u8]) -> Option<Address> {
        None
    }

    fn coinbase(&self) -> Address {
        Address::default()
    }

    fn balance(&self, address: Address) -> Option<U256> {
        None
    }

    fn timestamp(&self) -> U256 {
        U256::zero()
    }

    fn number(&self) -> U256 {
        U256::zero()
    }

    fn difficulty(&self) -> U256 {
        U256::zero()
    }

    fn gas_limit(&self) -> Gas {
        Gas::zero()
    }

    fn account_storage(&self, address: Address, index: U256) -> U256 {
        match self.storages.get(&address) {
            None => U256::zero(),
            Some(ref ve) => {
                let index: usize = index.into();

                match ve.get(index) {
                    Some(&v) => v,
                    None => U256::zero()
                }
            }
        }
    }

    fn set_account_storage(&mut self, address: Address, index: U256, val: U256) {
        if self.storages.get(&address).is_none() {
            self.storages.insert(address, Vec::new());
        }

        let v = self.storages.get_mut(&address).unwrap();

        let index: usize = index.into();

        if v.len() <= index {
            v.resize(index + 1, 0.into());
        }

        v[index] = val;
    }

    fn log(&mut self, address: Address, data: &[u8], topics: &[U256]) {
        unimplemented!()
    }

    fn blockhash(&self, n: U256) -> H256 {
        H256::default()
    }
}
