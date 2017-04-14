use utils::u256::U256;

pub trait Memory {
    fn write(&mut self, index: U256, value: U256);
    fn read(&mut self, index: U256) -> U256;
    fn active_len(&self) -> usize;
}

pub struct VectorMemory {
    memory: Vec<u8>,
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
        let start: usize = index.into();
        let end = start + 32;

        if self.memory.len() <= end - 1 {
            self.memory.resize(end - 1, 0u8);
        }

        let a: &[u8] = value.as_ref();
        for i in start..end {
            self.memory[i] = a[i - start];
        }
    }

    fn read(&mut self, index: U256) -> U256 {
        // This is required to be &mut self because a read might modify u_i
        let start: usize = index.into();
        let end = start + 32;

        if self.memory.len() <= end - 1 {
            self.memory.resize(end - 1, 0.into());
        }

        let mut a: [u8; 32] = [0u8; 32];
        for i in start..end {
            a[i - start] = self.memory[i];
        }

        a.into()
    }

    fn active_len(&self) -> usize {
        self.memory.len()
    }
}
