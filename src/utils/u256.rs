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

use ::std::convert::{From, Into, AsRef};
use ::std::ops::{Add, Sub, Not, Mul, Div, Shr, Shl, BitAnd, BitOr, BitXor};
use ::std::cmp::Ordering;

#[repr(C)]
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct U256([u64; 4]);

impl U256 {
    pub fn zero() -> U256 { 0.into() }
    pub fn one() -> U256 { 1.into() }

    pub fn max_value() -> U256 {
        !U256::zero()
    }

    pub fn min_value() -> U256 {
        U256::zero()
    }

    pub fn overflowing_add(self, other: U256) -> (U256, bool) {
        let U256(ref me) = self;
        let U256(ref you) = other;

        let mut ret = [0u64; 4];
        let mut carry = false;
        for i in 0..4 {
            let (v, o1) = me[i].overflowing_add(you[i]);
            let (v, o2) = v.overflowing_add(if carry { 1 } else { 0 });
            ret[i] = v;
            carry = o1 || o2;
        }

        (U256(ret), carry)
    }

    pub fn low_u32(&self) -> u32 {
        let &U256(ref arr) = self;
        arr[0] as u32
    }

    pub fn mul_u32(self, other: u32) -> U256 {
        let U256(ref arr) = self;
        let mut carry = [0u64; 4];
        let mut ret = [0u64; 4];
        for i in 0..4 {
            let upper = other as u64 * (arr[i] >> 32);
            let lower = other as u64 * (arr[i] & 0xFFFFFFFF);
            if i < 3 {
                carry[i + 1] += upper >> 32;
            }
            ret[i] = lower + (upper << 32);
        }
        U256(ret) + U256(carry)
    }

    pub fn bits(&self) -> usize {
        let &U256(ref arr) = self;
        for i in 1..4 {
            if arr[4 - i] > 0 { return (0x40 * (4 - i + 1)) - arr[4 - i].leading_zeros() as usize; }
        }
        0x40 - arr[0].leading_zeros() as usize
    }

    pub fn log2floor(&self) -> isize {
        assert!(*self != U256::zero());
        let mut l: isize = 256;
        for i in 0..4 {
            if self.0[3 - i] == 0u64 {
                l -= 64;
            } else {
                l -= self.0[3 - i].leading_zeros() as isize;
                return l;
            }
        }
        return l;
    }
}

impl AsRef<[u8]> for U256 {
    fn as_ref(&self) -> &[u8] {
        use std::mem::transmute;
        let r: &[u8; 32] = unsafe { transmute(&self.0) };
        r
    }
}

impl From<u64> for U256 {
    fn from(val: u64) -> U256 {
        U256([val, 0, 0, 0])
    }
}

impl Into<u64> for U256 {
    fn into(self) -> u64 {
        assert!(self.0[1] == 0 && self.0[2] == 0 && self.0[3] == 0);
        self.0[0]
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

impl From<i32> for U256 {
    fn from(val: i32) -> U256 {
        (val as u64).into()
    }
}

impl<'a> From<&'a [u8]> for U256 {
    fn from(val: &'a [u8]) -> U256 {
        assert!(val.len() <= 256 / 8);
        let mut u256 = U256::zero();

        for i in 0..val.len() {
            let rev = val.len() - 1 - i;
            let pos = rev / 8;
            u256.0[pos] += (val[i] as u64) << ((rev % 8) * 8);
        }

        u256
    }
}

impl From<[u8; 32]> for U256 {
    fn from(val: [u8; 32]) -> U256 {
        val.as_ref().into()
    }
}

impl Not for U256 {
    type Output = U256;

    fn not(self) -> U256 {
        let U256(ref arr) = self;
        let mut ret = [0u64; 4];
        for i in 0..4 {
            ret[i] = !arr[i];
        }
        U256(ret)
    }
}

impl Add for U256 {
    type Output = U256;

    fn add(self, other: U256) -> U256 {
        let (o, v) = self.overflowing_add(other);
        assert!(v == false);
        o
    }
}

impl Sub for U256 {
    type Output = U256;

    #[inline]
    fn sub(self, other: U256) -> U256 {
        let (o, v) = self.overflowing_add(!other);
        assert!(v == true);
        o + U256::one()
    }
}

impl Mul for U256 {
    type Output = U256;

    fn mul(self, other: U256) -> U256 {
        let mut me = self;
        // TODO: be more efficient about this
        for i in 0..(2 * 4) {
            me = (me + me.mul_u32((other >> (32 * i)).low_u32())) << (32 * i);
        }
        me
    }
}

impl Div for U256 {
    type Output = U256;

    fn div(self, other: U256) -> U256 {
        let mut sub_copy = self;
        let mut shift_copy = other;
        let mut ret = [0u64; 4];

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
                ret[shift / 64] |= 1 << (shift % 64);
                sub_copy = sub_copy - shift_copy;
            }
            shift_copy = shift_copy >> 1;
            if shift == 0 { break; }
            shift -= 1;
        }

        U256(ret)
    }
}

