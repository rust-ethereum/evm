// From rustnum

use std::cmp::Ordering::{self, Less, Greater, Equal};
use super::Sign;

#[allow(non_snake_case)]
pub mod big_digit {
    /// A `BigDigit` is a `BigUint`'s composing element.
    pub type BigDigit = u32;

    /// A `DoubleBigDigit` is the internal type used to do the computations.  Its
    /// size is the double of the size of `BigDigit`.
    pub type DoubleBigDigit = u64;

    pub const ZERO_BIG_DIGIT: BigDigit = 0;

    // `DoubleBigDigit` size dependent
    pub const BITS: usize = 32;

    pub const BASE: DoubleBigDigit = 1 << BITS;
    const LO_MASK: DoubleBigDigit = (-1i32 as DoubleBigDigit) >> BITS;

    #[inline]
    pub fn get_hi(n: DoubleBigDigit) -> BigDigit {
        (n >> BITS) as BigDigit
    }
    #[inline]
    pub fn get_lo(n: DoubleBigDigit) -> BigDigit {
        (n & LO_MASK) as BigDigit
    }

    /// Split one `DoubleBigDigit` into two `BigDigit`s.
    #[inline]
    pub fn from_doublebigdigit(n: DoubleBigDigit) -> (BigDigit, BigDigit) {
        (get_hi(n), get_lo(n))
    }

    /// Join two `BigDigit`s into one `DoubleBigDigit`
    #[inline]
    pub fn to_doublebigdigit(hi: BigDigit, lo: BigDigit) -> DoubleBigDigit {
        (lo as DoubleBigDigit) | ((hi as DoubleBigDigit) << BITS)
    }
}

use self::big_digit::{BigDigit, DoubleBigDigit};

// Generic functions for add/subtract/multiply with carry/borrow:

// Add with carry:
#[inline]
fn adc(a: BigDigit, b: BigDigit, carry: &mut BigDigit) -> BigDigit {
    let (hi, lo) = big_digit::from_doublebigdigit((a as DoubleBigDigit) + (b as DoubleBigDigit) +
                                                  (*carry as DoubleBigDigit));

    *carry = hi;
    lo
}

// Subtract with borrow:
#[inline]
fn sbb(a: BigDigit, b: BigDigit, borrow: &mut BigDigit) -> BigDigit {
    let (hi, lo) = big_digit::from_doublebigdigit(big_digit::BASE + (a as DoubleBigDigit) -
                                                  (b as DoubleBigDigit) -
                                                  (*borrow as DoubleBigDigit));
    // hi * (base) + lo == 1*(base) + ai - bi - borrow
    // => ai - bi - borrow < 0 <=> hi == 0
    *borrow = (hi == 0) as BigDigit;
    lo
}

#[inline]
pub fn mac_with_carry(a: BigDigit, b: BigDigit, c: BigDigit, carry: &mut BigDigit) -> BigDigit {
    let (hi, lo) = big_digit::from_doublebigdigit((a as DoubleBigDigit) +
                                                  (b as DoubleBigDigit) * (c as DoubleBigDigit) +
                                                  (*carry as DoubleBigDigit));
    *carry = hi;
    lo
}

pub fn inc(a: &mut [BigDigit]) -> BigDigit {
    let mut added = false;
    let mut carry = 0;

    for a in a.iter_mut().rev() {
        let a: &mut BigDigit = a;
        if !added {
            *a = adc(*a, 1, &mut carry);
            added = true;
        }
    }

    carry
}

pub fn add2(a: &mut [BigDigit], b: &[BigDigit]) -> BigDigit {
    debug_assert!(a.len() == b.len());

    let mut carry = 0;

    for (a, b) in a.iter_mut().zip(b.iter()).rev() {
        let (a, b): (&mut BigDigit, &BigDigit) = (a, b);
        *a = adc(*a, *b, &mut carry);
    }

    carry
}

pub fn sub2(a: &mut [BigDigit], b: &[BigDigit]) -> BigDigit {
    debug_assert!(a.len() == b.len());

    let mut borrow: BigDigit = 0;

    for (a, b) in a.iter_mut().zip(b.iter()).rev() {
        let (a, b): (&mut BigDigit, &BigDigit) = (a, b);
        *a = sbb(*a, *b, &mut borrow);
    }

    borrow
}

pub fn sub2_rev(a: &[BigDigit], b: &mut [BigDigit]) -> BigDigit {
    debug_assert!(b.len() == a.len());

    let mut borrow: BigDigit = 0;

    for (a, b) in a.iter().zip(b.iter_mut()).rev() {
        let (a, b): (&BigDigit, &mut BigDigit) = (a, b);
        *b = sbb(*a, *b, &mut borrow);
    }

    borrow
}

pub fn sub2_sign(a: &mut [BigDigit], b: &[BigDigit]) -> Sign {
    match cmp_slice(a, b) {
        Greater => {
            sub2(a, b);
            Sign::Plus
        }
        Less => {
            sub2_rev(b, a);
            Sign::Minus
        }
        _ => {
            sub2(a, b);
            Sign::NoSign
        },
    }
}

/// Three argument multiply accumulate:
/// acc += b * c
pub fn mac3(acc: &mut [BigDigit], b: &[BigDigit], c: BigDigit) -> BigDigit {
    if c == 0 {
        return 0;
    }

    let mut b_iter = b.iter().rev();
    let mut carry = 0;

    for ai in acc.iter_mut().rev() {
        if let Some(bi) = b_iter.next() {
            *ai = mac_with_carry(*ai, *bi, c, &mut carry);
        } else if carry != 0 {
            *ai = mac_with_carry(*ai, 0, c, &mut carry);
        } else {
            break;
        }
    }

    carry
}

pub fn cmp_slice(a: &[BigDigit], b: &[BigDigit]) -> Ordering {
    debug_assert!(a.len() == b.len());

    for (&ai, &bi) in a.iter().zip(b) {
        if ai < bi {
            return Less;
        }
        if ai > bi {
            return Greater;
        }
    }
    return Equal;
}
