//! VM memory representation

use bigint::{U256, M256};

use super::errors::MemoryError;

/// Represent a memory in EVM. Read should always succeed. Write can
/// fall.
pub trait Memory {
    /// Check whether write on this index would result in an error. If
    /// this function returns Ok, then both `write` and `write_raw` on
    /// this index should succeed.
    fn check_write(&self, index: U256) -> Result<(), MemoryError>;
    /// Check whether write on the given index range would result in
    /// an error. If this function returns Ok, then both `write` and
    /// `write_raw` on the given index range should succeed.
    fn check_write_range(&self, start: U256, len: U256) -> Result<(), MemoryError>;

    /// Write value into the index.
    fn write(&mut self, index: U256, value: M256) -> Result<(), MemoryError>;
    /// Write only one byte value into the index.
    fn write_raw(&mut self, index: U256, value: u8) -> Result<(), MemoryError>;
    /// Read value from the index.
    fn read(&self, index: U256) -> M256;
    /// Read only one byte value from the index.
    fn read_raw(&self, index: U256) -> u8;
}

/// A sequencial memory. It uses Rust's `Vec` for internal
/// representation.
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

impl SeqMemory {
    pub fn len(&self) -> usize {
        self.memory.len()
    }
}

impl Memory for SeqMemory {
    fn check_write(&self, index: U256) -> Result<(), MemoryError> {
        let end = index + 32.into();
        if end > U256::from(usize::max_value()) {
            Err(MemoryError::IndexNotSupported)
        } else {
            Ok(())
        }
    }

    fn check_write_range(&self, start: U256, len: U256) -> Result<(), MemoryError> {
        if len == U256::zero() {
            return Ok(());
        }

        if M256::from(start) + M256::from(len) < M256::from(start) {
            Err(MemoryError::IndexNotSupported)
        } else {
            self.check_write(start + len - U256::from(1u64))
        }
    }

    fn write(&mut self, index: U256, value: M256) -> Result<(), MemoryError> {
        let end = M256::from(index) + 32.into();
        if end > M256::from(usize::max_value()) {
            return Err(MemoryError::IndexNotSupported);
        }

        for i in 0..32 {
            self.write_raw(index + i.into(), value.index(i)).unwrap();
        }
        Ok(())
    }

    fn write_raw(&mut self, index: U256, value: u8) -> Result<(), MemoryError> {
        if index > U256::from(usize::max_value()) {
            return Err(MemoryError::IndexNotSupported);
        }

        let index: usize = index.as_usize();

        if self.memory.len() <= index {
            self.memory.resize(index + 1, 0u8);
        }

        self.memory[index] = value;
        Ok(())
    }

    fn read(&self, index: U256) -> M256 {
        let mut a: [u8; 32] = [0u8; 32];

        for i in 0..32 {
            a[i] = self.read_raw(index + i.into());
        }
        a.as_ref().into()
    }

    fn read_raw(&self, index: U256) -> u8 {
        if index > U256::from(usize::max_value()) {
            return 0u8;
        }

        let index: usize = index.as_usize();

        if self.memory.len() <= index {
            return 0u8;
        }

        self.memory[index]
    }
}
