macro_rules! pop {
    ( $machine:expr, $( $x:ident ),* ) => (
        $(
            let $x = $machine.stack.pop().unwrap();
        )*
    );
    ( $machine:expr, $( $x:ident : $t: ty ),* ) => (
        $(
            let $x: $t = $machine.stack.pop().unwrap().into();
        )*
    );
}

macro_rules! push {
    ( $machine:expr, $( $x:expr ),* ) => (
        $(
            $machine.stack.push($x).unwrap();
        )*
    )
}

macro_rules! op2 {
    ( $machine:expr, $op:ident ) => ({
        pop!($machine, op1, op2);
        push!($machine, op1.$op(op2).into());
    });
    ( $machine:expr, $op:ident, $t:ty ) => ({
        pop!($machine, op1:$t, op2:$t);
        push!($machine, op1.$op(op2).into());
    });
}

macro_rules! op2_ref {
    ( $machine:expr, $op:ident ) => ({
        pop!($machine, op1, op2);
        push!($machine, op1.$op(&op2).into());
    });
    ( $machine:expr, $op:ident, $t:ty ) => ({
        pop!($machine, op1:$t, op2:$t);
        push!($machine, op1.$op(&op2).into());
    });
}

mod arithmetic;
mod bitwise;
mod flow;

use utils::gas::Gas;
use utils::bigint::MI256;
use std::ops::{Add, Sub, Mul, Div, Rem, BitAnd, BitOr, BitXor};
use vm::{Memory, Storage, Instruction};
use super::{State, Control};

#[allow(unused_variables)]
pub fn run_opcode<M: Memory + Default, S: Storage + Default + Clone>(instruction: Instruction, state: &mut State<M, S>, stipend_gas: Gas, after_gas: Gas) -> Option<Control> {
    match instruction {
        Instruction::STOP => { Some(Control::Stop) },
        Instruction::ADD => { op2!(state, add); None },
        Instruction::MUL => { op2!(state, mul); None },
        Instruction::SUB => { op2!(state, sub); None },
        Instruction::DIV => { op2!(state, div); None },
        Instruction::SDIV => { op2!(state, div, MI256); None },
        Instruction::MOD => { op2!(state, rem); None },
        Instruction::SMOD => { op2!(state, rem, MI256); None },
        Instruction::ADDMOD => { arithmetic::addmod(state); None },
        Instruction::MULMOD => { arithmetic::mulmod(state); None },
        Instruction::EXP => { arithmetic::exp(state); None },
        Instruction::SIGNEXTEND => { arithmetic::signextend(state); None },

        Instruction::LT => { op2_ref!(state, lt); None },
        Instruction::GT => { op2_ref!(state, gt); None },
        Instruction::SLT => { op2_ref!(state, lt, MI256); None },
        Instruction::SGT => { op2_ref!(state, gt, MI256); None },
        Instruction::EQ => { op2_ref!(state, eq); None },
        Instruction::ISZERO => { bitwise::iszero(state); None },
        Instruction::AND => { op2!(state, bitand); None },
        Instruction::OR => { op2!(state, bitor); None },
        Instruction::XOR => { op2!(state, bitxor); None },
        Instruction::NOT => { bitwise::not(state); None },
        Instruction::BYTE => { bitwise::byte(state); None },

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
        Instruction::SLOAD => { flow::sload(state); None },
        Instruction::SSTORE => { flow::sstore(state); None },
        // Instruction::JUMP => Instruction::JUMP,
        // Instruction::JUMPI => Instruction::JUMPI,
        // Instruction::PC => Instruction::PC,
        // Instruction::MSIZE => Instruction::MSIZE,
        // Instruction::GAS => Instruction::GAS,
        // Instruction::JUMPDEST => Instruction::JUMPDEST,

        Instruction::PUSH(v) => { push!(state, v); None }

        // Instruction::DUP(v) => Instruction::DUP(v),
        // Instruction::SWAP(v) => Instruction::SWAP(v),
        // Instruction::LOG(v) => Instruction::LOG(v),

        // Instruction::CREATE => Instruction::CREATE,
        // Instruction::CALL => Instruction::CALL,
        // Instruction::CALLCODE => Instruction::CALLCODE,
        // Instruction::RETURN => Instruction::RETURN,
        // Instruction::DELEGATECALL => Instruction::DELEGATECALL,

        // Instruction::INVALID => {
        //     return Err(PCError::InvalidInstruction);
        // },
        // Instruction::SUICIDE => Instruction::SUICIDE,

        _ => unimplemented!(),
    }
}
