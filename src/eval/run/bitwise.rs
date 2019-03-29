//! Bitwise instructions

use bigint::{Sign, M256, MI256};

use super::State;
use crate::{Memory, Patch};

pub fn iszero<M: Memory, P: Patch>(state: &mut State<M, P>) {
    pop!(state, op1);

    if op1 == M256::zero() {
        push!(state, M256::from(1u64));
    } else {
        push!(state, M256::zero());
    }
}

pub fn not<M: Memory, P: Patch>(state: &mut State<M, P>) {
    pop!(state, op1);
    push!(state, !op1);
}

pub fn byte<M: Memory, P: Patch>(state: &mut State<M, P>) {
    pop!(state, op1, op2);

    let mut ret = M256::zero();

    for i in 0..256 {
        if i < 8 && op1 < 32.into() {
            let o: usize = op1.as_usize();
            let t = 255 - (7 - i + 8 * o);
            let bit_mask = M256::one() << t;
            let value = (op2 & bit_mask) >> t;
            ret = ret + (value << i);
        }
    }

    push!(state, ret);
}

pub fn shl<M: Memory, P: Patch>(state: &mut State<M, P>) {
    pop!(state, shift, value);

    let result = if value == M256::zero() || shift >= M256::from(256) {
        M256::zero()
    } else {
        let shift: u64 = shift.into();
        value << shift as usize
    };

    push!(state, result);
}

pub fn shr<M: Memory, P: Patch>(state: &mut State<M, P>) {
    pop!(state, shift, value);

    let result = if value == M256::zero() || shift >= M256::from(256) {
        M256::zero()
    } else {
        let shift: u64 = shift.into();
        value >> shift as usize
    };

    push!(state, result);
}

pub fn sar<M: Memory, P: Patch>(state: &mut State<M, P>) {
    pop!(state, shift, value);
    let value = MI256::from(value);

    let result = if value == MI256::zero() || shift >= M256::from(256) {
        let MI256(sign, _) = value;
        match sign {
            // value is 0 or >=1, pushing 0
            Sign::Plus | Sign::NoSign => M256::zero(),
            // value is <0, pushing -1
            Sign::Minus => MI256(Sign::Minus, M256::one()).into(),
        }
    } else {
        let shift: u64 = shift.into();

        match value.0 {
            Sign::Plus | Sign::NoSign => value.1 >> shift as usize,
            Sign::Minus => {
                let shifted = ((value.1 - M256::one()) >> shift as usize) + M256::one();
                MI256(Sign::Minus, shifted).into()
            }
        }
    };

    push!(state, result);
}
