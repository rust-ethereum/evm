use utils::u256::U256;

pub trait Memory {
    fn write(&mut self, index: U256, value: U256) {
        // Vector only deals with usize, so the maximum size is
        // actually smaller than 2^256
        let end = index + 32.into();

        let a: &[u8] = value.as_ref();
        for i in 0..32 {
            self.write_raw(index + i.into(), a[i]);
        }
    }
    fn read(&mut self, index: U256) -> U256 {
        let end = index + 32.into();
        let mut a: [u8; 32] = [0u8; 32];

        for i in 0..32 {
            a[i] = self.read_raw(index + i.into())
        }
        a.into()
    }
    fn write_raw(&mut self, index: U256, value: u8);
    fn read_raw(&mut self, index: U256) -> u8;
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

impl AsRef<[u8]> for VectorMemory {
    fn as_ref(&self) -> &[u8] {
        self.memory.as_ref()
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

    fn write_raw(&mut self, index: U256, value: u8) {
        let index: usize = index.into();

        if self.memory.len() <= index {
            self.memory.resize(index, 0u8);
        }

        self.memory[index] = value;
    }

    fn read(&mut self, index: U256) -> U256 {
        // This is required to be &mut self because a read might modify u_i
        let start: usize = index.into();
        let end = start + 32;

        if self.memory.len() <= end - 1 {
            self.memory.resize(end - 1, 0u8);
        }

        let mut a: [u8; 32] = [0u8; 32];
        for i in start..end {
            a[i - start] = self.memory[i];
        }

        a.into()
    }

    fn read_raw(&mut self, index: U256) -> u8 {
        let index: usize = index.into();

        if self.memory.len() <= index {
            self.memory.resize(index, 0u8);
        }

        self.memory[index]
    }

    fn active_len(&self) -> usize {
        self.memory.len()
    }
}
