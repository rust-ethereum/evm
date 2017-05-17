//! Bitwise instructions

use utils::bigint::M256;

use vm::{Memory, Storage};
use super::State;

pub fn iszero<M: Memory + Default, S: Storage + Default + Clone>(state: &mut State<M, S>) {
    pop!(state, op1);

    if op1 == M256::zero() {
        push!(state, M256::from(1u64));
    } else {
        push!(state, M256::zero());
    }
}

pub fn not<M: Memory + Default, S: Storage + Default + Clone>(state: &mut State<M, S>) {
    pop!(state, op1);
    push!(state, !op1);
}

pub fn byte<M: Memory + Default, S: Storage + Default + Clone>(state: &mut State<M, S>) {
    pop!(state, op1, op2);

    let mut ret = M256::zero();

    for i in 0..256 {
        if i < 8 && op1 < 32.into() {
            let o: usize = op1.into();
            let t = 255 - (7 - i + 8 * o);
            let bit_mask = M256::one() << t;
            let value = (op2 & bit_mask) >> t;
            ret = ret + (value << i);
        }
    }

    push!(state, ret);
}
