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
mod environment;

use utils::gas::Gas;
use utils::bigint::{M256, MI256};
use utils::address::Address;
use std::ops::{Add, Sub, Mul, Div, Rem, BitAnd, BitOr, BitXor};
use vm::{Memory, Storage, Instruction};
use super::{State, Control};
use super::utils::{copy_from_memory, copy_into_memory};

#[allow(unused_variables)]
pub fn run_opcode<M: Memory + Default, S: Storage + Default + Clone>(pc: (Instruction, usize), state: &mut State<M, S>, stipend_gas: Gas, after_gas: Gas) -> Option<Control> {
    match pc.0 {
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

        Instruction::ADDRESS => { push!(state, state.context.address.into()); None },
        Instruction::BALANCE => { pop!(state, address: Address);
                                  push!(state, state.account_state.balance(address).unwrap().into());
                                  None },
        Instruction::ORIGIN => { push!(state, state.context.origin.into()); None },
        Instruction::CALLER => { push!(state, state.context.caller.into()); None },
        Instruction::CALLVALUE => { push!(state, state.context.value.into()); None },
        Instruction::CALLDATALOAD => { environment::calldataload(state); None },
        Instruction::CALLDATASIZE => { push!(state, state.context.data.len().into()); None },
        Instruction::CALLDATACOPY => { pop!(state, memory_index, data_index, len);
                                       copy_into_memory(&mut state.memory,
                                                        state.context.data.as_slice(),
                                                        memory_index, data_index, len);
                                       None },
        Instruction::CODESIZE => { push!(state, state.context.code.len().into()); None },
        Instruction::CODECOPY => { pop!(state, memory_index, code_index, len);
                                   copy_into_memory(&mut state.memory,
                                                    state.context.code.as_slice(),
                                                    memory_index, code_index, len);
                                   None },
        Instruction::GASPRICE => { push!(state, state.context.gas_price.into()); None },
        Instruction::EXTCODESIZE => { pop!(state, address: Address);
                                      push!(state,
                                            state.account_state.code(address).unwrap().len().into());
                                      None },
        Instruction::EXTCODECOPY => { pop!(state, address: Address);
                                      pop!(state, memory_index, code_index, len);
                                      copy_into_memory(&mut state.memory,
                                                       state.account_state.code(address).unwrap(),
                                                       memory_index, code_index, len);
                                      None },

        Instruction::BLOCKHASH => { pop!(state, number);
                                    let current_number = state.block.number;
                                    if number >= current_number || current_number - number > M256::from(256u64) {
                                        push!(state, state.blockhash_state.get(number).unwrap());
                                    } else {
                                        push!(state, M256::zero());
                                    }
                                    None },
        Instruction::COINBASE => { push!(state, state.block.coinbase.into()); None },
        Instruction::TIMESTAMP => { push!(state, state.block.timestamp); None },
        Instruction::NUMBER => { push!(state, state.block.number); None },
        Instruction::DIFFICULTY => { push!(state, state.block.difficulty); None },
        Instruction::GASLIMIT => { push!(state, state.block.gas_limit.into()); None },

        Instruction::POP => { state.stack.pop().unwrap(); None },
        Instruction::MLOAD => { flow::mload(state); None },
        Instruction::MSTORE => { flow::mstore(state); None },
        Instruction::MSTORE8 => { flow::mstore8(state); None },
        Instruction::SLOAD => { flow::sload(state); None },
        Instruction::SSTORE => { flow::sstore(state); None },
        Instruction::JUMP => { pop!(state, dest); Some(Control::Jump(dest)) }
        Instruction::JUMPI => { pop!(state, dest, value);
                                if value != M256::zero() {
                                    Some(Control::Jump(dest))
                                } else {
                                    None
                                } },
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
        Instruction::RETURN => { pop!(state, start, len);
                                 state.out = copy_from_memory(&mut state.memory, start, len);
                                 Some(Control::Stop) },
        // Instruction::DELEGATECALL => Instruction::DELEGATECALL,

        // Instruction::INVALID => {
        //     return Err(PCError::InvalidInstruction);
        // },
        // Instruction::SUICIDE => Instruction::SUICIDE,

        _ => unimplemented!(),
    }
}
