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
    pub gas_limit: Gas,
    pub gas_price: Gas,
    pub origin: Address,
    pub value: M256,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct Log {
    pub address: Address,
    pub data: Vec<u8>,
    pub topics: Vec<M256>,
}

#[derive(Debug, Clone, Copy)]
pub enum Patch {
    None,
    Homestead,
    EIP150,
    EIP160
}

impl Patch {
    pub fn homestead(&self) -> bool {
        match self {
            &Patch::None => false,
            _ => true,
        }
    }

    pub fn eip150(&self) -> bool {
        match self {
            &Patch::None | &Patch::Homestead => false,
            _ => true,
        }
    }

    pub fn eip160(&self) -> bool {
        match self {
            &Patch::None | &Patch::Homestead | &Patch::EIP150 => false,
            _ => true,
        }
    }
}
