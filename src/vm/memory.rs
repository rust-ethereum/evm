use utils::bigint::M256;
use super::{ExecutionResult, ExecutionError};

pub trait Memory {
    fn write(&mut self, index: M256, value: M256) -> ExecutionResult<()> {
        // Vector only deals with usize, so the maximum size is
        // actually smaller than 2^256
        let end = index + 32.into();

        let a: [u8; 32] = value.into();
        for i in 0..32 {
            self.write_raw(index + i.into(), a[i]).unwrap();
        }
        Ok(())
    }
    fn read(&self, index: M256) -> ExecutionResult<M256> {
        let end = index + 32.into();
        let mut a: [u8; 32] = [0u8; 32];

        for i in 0..32 {
            a[i] = self.read_raw(index + i.into()).unwrap();
        }
        Ok(a.into())
    }
    fn write_raw(&mut self, index: M256, value: u8) -> ExecutionResult<()>;
    fn read_raw(&self, index: M256) -> ExecutionResult<u8>;
}

pub struct SeqMemory {
    memory: Vec<u8>,
}

impl Default for SeqMemory {
    fn default() -> SeqMemory {
        SeqMemory {
            memory: Vec::new(),
        }
    }
}

impl Memory for SeqMemory {
    fn write(&mut self, index: M256, value: M256) -> ExecutionResult<()> {
        let end = index + 32.into();
        if end > M256::from(usize::max_value()) {
            return Err(ExecutionError::MemoryTooLarge);
        }

        let a: [u8; 32] = value.into();
        for i in 0..32 {
            self.write_raw(index + i.into(), a[i]);
        }
        Ok(())
    }

    fn write_raw(&mut self, index: M256, value: u8) -> ExecutionResult<()> {
        if index > M256::from(usize::max_value()) {
            return Err(ExecutionError::MemoryTooLarge);
        }

        let index: usize = index.into();

        if self.memory.len() <= index {
            self.memory.resize(index + 1, 0u8);
        }

        self.memory[index] = value;
        Ok(())
    }

    fn read_raw(&self, index: M256) -> ExecutionResult<u8> {
        if index > M256::from(usize::max_value()) {
            return Ok(0u8);
        }

        let index: usize = index.into();

        if self.memory.len() <= index {
            return Ok(0u8);
        }

        Ok(self.memory[index])
    }
}
