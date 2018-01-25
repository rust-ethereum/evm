//! Check logic for instructions

use bigint::{U256, M256, Gas};

use ::{Memory, Instruction, Patch};
use errors::{OnChainError, NotSupportedError, EvalOnChainError};
use eval::{State, Runtime, ControlCheck};

use super::util::check_range;

#[allow(unused_variables)]
pub fn extra_check_opcode<M: Memory + Default, P: Patch>(instruction: Instruction, state: &State<M, P>, stipend_gas: Gas, after_gas: Gas) -> Result<(), OnChainError> {
    match instruction {
        Instruction::CALL | Instruction::CALLCODE | Instruction::DELEGATECALL => {
            if P::err_on_call_with_more_gas() && after_gas < state.stack.peek(0).unwrap().into() {
                Err(OnChainError::EmptyGas)
            } else {
                Ok(())
            }
        },
        _ => Ok(())
    }
}

pub fn check_support<M: Memory + Default, P: Patch>(instruction: Instruction, state: &State<M, P>) -> Result<(), NotSupportedError> {
    match instruction {
        Instruction::MSTORE => {
            state.memory.check_write(state.stack.peek(0).unwrap().into())?;
            Ok(())
        },
        Instruction::MSTORE8 => {
            state.memory.check_write(state.stack.peek(0).unwrap().into())?;
            Ok(())
        },
        Instruction::CALLDATACOPY => {
            state.memory.check_write_range(
                state.stack.peek(0).unwrap().into(), state.stack.peek(2).unwrap().into())?;
            Ok(())
        },
        Instruction::CODECOPY => {
            state.memory.check_write_range(
                state.stack.peek(0).unwrap().into(), state.stack.peek(2).unwrap().into())?;
            Ok(())
        },
        Instruction::EXTCODECOPY => {
            state.memory.check_write_range(
                state.stack.peek(1).unwrap().into(), state.stack.peek(3).unwrap().into())?;
            Ok(())
        },
        Instruction::CALL => {
            state.memory.check_write_range(
                state.stack.peek(5).unwrap().into(), state.stack.peek(6).unwrap().into())?;
            Ok(())
        },
        Instruction::CALLCODE => {
            state.memory.check_write_range(
                state.stack.peek(5).unwrap().into(), state.stack.peek(6).unwrap().into())?;
            Ok(())
        },
        Instruction::DELEGATECALL => {
            state.memory.check_write_range(
                state.stack.peek(4).unwrap().into(), state.stack.peek(5).unwrap().into())?;
            Ok(())
        },
        _ => Ok(()),
    }
}

