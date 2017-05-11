use utils::bigint::M256;

use vm::{Memory, Storage, Instruction};
use vm::errors::EvalError;

use vm::eval::{State, ControlCheck};
use super::utils::check_memory_write_range;

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

        Instruction::LT => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::GT => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::SLT => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::SGT => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::EQ => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::ISZERO => { state.stack.check_pop_push(1, 1)?; Ok(None) },
        Instruction::AND => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::OR => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::XOR => { state.stack.check_pop_push(2, 1)?; Ok(None) },
        Instruction::NOT => { state.stack.check_pop_push(1, 1)?; Ok(None) },
        Instruction::BYTE => { state.stack.check_pop_push(2, 1)?; Ok(None) },

        Instruction::SHA3 => unimplemented!(),

        Instruction::ADDRESS => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::BALANCE => {
            state.stack.check_pop_push(1, 1)?;
            state.account_state.balance(state.stack.peek(0).unwrap().into())?;
            Ok(None)
        },
        Instruction::ORIGIN => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::CALLER => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::CALLVALUE => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::CALLDATALOAD => { state.stack.check_pop_push(1, 1)?; Ok(None) },
        Instruction::CALLDATASIZE => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::CALLDATACOPY => {
            state.stack.check_pop_push(3, 0)?;
            check_memory_write_range(&state.memory,
                                     state.stack.peek(0).unwrap(), state.stack.peek(2).unwrap())?;
            Ok(None)
        },
        Instruction::CODESIZE => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::CODECOPY => {
            state.stack.check_pop_push(3, 0)?;
            check_memory_write_range(&state.memory,
                               state.stack.peek(0).unwrap(), state.stack.peek(2).unwrap())?;
            Ok(None)
        },
        Instruction::GASPRICE => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::EXTCODESIZE => {
            state.stack.check_pop_push(1, 1)?;
            state.account_state.code(state.stack.peek(0).unwrap().into())?;
            Ok(None)
        },
        Instruction::EXTCODECOPY => {
            state.stack.check_pop_push(4, 0)?;
            state.account_state.code(state.stack.peek(0).unwrap().into())?;
            check_memory_write_range(&state.memory,
                                     state.stack.peek(1).unwrap(), state.stack.peek(3).unwrap())?;
            Ok(None)
        },

        Instruction::BLOCKHASH => {
            state.stack.check_pop_push(1, 1)?;
            let current_number = state.block.number;
            let number = state.stack.peek(0).unwrap();
            if !(number >= current_number || current_number - number > M256::from(256u64)) {
                state.blockhash_state.get(number)?;
            }
            Ok(None)
        },
        Instruction::COINBASE => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::TIMESTAMP => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::NUMBER => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::DIFFICULTY => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::GASLIMIT => { state.stack.check_pop_push(0, 1)?; Ok(None) },

        Instruction::POP => { state.stack.check_pop_push(1, 0)?; Ok(None) },
        Instruction::MLOAD => { state.stack.check_pop_push(1, 1)?; Ok(None) },
        Instruction::MSTORE => {
            state.stack.check_pop_push(2, 0)?;
            state.memory.check_write(state.stack.peek(0).unwrap())?;
            Ok(None)
        },
        Instruction::MSTORE8 => {
            state.stack.check_pop_push(2, 0)?;
            state.memory.check_write(state.stack.peek(0).unwrap())?;
            Ok(None)
        },
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
        Instruction::JUMP => {
            state.stack.check_pop_push(1, 0)?;
            Ok(Some(ControlCheck::Jump(state.stack.peek(0).unwrap())))
        },
        Instruction::JUMPI => {
            state.stack.check_pop_push(2, 0)?;
            if state.stack.peek(1).unwrap() != M256::zero() {
                Ok(Some(ControlCheck::Jump(state.stack.peek(0).unwrap())))
            } else {
                Ok(None)
            }
        },
        Instruction::PC => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::MSIZE => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::GAS => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::JUMPDEST => Ok(None),

        Instruction::PUSH(v) => { state.stack.check_pop_push(0, 1)?; Ok(None) },

        Instruction::DUP(v) => { state.stack.check_pop_push(v, v+1)?; Ok(None) },
        Instruction::SWAP(v) => { state.stack.check_pop_push(v+1, v+1)?; Ok(None) },

        Instruction::LOG(v) => { state.stack.check_pop_push(v+2, 0)?; Ok(None) },
        Instruction::CREATE => unimplemented!(),
        Instruction::CALL => unimplemented!(),
        Instruction::CALLCODE => unimplemented!(),
        Instruction::RETURN => { state.stack.check_pop_push(2, 0)?; Ok(None) },
        Instruction::DELEGATECALL => unimplemented!(),
        Instruction::SUICIDE => {
            state.stack.check_pop_push(1, 0)?;
            state.account_state.require(state.context.address)?;
            Ok(None)
        },
    }
}
