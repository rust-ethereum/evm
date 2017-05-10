mod check;

pub use self::check::check_opcode;

use utils::gas::Gas;
use vm::{Memory, Storage, Instruction};
use vm::errors::EvalError;
use super::State;

#[allow(unused_variables)]
pub fn run_opcode<M: Memory + Default, S: Storage + Default + Clone>(instruction: Instruction, state: &mut State<M, S>, stipend_gas: Gas, after_gas: Gas) -> Result<(), EvalError> {
    unimplemented!()
}
