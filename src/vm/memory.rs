use utils::u256::U256;

pub trait Memory {
    fn write(&mut self, index: U256, value: U256);
    fn read(&mut self, index: U256) -> U256;
    fn active_len(&self) -> usize;
}

pub struct VectorMemory {
    memory: Vec<U256>,
}

impl VectorMemory {
    pub fn new() -> VectorMemory {
        VectorMemory {
            memory: Vec::new(),
        }
    }
}

impl Memory for VectorMemory {
    fn write(&mut self, index: U256, value: U256) {
        // Vector only deals with usize, so the maximum size is
        // actually smaller than 2^256
        let index_u64: u64 = index.into();
        let index = index_u64 as usize;

        if self.memory.len() <= index {
            self.memory.resize(index, 0.into());
        }

        self.memory[index] = value;
    }

    fn read(&mut self, index: U256) -> U256 {
        // This is required to be &mut self because a read might modify u_i

        let index_u64: u64 = index.into();
        let index = index_u64 as usize;

        if self.memory.len() <= index {
            self.memory.resize(index, 0.into());
        }

        match self.memory.get(index) {
            Some(&v) => v,
            None => 0.into()
        }
    }

    fn active_len(&self) -> usize {
        self.memory.len()
    }
}
