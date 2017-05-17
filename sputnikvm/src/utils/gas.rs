//! Ethereum gas
use std::ops::{Add, Sub, Mul, Div, Rem};
use std::fmt;
use std::str::FromStr;
use std::cmp::Ordering;

use utils::bigint::{M256, U512, U256};
use utils::ParseHexError;

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
/// Represent an Ethereum gas.
pub struct Gas(U512);

impl Gas {
    /// Zero gas.
    pub fn zero() -> Gas { Gas(U512::zero()) }
    /// Bits needed to represent this value.
    pub fn bits(self) -> usize { self.0.bits() }
}

impl Default for Gas { fn default() -> Gas { Gas::zero() } }

impl FromStr for Gas {
    type Err = ParseHexError;

    fn from_str(s: &str) -> Result<Gas, ParseHexError> {
        U256::from_str(s).map(|s| Gas::from(s))
    }
}

impl From<u64> for Gas { fn from(val: u64) -> Gas { Gas::from(U256::from(val)) } }
impl Into<u64> for Gas {
    fn into(self) -> u64 {
        let gas: U256 = self.into();
        gas.into()
    }
}
impl From<usize> for Gas { fn from(val: usize) -> Gas { Gas::from(U256::from(val)) } }
impl Into<U256> for Gas { fn into(self) -> U256 { self.0.into() } }
impl From<U256> for Gas { fn from(val: U256) -> Gas { Gas(U512::from(val)) } }
impl From<M256> for Gas {
    fn from(val: M256) -> Gas {
        let val: U256 = val.into();
        Gas::from(val)
    }
}
impl Into<M256> for Gas {
    fn into(self) -> M256 {
        let val: U256 = self.0.into();
        M256::from(val)
    }
}

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
