//! Bitwise instructions

use bigint::M256;

use ::Memory;
use super::State;
use patch::Patch;

pub fn iszero<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, op1);

    if op1 == M256::zero() {
        push!(state, M256::from(1u64));
    } else {
        push!(state, M256::zero());
    }
}

pub fn not<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
    pop!(state, op1);
    push!(state, !op1);
}

pub fn byte<M: Memory + Default, P: Patch>(state: &mut State<M, P>) {
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
