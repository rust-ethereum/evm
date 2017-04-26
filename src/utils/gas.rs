use std::ops::{Add, Sub, Not, Mul, Div, Shr, Shl, BitAnd, BitOr, BitXor, Rem};
use std::fmt;
use std::str::FromStr;
use std::cmp::Ordering;

use utils::bigint::{M256, U256};
use utils::{read_hex, ParseHexError};

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct Gas(U256);

impl Gas {
    pub fn zero() -> Gas { Gas(U256::zero()) }
    pub fn one() -> Gas { Gas(U256::one()) }
    pub fn max_value() -> Gas { Gas(U256::max_value()) }
    pub fn min_value() -> Gas { Gas(U256::min_value()) }
    pub fn bits(self) -> usize { self.0.bits() }
    pub fn log2floor(self) -> usize { self.0.log2floor() }
}

impl Default for Gas { fn default() -> Gas { Gas::zero() } }

impl FromStr for Gas {
    type Err = ParseHexError;

    fn from_str(s: &str) -> Result<Gas, ParseHexError> {
        U256::from_str(s).map(|s| Gas(s))
    }
}

impl From<bool> for Gas { fn from(val: bool) -> Gas { Gas(U256::from(val)) } }
impl From<u64> for Gas { fn from(val: u64) -> Gas { Gas(U256::from(val)) } }
impl Into<u64> for Gas { fn into(self) -> u64 { self.0.into() } }
impl From<usize> for Gas { fn from(val: usize) -> Gas { Gas(U256::from(val)) } }
impl Into<usize> for Gas { fn into(self) -> usize { self.0.into() } }
impl<'a> From<&'a [u8]> for Gas { fn from(val: &'a [u8]) -> Gas { Gas(U256::from(val)) } }
impl From<[u8; 32]> for Gas { fn from(val: [u8; 32]) -> Gas { Gas(U256::from(val)) } }
impl Into<[u8; 32]> for Gas { fn into(self) -> [u8; 32] { self.0.into() } }
impl Into<[u32; 8]> for Gas { fn into(self) -> [u32; 8] { self.0.into() } }
impl From<[u32; 8]> for Gas { fn from(val: [u32; 8]) -> Gas { Gas(U256::from(val)) } }
impl From<U256> for Gas { fn from(val: U256) -> Gas { Gas(val) } }
impl Into<U256> for Gas { fn into(self) -> U256 { self.0 } }
impl From<M256> for Gas { fn from(val: M256) -> Gas { Gas(val.into()) } }
impl Into<M256> for Gas { fn into(self) -> M256 { M256::from(self.0) } }
impl From<i32> for Gas { fn from(val: i32) -> Gas { (val as u64).into() } }

impl Ord for Gas { fn cmp(&self, other: &Gas) -> Ordering { self.0.cmp(&other.0) } }
impl PartialOrd for Gas {
    fn partial_cmp(&self, other: &Gas) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Add<Gas> for Gas {
    type Output = Gas;

    fn add(self, other: Gas) -> Gas {
        Gas(self.0.add(other.0))
    }
}

impl Sub<Gas> for Gas {
    type Output = Gas;

    fn sub(self, other: Gas) -> Gas {
        Gas(self.0.sub(other.0))
    }
}

impl Mul<Gas> for Gas {
    type Output = Gas;

    fn mul(self, other: Gas) -> Gas {
        Gas(self.0.mul(other.0))
    }
}

impl Div for Gas {
    type Output = Gas;

    fn div(self, other: Gas) -> Gas {
        Gas(self.0.div(other.0))
    }
}

impl Rem for Gas {
    type Output = Gas;

    fn rem(self, other: Gas) -> Gas {
        Gas(self.0.rem(other.0))
    }
}

impl Not for Gas {
    type Output = Gas;

    fn not(self) -> Gas {
        Gas(self.0.not())
    }
}

impl fmt::LowerHex for Gas {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl fmt::UpperHex for Gas {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}", self.0)
    }
}
