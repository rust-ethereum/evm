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

        // Instruction::LT => Instruction::LT,
        // Instruction::GT => Instruction::GT,
        // Instruction::SLT => Instruction::SLT,
        // Instruction::SGT => Instruction::SGT,
        // Instruction::EQ => Instruction::EQ,
        // Instruction::ISZERO => Instruction::ISZERO,
        // Instruction::AND => Instruction::AND,
        // Instruction::OR => Instruction::OR,
        // Instruction::XOR => Instruction::XOR,
        // Instruction::NOT => Instruction::NOT,
        // Instruction::BYTE => Instruction::BYTE,

        // Instruction::SHA3 => Instruction::SHA3,

        // Instruction::ADDRESS => Instruction::ADDRESS,
        // Instruction::BALANCE => Instruction::BALANCE,
        // Instruction::ORIGIN => Instruction::ORIGIN,
        // Instruction::CALLER => Instruction::CALLER,
        // Instruction::CALLVALUE => Instruction::CALLVALUE,
        // Instruction::CALLDATALOAD => Instruction::CALLDATALOAD,
        // Instruction::CALLDATASIZE => Instruction::CALLDATASIZE,
        // Instruction::CALLDATACOPY => Instruction::CALLDATACOPY,
        // Instruction::CODESIZE => Instruction::CODESIZE,
        // Instruction::CODECOPY => Instruction::CODECOPY,
        // Instruction::GASPRICE => Instruction::GASPRICE,
        // Instruction::EXTCODESIZE => Instruction::EXTCODESIZE,
        // Instruction::EXTCODECOPY => Instruction::EXTCODECOPY,

        // Instruction::BLOCKHASH => Instruction::BLOCKHASH,
        // Instruction::COINBASE => Instruction::COINBASE,
        // Instruction::TIMESTAMP => Instruction::TIMESTAMP,
        // Instruction::NUMBER => Instruction::NUMBER,
        // Instruction::DIFFICULTY => Instruction::DIFFICULTY,
        // Instruction::GASLIMIT => Instruction::GASLIMIT,

        // Instruction::POP => Instruction::POP,
        // Instruction::MLOAD => Instruction::MLOAD,
        // Instruction::MSTORE => Instruction::MSTORE,
        // Instruction::MSTORE8 => Instruction::MSTORE8,
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
        // Instruction::JUMP => Instruction::JUMP,
        // Instruction::JUMPI => Instruction::JUMPI,
        // Instruction::PC => Instruction::PC,
        // Instruction::MSIZE => Instruction::MSIZE,
        // Instruction::GAS => Instruction::GAS,
        // Instruction::JUMPDEST => Instruction::JUMPDEST,

        Instruction::PUSH(v) => { state.stack.check_pop_push(0, 1)?; Ok(None) }

        // Instruction::DUP(v) => Instruction::DUP(v),
        // Instruction::SWAP(v) => Instruction::SWAP(v),
        // Instruction::LOG(v) => Instruction::LOG(v),

        // Instruction::CREATE => Instruction::CREATE,
        // Instruction::CALL => Instruction::CALL,
        // Instruction::CALLCODE => Instruction::CALLCODE,
        Instruction::RETURN => {
            state.stack.check_pop_push(2, 0)?;
            check_range(state.stack.peek(0).unwrap(), state.stack.peek(1).unwrap())?;
            Ok(None)
        }
        // Instruction::DELEGATECALL => Instruction::DELEGATECALL,

        // Instruction::INVALID => {
        //     return Err(PCError::InvalidInstruction);
        // },
        // Instruction::SUICIDE => Instruction::SUICIDE,

        _ => unimplemented!(),
    }
}
