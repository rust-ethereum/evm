//! Ethereum address
use std::str::FromStr;
use std::fmt;

use utils::bigint::M256;
use utils::{read_hex, ParseHexError};
use rlp::{Encodable, RlpStream};

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
/// Represents an Ethereum address. This address is 20 bytes long.
pub struct Address([u8; 20]);

impl Address {
    /// Bits needed to represent this value.
    pub fn bits(&self) -> usize {
        let u: M256 = self.clone().into();
        u.bits()
    }
}

impl Default for Address {
    fn default() -> Address {
        Address([0u8; 20])
    }
}

impl Encodable for Address {
    fn rlp_append(&self, s: &mut RlpStream) {
        let buffer: [u8; 20] = self.clone().into();
        s.encoder().encode_value(&buffer);
    }
}

impl Into<M256> for Address {
    fn into(self) -> M256 {
        M256::from(self.0.as_ref())
    }
}

impl From<M256> for Address {
    fn from(mut val: M256) -> Address {
        let mut i = 20;
        let mut a = [0u8; 20];

        while i != 0 {
            let u: u64 = (val & 0xFF.into()).into();
            a[i-1] = u as u8;

            i -= 1;
            val = val >> 8;
        }

        Address(a)
    }
}

impl Into<[u8; 20]> for Address {
    fn into(self) -> [u8; 20] {
        self.0
    }
}

impl FromStr for Address {
    type Err = ParseHexError;

    fn from_str(s: &str) -> Result<Address, ParseHexError> {
        read_hex(s).and_then(|v| {
            if v.len() > 20 {
                Err(ParseHexError::TooLong)
            } else if v.len() < 20 {
                Err(ParseHexError::TooShort)
            } else {
                let mut a = [0u8; 20];
                for i in 0..20 {
                    a[i] = v[i];
                }
                Ok(Address(a))
            }
        })
    }
}

impl fmt::LowerHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..20 {
            write!(f, "{:02x}", self.0[i])?;
        }
        Ok(())
    }
}

impl fmt::UpperHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..20 {
            write!(f, "{:02X}", self.0[i])?;
        }
        Ok(())
    }
}
