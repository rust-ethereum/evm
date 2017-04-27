use utils::bigint::M256;

pub trait Memory {
    fn write(&mut self, index: M256, value: M256) {
        // Vector only deals with usize, so the maximum size is
        // actually smaller than 2^256
        let end = index + 32.into();

        let a: [u8; 32] = value.into();
        for i in 0..32 {
            self.write_raw(index + i.into(), a[i]);
        }
    }
    fn read(&self, index: M256) -> M256 {
        let end = index + 32.into();
        let mut a: [u8; 32] = [0u8; 32];

        for i in 0..32 {
            a[i] = self.read_raw(index + i.into())
        }
        a.into()
    }
    fn write_raw(&mut self, index: M256, value: u8);
    fn read_raw(&self, index: M256) -> u8;
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
    fn write_raw(&mut self, index: M256, value: u8) {
        let index: usize = index.into();

        if self.memory.len() <= index {
            self.memory.resize(index + 1, 0u8);
        }

        self.memory[index] = value;
    }

    fn read_raw(&self, index: M256) -> u8 {
        let index: usize = index.into();

        if self.memory.len() <= index {
            return 0u8;
        }

        self.memory[index]
    }
}
