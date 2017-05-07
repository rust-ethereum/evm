use utils::gas::Gas;
use utils::address::Address;
use utils::bigint::{M256, U256};

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub coinbase: Address,
    pub timestamp: M256,
    pub number: M256,
    pub difficulty: M256,
    pub gas_limit: Gas
}

#[derive(Debug, Clone)]
pub struct Context {
    pub address: Address,
    pub caller: Address,
    pub code: Vec<u8>,
    pub data: Vec<u8>,
    pub gas: Gas,
    pub gas_price: Gas,
    pub origin: Address,
    pub value: M256,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct Log {
    pub data: Vec<u8>,
    pub topics: Vec<M256>,
}