#[allow(unused_variables)]
/// Check whether `run_opcode` would be static.
pub fn check_static<M: Memory + Default, P: Patch>(instruction: Instruction, state: &State<M, P>, runtime: &Runtime) -> Result<(), EvalOnChainError> {
    match instruction {
        Instruction::STOP |
        Instruction::ADD |
        Instruction::MUL |
        Instruction::SUB |
        Instruction::DIV |
        Instruction::SDIV |
        Instruction::MOD |
        Instruction::SMOD |
        Instruction::ADDMOD |
        Instruction::MULMOD |
        Instruction::EXP |
        Instruction::SIGNEXTEND => Ok(()),

        Instruction::LT |
        Instruction::GT |
        Instruction::SLT |
        Instruction::SGT |
        Instruction::EQ |
        Instruction::ISZERO |
        Instruction::AND |
        Instruction::OR |
        Instruction::XOR |
        Instruction::NOT |
        Instruction::BYTE => Ok(()),

        Instruction::SHA3 => Ok(()),

        Instruction::ADDRESS |
        Instruction::BALANCE |
        Instruction::ORIGIN |
        Instruction::CALLER |
        Instruction::CALLVALUE |
        Instruction::CALLDATALOAD |
        Instruction::CALLDATASIZE |
        Instruction::CALLDATACOPY |
        Instruction::CODESIZE |
        Instruction::CODECOPY |
        Instruction::GASPRICE |
        Instruction::EXTCODESIZE |
        Instruction::EXTCODECOPY |
        Instruction::RETURNDATASIZE |
        Instruction::RETURNDATACOPY => Ok(()),

        Instruction::BLOCKHASH |
        Instruction::COINBASE |
        Instruction::TIMESTAMP |
        Instruction::NUMBER |
        Instruction::DIFFICULTY |
        Instruction::GASLIMIT => Ok(()),

        Instruction::POP |
        Instruction::MLOAD |
        Instruction::MSTORE |
        Instruction::MSTORE8 => Ok(()),

        Instruction::SLOAD => Ok(()),
        Instruction::SSTORE => Err(EvalOnChainError::OnChain(OnChainError::NotStatic)),

        Instruction::JUMP |
        Instruction::JUMPI |
        Instruction::PC |
        Instruction::MSIZE |
        Instruction::GAS |
        Instruction::JUMPDEST => Ok(()),

        Instruction::PUSH(_) |
        Instruction::DUP(_) |
        Instruction::SWAP(_) => Ok(()),

        Instruction::LOG(_) => Err(EvalOnChainError::OnChain(OnChainError::NotStatic)),
        Instruction::CREATE => Err(EvalOnChainError::OnChain(OnChainError::NotStatic)),
        Instruction::CALL => {
            let value: U256 = state.stack.peek(2).unwrap().into();
            if value != U256::zero() {
                Err(EvalOnChainError::OnChain(OnChainError::NotStatic))
            } else {
                Ok(())
            }
        },
        Instruction::STATICCALL => Ok(()),
        Instruction::CALLCODE => Ok(()),
        Instruction::RETURN => Ok(()),
        Instruction::REVERT => Ok(()),
        Instruction::DELEGATECALL => Ok(()),
        Instruction::SUICIDE => Err(EvalOnChainError::OnChain(OnChainError::NotStatic)),
    }
}

