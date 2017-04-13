use utils::u256::U256;
use utils::address::Address;

pub trait Account {
    type Sto: Storage;

    fn address(&self) -> Address;
    fn balance(&self) -> U256;
    fn storage(&self) -> &Sto;
    fn storage_mut(&mut self) -> &mut Sto;
    fn code(&self) -> Option<&[u8]>;
}

pub struct FakeAccount {
    address: Address,
    nonce: usize,
    balance: U256,
    storage: FakeStorage,
    code: Option<Vec<u8>>,
}

impl FakeAccount {
    pub fn new(address: Address) -> FakeAccount {
        FakeAccount::with_code(address, None);
    }

    pub fn with_code(address: Address, code: Option<&[u8]>) -> FakeAccount {
        FakeAccount {
            address: Address,
            nounce: 0,
            balance: U256::zero(),
            storage: FakeStorage::new(),
            code: code.map(|s| s.into())
        }
    }
}

impl Default for FakeAccount {
    fn default() -> FakeAccount {
        FakeAccount::new(Address::default())
    }
}

impl Account for FakeAccount {
    type Storage = FakeStorage;

    fn address(&self) -> Address {
        self.address
    }

    fn balance(&self) -> U256 {
        self.balance
    }

    fn storage(&self) -> &Storage {
        &self.storage
    }

    fn storage_mut(&self) -> &mut Storage {
        &mut self.storage
    }

    fn code(&self) -> Option<&[u8]> {
        self.code.map(|s| s.into())
    }
}

pub trait Storage { // A word-addressable word array, similar to memory, and is not volatile.
    fn write(&mut self, index: U256, value: U256);
    fn read(&self, index: U256) -> U256;
}

pub struct FakeStorage {
    storage: Vec<U256>,
}

impl FakeStorage {
    pub fn new() -> FakeStorage {
        FakeStorage {
            storage: Vec::new(),
        }
    }
}

impl Storage for FakeStorage {
    fn write(&mut self, index: U256, value: U256) {
        let index: usize = index.into();

        if self.storage.len() <= index {
            self.storage.resize(index, 0.into());
        }

        self.storage[index] = value;
    }

    fn read(&self, index: U256) -> U256 {
        let index: usize = index.into();

        match self.storage.get(index) {
            Some(&v) => v,
            None => U256::zero()
        }
    }
}
