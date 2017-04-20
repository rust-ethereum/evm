use std::str::FromStr;

use utils::bigint::M256;
use utils::{read_hex, ParseHexError};

#[repr(C)]
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct H256([u8; 32]);

impl Default for H256 {
    fn default() -> H256 {
        H256([0u8; 32])
    }
}

impl Into<M256> for H256 {
    fn into(self) -> M256 {
        M256::from(self.0.as_ref())
    }
}

impl FromStr for H256 {
    type Err = ParseHexError;

    fn from_str(s: &str) -> Result<H256, ParseHexError> {
        read_hex(s).and_then(|v| {
            if v.len() > 32 {
                Err(ParseHexError::TooLong)
            } else if v.len() < 32 {
                Err(ParseHexError::TooShort)
            } else {
                let mut a = [0u8; 32];
                for i in 0..32 {
                    a[i] = v[i];
                }
                Ok(H256(a))
            }
        })
    }
}
