// Copyright Ethereum Classic Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// Rust Bitcoin Library
// Written in 2014 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # Big unsigned integer types
//!
//! Implementation of a various large-but-fixed sized unsigned integer types.
//! The functions here are designed to be fast.
//!

// #![no_std]

use std::convert::{From, Into, AsRef};
use std::str::FromStr;
use std::ops::{Add, Sub, Not, Mul, Div, Shr, Shl, BitAnd, BitOr, BitXor, Rem};

use std::cmp::Ordering;
use std::fmt;

use super::{Sign, ParseHexError, read_hex};
use super::algorithms::{add2, mac3, from_signed, sub2_sign, big_digit};

pub const SIGN_BIT_MASK: U256 = U256([0b01111111111111111111111111111111u32,
                                      0xffffffffu32, 0xffffffffu32, 0xffffffffu32,
                                      0xffffffffu32, 0xffffffffu32, 0xffffffffu32, 0xffffffffu32]);

#[repr(C)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
/// Represents an unsigned 256-bit integer.
pub struct U256([u32; 8]);

impl U256 {
    /// Zero value of U256.
    pub fn zero() -> U256 { 0u64.into() }
    /// One value of U256.
    pub fn one() -> U256 { 1u64.into() }

    /// Maximum value of U256.
    pub fn max_value() -> U256 {
        !U256::zero()
    }
    /// Minimum value of U256.
    pub fn min_value() -> U256 {
        U256::zero()
    }

    /// Add two U256 with overflowing. The same as M256::add.
    pub fn overflowing_add(mut self, other: U256) -> (U256, bool) {
        let U256(ref mut a) = self;
        let U256(ref b) = other;

        let carry = add2(a, b);
        (U256(*a), if carry > 0 { true } else { false })
    }

    /// Substract two U256 with underflowing. The same as M256::sub.
    pub fn underflowing_sub(mut self, other: U256) -> (U256, bool) {
        let U256(ref mut a) = self;
        let U256(ref b) = other;

        let sign = sub2_sign(a, b);
        from_signed(sign, a);
        (U256(*a), if sign == Sign::Minus { true } else { false })
    }

    /// Multiply two U256 with overflowing. The same as M256::mul.
    pub fn overflowing_mul(mut self, other: U256) -> (U256, bool) {
        let mut ret = [0u32; 8];
        let U256(ref mut a) = self;
        let U256(ref b) = other;

        let mut carry = 0;

        for (i, bi) in b.iter().rev().enumerate() {
            carry = mac3(&mut ret[0..(8-i)], a, *bi);
        }

        (U256(ret), if carry > 0 { true } else { false })
    }

    /// Bits needed to represent this value.
    pub fn bits(&self) -> usize {
        let &U256(ref arr) = self;
        let mut current_bits = 0;
        for i in (0..8).rev() {
            if arr[i] == 0 {
                continue;
            }

            current_bits = (32 - arr[i].leading_zeros() as usize) + ((7 - i) * 32);
        }
        current_bits
    }

    /// Equals `floor(log2(*))`. This is always an integer.
    pub fn log2floor(&self) -> usize {
        assert!(*self != U256::zero());
        let mut l: usize = 256;
        for i in 0..8 {
            if self.0[i] == 0u32 {
                l -= 32;
            } else {
                l -= self.0[i].leading_zeros() as usize;
                if l == 0 {
                    return l
                } else {
                    return l-1;
                }
            }
        }
        return l;
    }
}

// Froms, Intos and Defaults

impl Default for U256 {
    fn default() -> U256 {
        U256::zero()
    }
}

impl FromStr for U256 {
    type Err = ParseHexError;

    fn from_str(s: &str) -> Result<U256, ParseHexError> {
        read_hex(s).map(|s| {
            U256::from(s.as_ref())
        })
    }
}

impl From<bool> for U256 {
    fn from(val: bool) -> U256 {
        if val {
            U256::one()
        } else {
            U256::zero()
        }
    }
}

impl From<u64> for U256 {
    fn from(val: u64) -> U256 {
        U256([0, 0, 0, 0, 0, 0, big_digit::get_hi(val), big_digit::get_lo(val)])
    }
}