impl Ord for U256 {
    fn cmp(&self, other: &U256) -> Ordering {
	let &U256(ref me) = self;
	let &U256(ref you) = other;
	let mut i = 4;
	while i > 0 {
	    i -= 1;
	    if me[i] < you[i] { return Ordering::Less; }
	    if me[i] > you[i] { return Ordering::Greater; }
	}
	Ordering::Equal
    }
}

impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &U256) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Shl<usize> for U256 {
    type Output = U256;

    fn shl(self, shift: usize) -> U256 {
        let U256(ref original) = self;
        let mut ret = [0u64; 4];
        let word_shift = shift / 64;
        let bit_shift = shift % 64;
        for i in 0..4 {
            // Shift
            if bit_shift < 64 && i + word_shift < 4 {
                ret[i + word_shift] += original[i] << bit_shift;
            }
            // Carry
            if bit_shift > 0 && i + word_shift + 1 < 4 {
                ret[i + word_shift + 1] += original[i] >> (64 - bit_shift);
            }
        }
        U256(ret)
    }
}

impl Shr<usize> for U256 {
    type Output = U256;

    fn shr(self, shift: usize) -> U256 {
        let U256(ref original) = self;
        let mut ret = [0u64; 4];
        let word_shift = shift / 64;
        let bit_shift = shift % 64;
        for i in word_shift..4 {
            // Shift
            ret[i - word_shift] += original[i] >> bit_shift;
            // Carry
            if bit_shift > 0 && i < 4 - 1 {
                ret[i - word_shift] += original[i + 1] << (64 - bit_shift);
            }
        }
        U256(ret)
    }
}

impl BitAnd for U256 {
    type Output = U256;

    fn bitand(self, other: U256) -> U256 {
        let mut r: U256 = self;
        for i in 0..4 {
            r.0[i] = r.0[i] & other.0[i];
        }
        r
    }
}

impl BitOr for U256 {
    type Output = U256;

    fn bitor(self, other: U256) -> U256 {
        let mut r: U256 = self;
        for i in 0..4 {
            r.0[i] = r.0[i] | other.0[i];
        }
        r
    }
}

impl BitXor for U256 {
    type Output = U256;

    fn bitxor(self, other: U256) -> U256 {
        let mut r: U256 = self;
        for i in 0..4 {
            r.0[i] = r.0[i] ^ other.0[i];
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::U256;

    #[test]
    fn u256_add() {
        assert_eq!(
            U256([0xffffffffffffffffu64, 0u64, 0u64, 0u64]) +
            U256([0xffffffffffffffffu64, 0u64, 0u64, 0u64]),
            U256([0xfffffffffffffffeu64, 1u64, 0u64, 0u64])
        );
    }

    #[test]
    fn u256_overflowing_add() {
        assert_eq!(
            U256::max_value().overflowing_add(U256::one() + U256::one()).0,
            U256::one()
        );
    }

    #[test]
    fn u256_sub() {
        assert_eq!(
            U256([0xfffffffffffffffeu64, 1u64, 0u64, 0u64]) -
            U256([0xffffffffffffffffu64, 0u64, 0u64, 0u64]),
            U256([0xffffffffffffffffu64, 0u64, 0u64, 0u64])
        );
    }

    #[test]
    fn u256_bitand() {
        assert_eq!(
            U256::min_value() & U256::max_value(),
            U256::min_value()
        );
    }

    #[test]
    fn u256_bitor() {
        assert_eq!(
            U256::min_value() | U256::max_value(),
            U256::max_value()
        );
    }

    #[test]
    fn u256_not() {
        assert_eq!(
            !U256::min_value(),
            U256([0xffffffffffffffffu64, 0xffffffffffffffffu64,
                  0xffffffffffffffffu64, 0xffffffffffffffffu64])
        );
        assert_eq!(
            !U256::max_value(),
            U256([0u64, 0u64, 0u64, 0u64])
        );
    }

    #[test]
    fn u256_log2floor() {
        assert_eq!(
            U256([u64::max_value(), u64::max_value(), u64::max_value(), u64::max_value()])
                .log2floor(),
            256, "testing log2floor for max value");
        assert_eq!(
            U256([0x1u64, 0u64, 0u64, 0u64]).log2floor(),
            1, "testing log2floor for 1");
    }
}