#[allow(unused_variables)]
/// Check whether `run_opcode` would fail without mutating any of the
/// machine state.
pub fn check_opcode<M: Memory + Default, P: Patch>(instruction: Instruction, state: &State<M, P>, runtime: &Runtime) -> Result<Option<ControlCheck>, EvalOnChainError> {
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

        Instruction::SHA3 => {
            state.stack.check_pop_push(2, 1)?;
            check_range(state.stack.peek(0).unwrap().into(), state.stack.peek(1).unwrap().into())?;
            Ok(None)
        },

        Instruction::ADDRESS => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::BALANCE => {
            state.stack.check_pop_push(1, 1)?;
            state.account_state.require(state.stack.peek(0).unwrap().into())?;
            Ok(None)
        },
        Instruction::ORIGIN => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::CALLER => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::CALLVALUE => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::CALLDATALOAD => { state.stack.check_pop_push(1, 1)?; Ok(None) },
        Instruction::CALLDATASIZE => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::CALLDATACOPY => {
            state.stack.check_pop_push(3, 0)?;
            check_range(state.stack.peek(0).unwrap().into(), state.stack.peek(2).unwrap().into())?;
            Ok(None)
        },
        Instruction::CODESIZE => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::CODECOPY => {
            state.stack.check_pop_push(3, 0)?;
            check_range(state.stack.peek(0).unwrap().into(), state.stack.peek(2).unwrap().into())?;
            Ok(None)
        },
        Instruction::GASPRICE => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::EXTCODESIZE => {
            state.stack.check_pop_push(1, 1)?;
            state.account_state.require_code(state.stack.peek(0).unwrap().into())?;
            Ok(None)
        },
        Instruction::EXTCODECOPY => {
            state.stack.check_pop_push(4, 0)?;
            state.account_state.require_code(state.stack.peek(0).unwrap().into())?;
            check_range(state.stack.peek(1).unwrap().into(), state.stack.peek(3).unwrap().into())?;
            Ok(None)
        },
        Instruction::RETURNDATASIZE => { state.stack.check_pop_push(0, 1)?; Ok(None) },
        Instruction::RETURNDATACOPY => {
            state.stack.check_pop_push(3, 0)?;
            let start = state.stack.peek(0).unwrap().into();
            let end = state.stack.peek(2).unwrap().into();
            check_range(start, end)?;
            if start + end > U256::from(state.ret.len()) {
                Err(EvalOnChainError::OnChain(OnChainError::InvalidRange))
            } else {
                Ok(None)
            }
        },

        Instruction::BLOCKHASH => {
            state.stack.check_pop_push(1, 1)?;
            let current_number = runtime.block.number;
            let number: U256 = state.stack.peek(0).unwrap().into();
            if !(number >= current_number || current_number - number > U256::from(256u64)) {
                runtime.blockhash_state.get(number)?;
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
            Ok(None)
        },
        Instruction::MSTORE8 => {
            state.stack.check_pop_push(2, 0)?;
            Ok(None)
        },
        Instruction::SLOAD => {
            state.stack.check_pop_push(1, 1)?;
            state.account_state.require(state.context.address)?;
            state.account_state.require_storage(state.context.address, state.stack.peek(0).unwrap().into())?;
            Ok(None)
        },
        Instruction::SSTORE => {
            state.stack.check_pop_push(2, 0)?;
            state.account_state.require(state.context.address)?;
            state.account_state.require_storage(state.context.address, state.stack.peek(0).unwrap().into())?;
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

        Instruction::LOG(v) => {
            state.stack.check_pop_push(v+2, 0)?;
            check_range(state.stack.peek(0).unwrap().into(), state.stack.peek(1).unwrap().into())?;
            Ok(None)
        },
        Instruction::CREATE => {
            state.stack.check_pop_push(3, 1)?;
            check_range(state.stack.peek(1).unwrap().into(), state.stack.peek(2).unwrap().into())?;
            state.account_state.require(state.context.address)?;
            Ok(None)
        },
        Instruction::CALL => {
            state.stack.check_pop_push(7, 1)?;
            check_range(state.stack.peek(3).unwrap().into(), state.stack.peek(4).unwrap().into())?;
            check_range(state.stack.peek(5).unwrap().into(), state.stack.peek(6).unwrap().into())?;
            state.account_state.require(state.context.address)?;
            state.account_state.require(state.stack.peek(1).unwrap().into())?;
            Ok(None)
        },
        Instruction::STATICCALL => {
            state.stack.check_pop_push(6, 1)?;
            check_range(state.stack.peek(2).unwrap().into(), state.stack.peek(3).unwrap().into())?;
            check_range(state.stack.peek(4).unwrap().into(), state.stack.peek(5).unwrap().into())?;
            state.account_state.require(state.context.address)?;
            state.account_state.require(state.stack.peek(1).unwrap().into())?;
            Ok(None)
        },
        Instruction::CALLCODE => {
            state.stack.check_pop_push(7, 1)?;
            check_range(state.stack.peek(3).unwrap().into(), state.stack.peek(4).unwrap().into())?;
            check_range(state.stack.peek(5).unwrap().into(), state.stack.peek(6).unwrap().into())?;
            state.account_state.require(state.context.address)?;
            state.account_state.require(state.stack.peek(1).unwrap().into())?;
            Ok(None)
        },
        Instruction::RETURN => {
            state.stack.check_pop_push(2, 0)?;
            check_range(state.stack.peek(0).unwrap().into(), state.stack.peek(1).unwrap().into())?;
            Ok(None)
        },
        Instruction::REVERT => {
            state.stack.check_pop_push(2, 0)?;
            check_range(state.stack.peek(0).unwrap().into(), state.stack.peek(1).unwrap().into())?;
            Ok(None)
        },
        Instruction::DELEGATECALL => {
            state.stack.check_pop_push(6, 1)?;
            check_range(state.stack.peek(2).unwrap().into(), state.stack.peek(3).unwrap().into())?;
            check_range(state.stack.peek(4).unwrap().into(), state.stack.peek(5).unwrap().into())?;
            state.account_state.require(state.context.address)?;
            state.account_state.require(state.stack.peek(1).unwrap().into())?;
            Ok(None)
        },
        Instruction::SUICIDE => {
            state.stack.check_pop_push(1, 0)?;
            state.account_state.require(state.context.address)?;
            state.account_state.require(state.stack.peek(0).unwrap().into())?;
            Ok(None)
        },
    }
}
