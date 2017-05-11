use vm::{Memory, Storage, Instruction};
use vm::errors::EvalError;

use vm::eval::{State, ControlCheck};
use super::utils::check_range;

#[allow(unused_variables)]
pub fn check_opcode<M: Memory + Default, S: Storage + Default + Clone>(instruction: Instruction, state: &State<M, S>) -> Result<Option<ControlCheck>, EvalError> {
    match instruction {
        Instruction::STOP => Ok(None),
        Instruction::ADD => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::MUL => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::SUB => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::DIV => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::SDIV => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::MOD => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::SMOD => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::ADDMOD => { state.stack.check_pop_push(3, 1)?; Ok(None) },
        Instruction::MULMOD => { state.stack.check_pop_push(3, 1)?; Ok(None) },
        Instruction::EXP => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::SIGNEXTEND => { state.stack.check_pop_push(2, 1)?; Ok(None) },

        Instruction::LT => unimplemented!(),
        Instruction::GT => unimplemented!(),
        Instruction::SLT => unimplemented!(),
        Instruction::SGT => unimplemented!(),
        Instruction::EQ => unimplemented!(),
        Instruction::ISZERO => unimplemented!(),
        Instruction::AND => unimplemented!(),
        Instruction::OR => unimplemented!(),
        Instruction::XOR => unimplemented!(),
        Instruction::NOT => unimplemented!(),
        Instruction::BYTE => unimplemented!(),

        Instruction::SHA3 => unimplemented!(),

        Instruction::ADDRESS => unimplemented!(),
        Instruction::BALANCE => unimplemented!(),
        Instruction::ORIGIN => unimplemented!(),
        Instruction::CALLER => unimplemented!(),
        Instruction::CALLVALUE => unimplemented!(),
        Instruction::CALLDATALOAD => unimplemented!(),
        Instruction::CALLDATASIZE => unimplemented!(),
        Instruction::CALLDATACOPY => unimplemented!(),
        Instruction::CODESIZE => unimplemented!(),
        Instruction::CODECOPY => unimplemented!(),
        Instruction::GASPRICE => unimplemented!(),
        Instruction::EXTCODESIZE => unimplemented!(),
        Instruction::EXTCODECOPY => unimplemented!(),

        Instruction::BLOCKHASH => unimplemented!(),
        Instruction::COINBASE => unimplemented!(),
        Instruction::TIMESTAMP => unimplemented!(),
        Instruction::NUMBER => unimplemented!(),
        Instruction::DIFFICULTY => unimplemented!(),
        Instruction::GASLIMIT => unimplemented!(),

        Instruction::POP => unimplemented!(),
        Instruction::MLOAD => unimplemented!(),
        Instruction::MSTORE => unimplemented!(),
        Instruction::MSTORE8 => unimplemented!(),
        Instruction::SLOAD => {
            state.stack.check_pop_push(1, 1)?;
            state.account_state.require(state.context.address)?;
            Ok(None)
        },
        Instruction::SSTORE => {
            state.stack.check_pop_push(2, 0)?;
            state.account_state.storage(state.context.address)?.
                check_write(state.stack.peek(0).unwrap())?;
            Ok(None)
        },
        Instruction::JUMP => unimplemented!(),
        Instruction::JUMPI => unimplemented!(),
        Instruction::PC => unimplemented!(),
        Instruction::MSIZE => unimplemented!(),
        Instruction::GAS => unimplemented!(),
        Instruction::JUMPDEST => unimplemented!(),

        Instruction::PUSH(v) => { state.stack.check_pop_push(0, 1)?; Ok(None) }

        Instruction::DUP(v) => unimplemented!(),
        Instruction::SWAP(v) => unimplemented!(),
        Instruction::LOG(v) => unimplemented!(),

        Instruction::CREATE => unimplemented!(),
        Instruction::CALL => unimplemented!(),
        Instruction::CALLCODE => unimplemented!(),
        Instruction::RETURN => {
            state.stack.check_pop_push(2, 0)?;
            check_range(state.stack.peek(0).unwrap(), state.stack.peek(1).unwrap())?;
            Ok(None)
        }
        Instruction::DELEGATECALL => unimplemented!(),
        Instruction::SUICIDE => unimplemented!(),
    }
}