impl Into<u64> for U256 {
    fn into(self) -> u64 {
        let p = self.0.iter().position(|s| *s != 0);
        assert!(p.is_none() || p.unwrap() >= 6);
        let lo = self.0[7] as u64;
        let hi = self.0[6] as u64;
        lo + (hi << 32)
    }
}

impl From<usize> for U256 {
    fn from(val: usize) -> U256 {
        (val as u64).into()
    }
}

impl Into<usize> for U256 {
    fn into(self) -> usize {
        let v64: u64 = self.into();
        v64 as usize
    }
}

impl<'a> From<&'a [u8]> for U256 {
    fn from(val: &'a [u8]) -> U256 {
        assert!(val.len() <= 256 / 8);

        let mut r = [0u8; 32];
        let reserved = 32 - val.len();

        for i in 0..val.len() {
            r[i + reserved] = val[i];
        }
        r.into()
    }
}

impl From<[u8; 32]> for U256 {
    fn from(val: [u8; 32]) -> U256 {
        let mut r = [0u32; 8];
        for i in 0..32 {
            let pos = i / 4;
            r[pos] += (val[i] as u32) << (8 * (3 - (i - (pos * 4))));
        }
        U256(r)
    }
}

impl Into<[u8; 32]> for U256 {
    fn into(self) -> [u8; 32] {
        let mut r = [0u8; 32];

        for i in 0..32 {
            let pos = i / 4;
            r[i] = (self.0[pos] >> (8 * (3 - (i - (pos * 4)))) & 0xFF) as u8;
        }

        r
    }
}

impl Into<[u32; 8]> for U256 {
    fn into(self) -> [u32; 8] {
        self.0
    }
}

impl From<[u32; 8]> for U256 {
    fn from(val: [u32; 8]) -> U256 {
        U256(val)
    }
}

// Ord

impl Ord for U256 {
    fn cmp(&self, other: &U256) -> Ordering {
	let &U256(ref me) = self;
	let &U256(ref you) = other;
	let mut i = 0;
	while i < 8 {
	    if me[i] < you[i] { return Ordering::Less; }
	    if me[i] > you[i] { return Ordering::Greater; }
            i += 1;
	}
	Ordering::Equal
    }
}

impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &U256) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl BitAnd<U256> for U256 {
    type Output = U256;

    fn bitand(self, other: U256) -> U256 {
        let mut r: U256 = self;
        for i in 0..8 {
            r.0[i] = r.0[i] & other.0[i];
        }
        r
    }
}

impl BitOr<U256> for U256 {
    type Output = U256;

    fn bitor(self, other: U256) -> U256 {
        let mut r: U256 = self;
        for i in 0..8 {
            r.0[i] = r.0[i] | other.0[i];
        }
        r
    }
}

impl BitXor<U256> for U256 {
    type Output = U256;

    fn bitxor(self, other: U256) -> U256 {
        let mut r: U256 = self;
        for i in 0..8 {
            r.0[i] = r.0[i] ^ other.0[i];
        }
        r
    }
}

impl Shl<usize> for U256 {
    type Output = U256;

    fn shl(self, shift: usize) -> U256 {
        let U256(ref original) = self;
        let mut ret = [0u32; 8];
        let word_shift = shift / 32;
        let bit_shift = shift % 32;
        for i in (0..8).rev() {
            // Shift
            if i >= word_shift {
                ret[i - word_shift] += original[i] << bit_shift;
            }
            // Carry
            if bit_shift > 0 && i >= word_shift + 1 {
                ret[i - word_shift - 1] += original[i] >> (32 - bit_shift);
            }
        }
        U256(ret)
    }
}

impl Shr<usize> for U256 {
    type Output = U256;

    fn shr(self, shift: usize) -> U256 {
        let U256(ref original) = self;
        let mut ret = [0u32; 8];
        let word_shift = shift / 32;
        let bit_shift = shift % 32;
        for i in (0..8).rev() {
            // Shift
            if i + word_shift < 8 {
                ret[i + word_shift] += original[i] >> bit_shift;
            }
            // Carry
            if bit_shift > 0 && i > 0 && i + word_shift < 8 {
                ret[i + word_shift] += original[i - 1] << (32 - bit_shift);
            }
        }
        U256(ret)
    }
}

