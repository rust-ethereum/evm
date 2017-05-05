pub enum Account<S> {
    Full {
        address: Address,
        balance: M256,
        storage: S,
        code: Vec<u8>,
        appending_logs: Vec<Log>,
    },
    Code {
        address: Address,
        code: Vec<u8>,
    },
    Remove(Address),
    Topup(Address, M256),
}

impl<S: Storage> Account<S> {
    pub fn address(&self) -> Address {
        match self {
            &Full {
                address: address,
                balance: _,
                storage: _,
                code: _,
                appending_logs: _,
            } => address,
            &Code {
                address: address,
                code: _,
            } => address,
            &Remove(address) => address,
            &Topup(address, _) => address,
        }
    }
}

impl<S: Storage> From<Commitment<S>> for Account<S> {
    fn from(val: Commitment<S>) -> Account<S> {
        match val {
            Commitment::Full {
                balance: balance,
                storage: storage,
                code: code,
            } => Account::Full {
                balance: balance,
                storage: storage,
                code: code,
                appending_logs: Vec::new(),
            },
            Commitment::Code {
                code: code,
            } => Account::Code {
                code: code
            },
        }
    }
}

pub enum Commitment<S> {
    Full {
        address: Address,
        balance: M256,
        storage: S,
        code: Option<Vec<u8>>,
    },
    Code {
        address: Address,
        code: Option<Vec<u8>>,
    },
}

impl<S: Storage> Commitment<S> {
    pub fn address(&self) -> Address {
        match self {
            &Full {
                address: address,
                balance: _,
                storage: _,
                code: _,
                appending_logs: _,
            } => address,
            &Code {
                address: address,
                code: _,
            } => address,
        }
    }
}

pub trait Storage {
    fn read(&self, index: M256) -> Result<M256>;
    fn write(&mut self, index: M256, value: M256);
}

pub struct MapStorage(HashMap<M256, M256>);

impl Storage for MapStorage {
    fn read(&self, index: M256) -> Result<M256> {
        match self.0.get(&address) {
            None => M256::zero(),
            Some(ref ve) => {
                match ve.get(&index) {
                    Some(&v) => Ok(v),
                    None => Ok(M256::zero())
                }
            }
        }
    }

    fn write(&mut self, index: M256, val: M256) -> Result<()> {
        if self.0.get(&address).is_none() {
            self.0.insert(address, HashMap::new());
        }

        let v = self.0.get_mut(&address).unwrap();
        v.insert(index, val);
        Ok(())
    }
}

pub struct Log {
    pub data: Vec<u8>,
    pub topics: Vec<M256>,
}
