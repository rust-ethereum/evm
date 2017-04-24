use utils::bigint::{U256, M256};
use utils::gas::Gas;
use utils::address::Address;
use utils::hash::H256;

use std::collections::HashMap;
use std::slice;

pub trait Block {
    // Account information
    fn account_code(&self, address: Address) -> &[u8];
    fn set_account_code(&mut self, address: Address, code: &[u8]);
    fn balance(&self, address: Address) -> U256;
    fn set_balance(&mut self, address: Address, balance: U256);
    fn account_storage(&self, address: Address, index: M256) -> M256;
    fn set_account_storage(&mut self, address: Address, index: M256, val: M256);

    // Block information
    fn coinbase(&self) -> Address;
    fn timestamp(&self) -> M256;
    fn number(&self) -> M256;
    fn difficulty(&self) -> M256;
    fn gas_limit(&self) -> Gas;
    fn blockhash(&self, n: M256) -> H256;

    // Actions
    fn log(&mut self, address: Address, data: &[u8], topics: &[M256]);
    fn create_account(&mut self, code: &[u8]) -> Option<Address>;
}

pub struct FakeVectorBlock {
    storages: HashMap<Address, Vec<M256>>,
}

impl FakeVectorBlock {
    pub fn new() -> FakeVectorBlock {
        FakeVectorBlock {
            storages: HashMap::new()
        }
    }
}

static EMPTY: [u8; 0] = [];

impl Block for FakeVectorBlock {
    fn account_code(&self, address: Address) -> &[u8] {
        EMPTY.as_ref()
    }

    fn set_account_code(&mut self, address: Address, code: &[u8]) {
        unimplemented!()
    }

    fn set_balance(&mut self, address: Address, balance: U256) {
        unimplemented!()
    }

    fn create_account(&mut self, code: &[u8]) -> Option<Address> {
        None
    }

    fn coinbase(&self) -> Address {
        Address::default()
    }

    fn balance(&self, address: Address) -> U256 {
        U256::zero()
    }

    fn timestamp(&self) -> M256 {
        M256::zero()
    }

    fn number(&self) -> M256 {
        M256::zero()
    }

    fn difficulty(&self) -> M256 {
        M256::zero()
    }

    fn gas_limit(&self) -> Gas {
        Gas::zero()
    }

    fn account_storage(&self, address: Address, index: M256) -> M256 {
        match self.storages.get(&address) {
            None => M256::zero(),
            Some(ref ve) => {
                let index: usize = index.into();

                match ve.get(index) {
                    Some(&v) => v,
                    None => M256::zero()
                }
            }
        }
    }

    fn set_account_storage(&mut self, address: Address, index: M256, val: M256) {
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

    fn log(&mut self, address: Address, data: &[u8], topics: &[M256]) {
        unimplemented!()
    }

    fn blockhash(&self, n: M256) -> H256 {
        H256::default()
    }
}
