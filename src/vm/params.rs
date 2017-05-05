pub struct Block {
    pub coinbase: Address,
    pub timestamp: M256,
    pub number: M256,
    pub difficulty: M256,
    pub gas_limit: Gas
}

pub enum Transaction {
    MessageCall {
        pub gas_price: Gas,
        pub gas_limit: Gas,
        pub to: Address,
        pub originator: Address,
        pub caller: Address,
        pub value: M256,
        pub data: Vec<u8>,
    },
    ContractCreation {
        pub gas_price: Gas,
        pub gas_limit: Gas,
        pub originator: Address,
        pub caller: Address,
        pub value: M256,
        pub init: Vec<u8>
    }
}
