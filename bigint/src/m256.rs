//! Unsigned modulo 256-bit integer

use std::convert::{From, Into};
use std::str::FromStr;
use std::ops::{Add, Sub, Not, Mul, Div, Shr, Shl, BitAnd, BitOr, BitXor, Rem};
use std::cmp::Ordering;
use std::fmt;

use super::{ParseHexError, U512, U256};
use rlp::{Encodable, RlpStream};

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
/// Represent an unsigned modulo 256-bit integer
pub struct M256(U256);

impl M256 {
    /// Zero value of M256,
    pub fn zero() -> M256 { M256(U256::zero()) }
    /// One value of M256,
    pub fn one() -> M256 { M256(U256::one()) }
    /// Maximum value of M256,
    pub fn max_value() -> M256 { M256(U256::max_value()) }
    /// Minimum value of M256,
    pub fn min_value() -> M256 { M256(U256::min_value()) }
    /// Bits required to represent this value.
    pub fn bits(self) -> usize { self.0.bits() }
    /// Equals `floor(log2(*))`. This is always an integer.
    pub fn log2floor(self) -> usize { self.0.log2floor() }
}

impl Default for M256 { fn default() -> M256 { M256::zero() } }

impl FromStr for M256 {
    type Err = ParseHexError;

    fn from_str(s: &str) -> Result<M256, ParseHexError> {
        U256::from_str(s).map(|s| M256(s))
    }
}

impl Encodable for M256 {
    fn rlp_append(&self, s: &mut RlpStream) {
        let leading_empty_bytes = 32 - (self.bits() + 7) / 8;
        let buffer: [u8; 32] = self.clone().into();
        s.encoder().encode_value(&buffer[leading_empty_bytes..]);
    }
}

impl From<bool> for M256 { fn from(val: bool) -> M256 { M256(U256::from(val)) } }
impl From<u64> for M256 { fn from(val: u64) -> M256 { M256(U256::from(val)) } }
impl Into<u64> for M256 { fn into(self) -> u64 { self.0.into() } }
impl From<usize> for M256 { fn from(val: usize) -> M256 { M256(U256::from(val)) } }
impl Into<usize> for M256 { fn into(self) -> usize { self.0.into() } }
impl<'a> From<&'a [u8]> for M256 { fn from(val: &'a [u8]) -> M256 { M256(U256::from(val)) } }
impl From<[u8; 32]> for M256 { fn from(val: [u8; 32]) -> M256 { M256(U256::from(val)) } }
impl Into<[u8; 32]> for M256 { fn into(self) -> [u8; 32] { self.0.into() } }
impl Into<[u32; 8]> for M256 { fn into(self) -> [u32; 8] { self.0.into() } }
impl From<[u32; 8]> for M256 { fn from(val: [u32; 8]) -> M256 { M256(U256::from(val)) } }
impl From<U256> for M256 { fn from(val: U256) -> M256 { M256(val) } }
impl Into<U256> for M256 { fn into(self) -> U256 { self.0 } }
impl From<U512> for M256 { fn from(val: U512) -> M256 { M256(val.into()) } }
impl Into<U512> for M256 { fn into(self) -> U512 { self.0.into() } }
impl From<i32> for M256 { fn from(val: i32) -> M256 { (val as u64).into() } }

impl Ord for M256 { fn cmp(&self, other: &M256) -> Ordering { self.0.cmp(&other.0) } }
impl PartialOrd for M256 {
    fn partial_cmp(&self, other: &M256) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl BitAnd<M256> for M256 {
    type Output = M256;

    fn bitand(self, other: M256) -> M256 {
        M256(self.0.bitand(other.0))
    }
}

impl BitOr<M256> for M256 {
    type Output = M256;

    fn bitor(self, other: M256) -> M256 {
        M256(self.0.bitor(other.0))
    }
}

impl BitXor<M256> for M256 {
    type Output = M256;

    fn bitxor(self, other: M256) -> M256 {
        M256(self.0.bitxor(other.0))
    }
}

impl Shl<usize> for M256 {
    type Output = M256;

    fn shl(self, shift: usize) -> M256 {
        M256(self.0.shl(shift))
    }
}

impl Shr<usize> for M256 {
    type Output = M256;

    fn shr(self, shift: usize) -> M256 {
        M256(self.0.shr(shift))
    }
}

impl Add<M256> for M256 {
    type Output = M256;

    fn add(self, other: M256) -> M256 {
        let (o, _) = self.0.overflowing_add(other.0);
        M256(o)
    }
}

impl Sub<M256> for M256 {
    type Output = M256;

    fn sub(self, other: M256) -> M256 {
        if self.0 >= other.0 {
            M256(self.0 - other.0)
        } else {
            M256(U256::max_value() - other.0 + self.0 + U256::from(1u64))
        }
    }
}

impl Mul<M256> for M256 {
    type Output = M256;

    fn mul(self, other: M256) -> M256 {
        let (o, _) = self.0.overflowing_mul(other.0);
        M256(o)
    }
}

impl Div for M256 {
    type Output = M256;

    fn div(self, other: M256) -> M256 {
        if other == M256::zero() {
            M256::zero()
        } else {
            M256(self.0.div(other.0))
        }
    }
}

impl Rem for M256 {
    type Output = M256;

    fn rem(self, other: M256) -> M256 {
        if other == M256::zero() {
            M256::zero()
        } else {
            M256(self.0.rem(other.0))
        }
    }
}

impl Not for M256 {
    type Output = M256;

    fn not(self) -> M256 {
        M256(self.0.not())
    }
}

impl fmt::LowerHex for M256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl fmt::UpperHex for M256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::M256;
    use std::str::FromStr;

    #[test]
    pub fn sub() {
        assert_eq!(M256::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap() - M256::from_str("0000000000000000000000000000000100000000000000000000000000000000").unwrap(), M256::from_str("ffffffffffffffffffffffffffffffff00000000000000000000000000000000").unwrap());
    }
}
