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
}

#[derive(Debug, Clone)]
/// Additional logs to be added due to the current VM
/// execution. SputnikVM defer calculation of log bloom to the client,
/// because VMs can run concurrently.
pub struct Log {
    pub address: Address,
    pub data: Vec<u8>,
    pub topics: Vec<M256>,
}

#[derive(Debug, Clone, Copy)]
/// Patches applied to the current blockchain.
pub enum Patch {
    None,
    Homestead,
    EIP150,
    EIP160
}

impl Patch {
    /// The homestead patch.
    pub fn homestead(&self) -> bool {
        match self {
            &Patch::None => false,
            _ => true,
        }
    }

    /// The homestead and EIP150 patch.
    pub fn eip150(&self) -> bool {
        match self {
            &Patch::None | &Patch::Homestead => false,
            _ => true,
        }
    }

    /// The homestead, EIP150 and EIP160 patch.
    pub fn eip160(&self) -> bool {
        match self {
            &Patch::None | &Patch::Homestead | &Patch::EIP150 => false,
            _ => true,
        }
    }
}
