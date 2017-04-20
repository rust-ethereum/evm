use std::convert::{From, Into, AsRef};
use std::str::FromStr;
use std::ops::{Add, Sub, Not, Mul, Div, Shr, Shl, BitAnd, BitOr, BitXor, Rem};
use std::cmp::Ordering;

use super::{U256};
use utils::ParseHexError;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct M256(U256);

impl M256 {
    pub fn zero() -> M256 { M256(U256::zero()) }
    pub fn one() -> M256 { M256(U256::one()) }
    pub fn max_value() -> M256 { M256(U256::max_value()) }
    pub fn min_value() -> M256 { M256(U256::min_value()) }
    pub fn bits(self) -> usize { self.0.bits() }
    pub fn log2floor(self) -> usize { self.0.log2floor() }
}

impl Default for M256 { fn default() -> M256 { M256::zero() } }
impl AsRef<[u8]> for M256 { fn as_ref(&self) -> &[u8] { self.0.as_ref() } }

impl FromStr for M256 {
    type Err = ParseHexError;

    fn from_str(s: &str) -> Result<M256, ParseHexError> {
        U256::from_str(s).map(|s| M256(s))
    }
}

impl From<bool> for M256 { fn from(val: bool) -> M256 { M256(U256::from(val)) } }
impl From<u64> for M256 { fn from(val: u64) -> M256 { M256(U256::from(val)) } }
impl Into<u64> for M256 { fn into(self) -> u64 { self.0.into() } }
impl From<usize> for M256 { fn from(val: usize) -> M256 { M256(U256::from(val)) } }
impl Into<usize> for M256 { fn into(self) -> usize { self.0.into() } }
impl<'a> From<&'a [u8]> for M256 { fn from(val: &'a [u8]) -> M256 { M256(U256::from(val)) } }
impl From<[u8; 32]> for M256 { fn from(val: [u8; 32]) -> M256 { M256(U256::from(val)) } }
impl Into<[u32; 8]> for M256 { fn into(self) -> [u32; 8] { self.0.into() } }
impl From<[u32; 8]> for M256 { fn from(val: [u32; 8]) -> M256 { M256(U256::from(val)) } }
impl From<U256> for M256 { fn from(val: U256) -> M256 { M256(val) } }
impl Into<U256> for M256 { fn into(self) -> U256 { self.0 } }
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
        let (o, v) = self.0.overflowing_add(other.0);
        M256(o)
    }
}

impl Sub<M256> for M256 {
    type Output = M256;

    fn sub(self, other: M256) -> M256 {
        let (o, v) = self.0.underflowing_sub(other.0);
        M256(o)
    }
}

impl Mul<M256> for M256 {
    type Output = M256;

    fn mul(self, other: M256) -> M256 {
        let (o, v) = self.0.overflowing_mul(other.0);
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
