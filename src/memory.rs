//! VM memory representation

#[cfg(not(feature = "std"))]
use alloc::Vec;

use bigint::{U256, M256};

use super::errors::NotSupportedError;
use super::Patch;

#[cfg(feature = "std")] use std::marker::PhantomData;
#[cfg(not(feature = "std"))] use core::marker::PhantomData;

/// Represent a memory in EVM. Read should always succeed. Write can
/// fall.
pub trait Memory {
    /// Check whether write on this index would result in an error. If
    /// this function returns Ok, then both `write` and `write_raw` on
    /// this index should succeed.
    fn check_write(&self, index: U256) -> Result<(), NotSupportedError>;
    /// Check whether write on the given index range would result in
    /// an error. If this function returns Ok, then both `write` and
    /// `write_raw` on the given index range should succeed.
    fn check_write_range(&self, start: U256, len: U256) -> Result<(), NotSupportedError>;

    /// Write value into the index.
    fn write(&mut self, index: U256, value: M256) -> Result<(), NotSupportedError>;
    /// Write only one byte value into the index.
    fn write_raw(&mut self, index: U256, value: u8) -> Result<(), NotSupportedError>;
    /// Read value from the index.
    fn read(&self, index: U256) -> M256;
    /// Read only one byte value from the index.
    fn read_raw(&self, index: U256) -> u8;
}

/// A sequencial memory. It uses Rust's `Vec` for internal
/// representation.
pub struct SeqMemory<P: Patch> {
    memory: Vec<u8>,
    _marker: PhantomData<P>,
}

impl<P: Patch> Default for SeqMemory<P> {
    fn default() -> SeqMemory<P> {
        SeqMemory {
            memory: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<P: Patch> SeqMemory<P> {
    /// Get the length of the current effective memory range.
    pub fn len(&self) -> usize {
        self.memory.len()
    }
}

impl<P: Patch> Memory for SeqMemory<P> {
    fn check_write(&self, index: U256) -> Result<(), NotSupportedError> {
        let end = index + 32.into();
        if end > U256::from(P::memory_limit()) {
            Err(NotSupportedError::MemoryIndexNotSupported)
        } else {
            Ok(())
        }
    }

    fn check_write_range(&self, start: U256, len: U256) -> Result<(), NotSupportedError> {
        if len == U256::zero() {
            return Ok(());
        }

        if start.saturating_add(len) > U256::from(P::memory_limit()) {
            Err(NotSupportedError::MemoryIndexNotSupported)
        } else {
            self.check_write(start + len - U256::from(1u64))
        }
    }

    fn write(&mut self, index: U256, value: M256) -> Result<(), NotSupportedError> {
        let end = M256::from(index) + 32.into();
        if end > M256::from(P::memory_limit()) {
            return Err(NotSupportedError::MemoryIndexNotSupported);
        }

        for i in 0..32 {
            self.write_raw(index + i.into(), value.index(i)).unwrap();
        }
        Ok(())
    }

    fn write_raw(&mut self, index: U256, value: u8) -> Result<(), NotSupportedError> {
        if index > U256::from(P::memory_limit()) {
            return Err(NotSupportedError::MemoryIndexNotSupported);
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
