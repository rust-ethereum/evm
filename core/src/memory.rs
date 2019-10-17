/// A sequencial memory. It uses Rust's `Vec` for internal
/// representation.
pub struct Memory {
    data: Vec<u8>,
    limit: usize,
}

impl Memory {
    /// Get the length of the current effective memory range.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Return true if current effective memory range is zero
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