impl Add<U256> for U256 {
    type Output = U256;

    fn add(self, other: U256) -> U256 {
        let (o, v) = self.overflowing_add(other);
        assert!(!v);
        o
    }
}

impl Sub<U256> for U256 {
    type Output = U256;

    fn sub(self, other: U256) -> U256 {
        let (o, v) = self.underflowing_sub(other);
        assert!(!v);
        o
    }
}

impl Mul<U256> for U256 {
    type Output = U256;

    fn mul(self, other: U256) -> U256 {
        let (o, v) = self.overflowing_mul(other);
        assert!(!v);
        o
    }
}

impl Div for U256 {
    type Output = U256;

    fn div(self, other: U256) -> U256 {
        let mut sub_copy = self;
        let mut shift_copy = other;
        let mut ret = [0u32; 8];

        let my_bits = self.bits();
        let your_bits = other.bits();

        // Check for division by 0
        assert!(your_bits != 0);

        // Early return in case we are dividing by a larger number than us
        if my_bits < your_bits {
            return U256(ret);
        }

        // Bitwise long division
        let mut shift = my_bits - your_bits;
        shift_copy = shift_copy << shift;
        loop {
            if sub_copy >= shift_copy {
                ret[7 - shift / 32] |= 1 << (shift % 32);
                sub_copy = sub_copy - shift_copy;
            }
            shift_copy = shift_copy >> 1;
            if shift == 0 { break; }
            shift -= 1;
        }

        U256(ret)
    }
}

impl Rem for U256 {
    type Output = U256;

    fn rem(self, other: U256) -> U256 {
        let d = self / other;
        self - (other * d)
    }
}

impl Not for U256 {
    type Output = U256;

    fn not(self) -> U256 {
        let U256(ref arr) = self;
        let mut ret = [0u32; 8];
        for i in 0..8 {
            ret[i] = !arr[i];
        }
        U256(ret)
    }
}

impl fmt::LowerHex for U256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..8 {
            write!(f, "{:08x}", self.0[i])?;
        }
        Ok(())
    }
}

impl fmt::UpperHex for U256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..8 {
            write!(f, "{:08X}", self.0[i])?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::U256;

    #[test]
    pub fn mul() {
        assert_eq!(U256([0, 0, 0, 0, 0, 0, 0, 2]) * U256([0, 0, 0, 0, 0, 0, 0, 3]),
                   U256([0, 0, 0, 0, 0, 0, 0, 6]));
        assert_eq!(U256([0x7FFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
                         0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF]) *
                   U256([0, 0, 0, 0, 0, 0, 0, 2]),
                   U256([0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
                         0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFE]));
    }

    #[test]
    pub fn div() {
        assert_eq!(U256([0, 0, 0, 0, 0, 0, 0, 3]) / U256([0, 0, 0, 0, 0, 0, 0, 2]),
                   U256::one());
        assert_eq!(U256([0, 0, 0, 0, 0, 0, 0, 1000000001]) / U256([0, 0, 0, 0, 0, 0, 0, 2]),
                   U256([0, 0, 0, 0, 0, 0, 0, 500000000]));
        assert_eq!(U256([0, 0, 0, 0, 0, 0, 0, 0xFFFFFFFD]) /
                   U256([0, 0, 0, 0, 0, 0, 0, 2]),
                   U256([0, 0, 0, 0, 0, 0, 0, 0x7FFFFFFE]));
        assert_eq!(U256([0, 0, 0, 0, 0, 0, 0xFFFFFFFF, 0xFFFFFFFD]) /
                   U256([0, 0, 0, 0, 0, 0, 0, 2]),
                   U256([0, 0, 0, 0, 0, 0, 0x7FFFFFFF, 0xFFFFFFFE]));
        assert_eq!(U256([0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
                         0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFD]) /
                   U256([0, 0, 0, 0, 0, 0, 0, 2]),
                   U256([0x7FFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
                         0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFE]));
    }
}
