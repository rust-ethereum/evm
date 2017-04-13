use utils::u256::U256;
use utils::gas::Gas;
use utils::address::Address;
use utils::hash::H256;

pub trait Block {
    fn account_code(&self, address: Address) -> Option<&[u8]>;
    fn coinbase(&self) -> Address;
    fn balance(&self, address: Address) -> Option<U256>;
    fn timestamp(&self) -> U256;
    fn number(&self) -> U256;
    fn difficulty(&self) -> U256;
    fn gas_limit(&self) -> Gas;
}

pub struct FakeBlock;

impl Block for FakeBlock {
    fn account_code(&self, address: Address) -> Option<&[u8]> {
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
}

impl Default for FakeBlock {
    fn default() -> FakeBlock {
        FakeBlock
    }
}

pub trait Blockchain {
    fn blockhash(n: U256) -> H256;
}

pub struct FakeBlockchain;

impl Blockchain for FakeBlockchain {
    fn blockhash(n: U256) -> H256 {
        H256::default()
    }
}

impl Default for FakeBlockchain {
    fn default() -> FakeBlockchain {
        FakeBlockchain
    }
}
