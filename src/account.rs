use utils::u256::U256;
use utils::address::Address;

pub trait Storage { // A word-addressable word array, similar to memory, and is not volatile.
    fn write(&mut self, index: U256, value: U256);
    fn read(&self, index: U256) -> U256;
}

pub struct VectorStorage {
    storage: Vec<U256>,
}

impl VectorStorage {
    pub fn new() -> VectorStorage {
        VectorStorage {
            storage: Vec::new(),
        }
    }

    pub fn with_storage(storage: &[U256]) -> VectorStorage {
        VectorStorage {
            storage: storage.into(),
        }
    }
}

impl AsRef<[U256]> for VectorStorage {
    fn as_ref(&self) -> &[U256] {
        self.storage.as_ref()
    }
}

impl Storage for VectorStorage {
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
