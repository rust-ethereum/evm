use vm::{Memory, Storage, Instruction};
use vm::errors::EvalError;

use vm::eval::State;

#[allow(unused_variables)]
pub fn check_opcode<M: Memory + Default, S: Storage + Default>(instruction: Instruction, state: &State<M, S>) -> Result<(), EvalError> {
    unimplemented!()
}
