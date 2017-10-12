//! Arithmetic instructions

use bigint::{M256, U512};

use ::Memory;
use super::State;
use patch::Patch;

pub fn addmod<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, op1: U512, op2: U512, op3: U512);

    if op3 == U512::zero() {
        push!(state, M256::zero());
    } else {
        let v = (op1 + op2) % op3;
        push!(state, v.into());
    }
}

pub fn mulmod<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, op1: U512, op2: U512, op3: U512);

    if op3 == U512::zero() {
        push!(state, M256::zero());
    } else {
        let v = (op1 * op2) % op3;
        push!(state, v.into());
    }
}


pub fn exp<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, op1, op2);
    let mut op1 = op1;
    let mut op2 = op2;
    let mut r: M256 = 1.into();

    while op2 != 0.into() {
        if op2 & 1.into() != 0.into() {
            r = r * op1;
        }
        op2 = op2 >> 1;
        op1 = op1 * op1;
    }

    push!(state, r);
}

pub fn signextend<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, op1, op2);

    if op1 > M256::from(32) {
        push!(state, op2);
    } else {
        let mut ret = M256::zero();
        let len: usize = op1.as_usize();
        let t: usize = 8 * (len + 1) - 1;
        let t_bit_mask = M256::one() << t;
        let t_value = (op2 & t_bit_mask) >> t;
        for i in 0..256 {
            let bit_mask = M256::one() << i;
            let i_value = (op2 & bit_mask) >> i;
            if i <= t {
                ret = ret + (i_value << i);
            } else {
                ret = ret + (t_value << i);
            }
        }
        push!(state, ret);
    }
}
