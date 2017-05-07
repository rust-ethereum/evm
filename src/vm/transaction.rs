pub enum Transaction {
    MessageCall {
        gas_price: Gas,
        gas_limit: Gas,
        to: Address,
        originator: Address,
        caller: Address,
        value: M256,
        data: Vec<u8>,
    },
    ContractCreation {
        gas_price: Gas,
        gas_limit: Gas,
        originator: Address,
        caller: Address,
        value: M256,
        init: Vec<u8>
    }
}

impl Transaction {
    pub fn gas_limit(&self) -> Gas {
        match self {
            &Transaction::MessageCall {
                gas_limit: gas_limit,
                ..
            } => gas_limit,
            &Transaction::ContractCreation {
                gas_limit: gas_limit,
                ..
            } => gas_limit,
        }
    }

    pub fn value(&self) -> M256 {
        match self {
            &Transaction::MessageCall {
                value: value,
                ..
            } => value,
            &Transaction::ContractCreation {
                value: value,
                ..
            } => value,
        }
    }

    pub fn caller(&self) -> Address {
        match self {
            &Transaction::MessageCall {
                caller: caller,
                ..
            } => caller,
            &Transaction::ContractCreation {
                caller: caller,
                ..
            } => caller,
        }
    }

    pub fn originator(&self) -> Address {
        match self {
            &Transaction::MessageCall {
                originator: originator,
                ..
            } => originator,
            &Transaction::ContractCreation {
                originator: originator,
                ..
            } => originator,
        }
    }

    pub fn gas_price(&self) -> Gas {
        match self {
            &Transaction::MessageCall {
                gas_price: gas_price,
                ..
            } => gas_price,
            &Transaction::ContractCreation {
                gas_price: gas_price,
                ..
            } => gas_price,
        }
    }
}

pub struct TransactionMachine<M, S> {
    transaction: Transaction,
    machine: Option<ContextMachine<M, S>>,
} // Also handles initialization and finalization of the transaction (gas cost move, mostly)
