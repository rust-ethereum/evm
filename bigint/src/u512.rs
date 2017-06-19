//! Unsigned 512-bit integer

use std::convert::{From, Into};
use std::ops::{Add, Sub, Mul, Div, Shr, Shl, BitAnd, Rem};
use std::cmp::Ordering;
use std::fmt;

use super::{U256, Sign};
use super::algorithms::{add2, mac3, sub2_sign};

#[repr(C)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
/// Represents an unsigned 512-bit integer.
pub struct U512([u32; 16]);

impl U512 {
    /// Bits needed to represent this value.
    pub fn bits(&self) -> usize {
        let &U512(ref arr) = self;
        let mut current_bits = 0;
        for i in (0..16).rev() {
            if arr[i] == 0 {
                continue;
            }

            current_bits = (32 - arr[i].leading_zeros() as usize) + ((15 - i) * 32);
        }
        current_bits
    }

    /// Zero value of U512.
    pub fn zero() -> U512 {
        U512([0u32; 16])
    }
}

impl From<U256> for U512 {
    fn from(val: U256) -> U512 {
        let mut ret = [0u32; 16];
        let val: [u32; 8] = val.into();
        for i in 0..8 {
            ret[8 + i] = val[i];
        }
        U512(ret)
    }
}

impl Into<U256> for U512 {
    fn into(self) -> U256 {
        assert!(self.0[0..8].iter().all(|s| *s == 0));
        let mut ret = [0u32; 8];
        for i in 0..8 {
            ret[i] = self.0[8 + i];
        }
        ret.into()
    }
}

impl Ord for U512 {
    fn cmp(&self, other: &U512) -> Ordering {
	let &U512(ref me) = self;
	let &U512(ref you) = other;
	let mut i = 0;
	while i < 16 {
	    if me[i] < you[i] { return Ordering::Less; }
	    if me[i] > you[i] { return Ordering::Greater; }
            i += 1;
	}
	Ordering::Equal
    }
}

impl PartialOrd for U512 {
    fn partial_cmp(&self, other: &U512) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl BitAnd<U512> for U512 {
    type Output = U512;

    fn bitand(self, other: U512) -> U512 {
        let mut r: U512 = self;
        for i in 0..16 {
            r.0[i] = r.0[i] & other.0[i];
        }
        r
    }
}

impl Shl<usize> for U512 {
    type Output = U512;

    fn shl(self, shift: usize) -> U512 {
        let U512(ref original) = self;
        let mut ret = [0u32; 16];
        let word_shift = shift / 32;
        let bit_shift = shift % 32;
        for i in (0..16).rev() {
            // Shift
            if i >= word_shift {
                ret[i - word_shift] += original[i] << bit_shift;
            }
            // Carry
            if bit_shift > 0 && i >= word_shift + 1 {
                ret[i - word_shift - 1] += original[i] >> (32 - bit_shift);
            }
        }
        U512(ret)
    }
}

impl Shr<usize> for U512 {
    type Output = U512;

    fn shr(self, shift: usize) -> U512 {
        let U512(ref original) = self;
        let mut ret = [0u32; 16];
        let word_shift = shift / 32;
        let bit_shift = shift % 32;
        for i in (0..16).rev() {
            // Shift
            if i + word_shift < 16 {
                ret[i + word_shift] += original[i] >> bit_shift;
            }
            // Carry
            if bit_shift > 0 && i > 0 && i + word_shift < 16 {
                ret[i + word_shift] += original[i - 1] << (32 - bit_shift);
            }
        }
        U512(ret)
    }
}

impl Add for U512 {
    type Output = U512;

    fn add(mut self, other: U512) -> U512 {
        let U512(ref mut a) = self;
        let U512(ref b) = other;

        let carry = add2(a, b);
        assert!(carry == 0);
        U512(*a)
    }
}

impl Sub for U512 {
    type Output = U512;

    fn sub(mut self, other: U512) -> U512 {
        let U512(ref mut a) = self;
        let U512(ref b) = other;

        let sign = sub2_sign(a, b);
        assert!(sign != Sign::Minus);
        U512(*a)
    }
}

impl Mul for U512 {
    type Output = U512;

    fn mul(mut self, other: U512) -> U512 {
        let mut ret = [0u32; 16];
        let U512(ref mut a) = self;
        let U512(ref b) = other;

        let mut carry = 0;

        for (i, bi) in b.iter().rev().enumerate() {
            carry = mac3(&mut ret[0..(16-i)], a, *bi);
        }

        assert!(carry == 0);
        U512(ret)
    }
}

impl Div for U512 {
    type Output = U512;

    fn div(self, other: U512) -> U512 {
        let mut sub_copy = self;
        let mut shift_copy = other;
        let mut ret = [0u32; 16];

        let my_bits = self.bits();
        let your_bits = other.bits();

        // Check for division by 0
        assert!(your_bits != 0);

        // Early return in case we are dividing by a larger number than us
        if my_bits < your_bits {
            return U512(ret);
        }

        // Bitwise long division
        let mut shift = my_bits - your_bits;
        shift_copy = shift_copy << shift;
        loop {
            if sub_copy >= shift_copy {
                ret[15 - shift / 32] |= 1 << (shift % 32);
                sub_copy = sub_copy - shift_copy;
            }
            shift_copy = shift_copy >> 1;
            if shift == 0 { break; }
            shift -= 1;
        }

        U512(ret)
    }
}

impl Rem for U512 {
    type Output = U512;

    fn rem(self, other: U512) -> U512 {
        let d = self / other;
        self - (other * d)
    }
}

impl fmt::LowerHex for U512 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..16 {
            write!(f, "{:08x}", self.0[i])?;
        }
        Ok(())
    }
}

impl fmt::UpperHex for U512 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..16 {
            write!(f, "{:08X}", self.0[i])?;
        }
        Ok(())
    }
}
