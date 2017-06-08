//! Parameters used by the VM.

use utils::gas::Gas;
use utils::address::Address;
use utils::bigint::{M256, U256};

#[derive(Debug, Clone)]
/// Block header.
pub struct BlockHeader {
    pub coinbase: Address,
    pub timestamp: M256,
    pub number: M256,
    pub difficulty: M256,
    pub gas_limit: Gas
}

#[derive(Debug, Clone)]
/// A VM context. See the Yellow Paper for more information.
pub struct Context {
    pub address: Address,
    pub caller: Address,
    pub code: Vec<u8>,
    pub data: Vec<u8>,
    pub gas_limit: Gas,
    pub gas_price: Gas,
    pub origin: Address,
    pub value: U256,
    pub create: bool,
}

#[derive(Debug, Clone, PartialEq)]
/// Additional logs to be added due to the current VM
/// execution. SputnikVM defer calculation of log bloom to the client,
/// because VMs can run concurrently.
pub struct Log {
    pub address: Address,
    pub data: Vec<u8>,
    pub topics: Vec<M256>,
}
