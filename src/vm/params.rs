//! Parameters used by the VM.

use util::gas::Gas;
use util::address::Address;
use util::bigint::{M256, U256};

#[derive(Debug, Clone)]
/// Block header.
pub struct BlockHeader {
    /// Block coinbase, the address that mines the block.
    pub coinbase: Address,
    /// Block timestamp.
    pub timestamp: M256,
    /// The current block number.
    pub number: M256,
    /// Difficulty of the block.
    pub difficulty: M256,
    /// Total block gas limit.
    pub gas_limit: Gas
}

#[derive(Debug, Clone)]
/// A VM context. See the Yellow Paper for more information.
pub struct Context {
    /// Address that is executing this runtime.
    pub address: Address,
    /// Caller of the runtime.
    pub caller: Address,
    /// Code to be executed.
    pub code: Vec<u8>,
    /// Data associated with this execution.
    pub data: Vec<u8>,
    /// Gas limit.
    pub gas_limit: Gas,
    /// Gas price.
    pub gas_price: Gas,
    /// The origin of the context. The same as caller when it is from
    /// a transaction.
    pub origin: Address,
    /// Value passed for this runtime.
    pub value: U256,
    /// Apprent value in the execution context.
    pub apprent_value: U256,
}

#[derive(Debug, Clone, PartialEq)]
/// Additional logs to be added due to the current VM
/// execution. SputnikVM defer calculation of log bloom to the client,
/// because VMs can run concurrently.
pub struct Log {
    /// Log appended to address.
    pub address: Address,
    /// Log data.
    pub data: Vec<u8>,
    /// Log topics.
    pub topics: Vec<M256>,
}
