//! RLP (Recursive Length Prefix) is to encode arbitrarily nested arrays of binary data,
//! RLP is the main encoding method used to serialize objects in Ethereum.
//!
//! See [RLP spec](https://github.com/ethereumproject/wiki/wiki/RLP)

// Copied from https://github.com/ethereumproject/emerald-rs/blob/master/src/util/rlp.rs

/// The `WriteRLP` trait is used to specify functionality of serializing data to RLP bytes
pub trait WriteRLP {
    /// Writes itself as RLP bytes into specified buffer
    fn write_rlp(&self, buf: &mut Vec<u8>);
}

/// A list serializable to RLP
pub struct RLPList {
    tail: Vec<u8>,
}

impl RLPList {
    /// Start with provided vector
    pub fn from_slice<T: WriteRLP>(items: &[T]) -> RLPList {
        let mut start = RLPList { tail: Vec::new() };
        for i in items {
            start.push(i)
        }
        start
    }

    /// Add an item to the list
    pub fn push<T: WriteRLP + ?Sized>(&mut self, item: &T) {
        item.write_rlp(&mut self.tail);
    }
}

impl Default for RLPList {
    fn default() -> RLPList {
        RLPList { tail: Vec::new() }
    }
}

impl WriteRLP for [u8] {
    fn write_rlp(&self, buf: &mut Vec<u8>) {
        let len = self.len();
        if len <= 55 {
            // Otherwise, if a string is 0-55 bytes long, the RLP encoding consists of a single byte
            // with value 0x80 plus the length of the string followed by the string. The range of
            // the first byte is thus [0x80, 0xb7].
            buf.push(0x80 + len as u8);
            buf.extend_from_slice(self);
        } else {
            // If a string is more than 55 bytes long, the RLP encoding consists of a single byte
            // with value 0xb7 plus the length in bytes of the length of the string in binary form,
            // followed by the length of the string, followed by the string. For example, a
            // length-1024 string would be encoded as \xb9\x04\x00 followed by the string. The
            // range of the first byte is thus [0xb8, 0xbf].
            let len_bytes = bytes_count(len);
            buf.push(0xb7 + len_bytes);
            buf.extend_from_slice(&to_bytes(len as u64, len_bytes));
            buf.extend_from_slice(self);
        }
    }
}
