//! EVM Program Counter.

use utils::bigint::M256;
use utils::opcode::Opcode;
use std::cmp::min;
use super::errors::PCError;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Instructions for the program counter. This is the same as `Opcode`
/// except `PUSH`, which might take longer length.
pub enum Instruction {
    STOP, ADD, MUL, SUB, DIV, SDIV, MOD, SMOD, ADDMOD, MULMOD, EXP,
    SIGNEXTEND, LT, GT, SLT, SGT, EQ, ISZERO, AND, OR, XOR, NOT, BYTE,
    SHA3, ADDRESS, BALANCE, ORIGIN, CALLER, CALLVALUE, CALLDATALOAD,
    CALLDATASIZE, CALLDATACOPY, CODESIZE, CODECOPY, GASPRICE,
    EXTCODESIZE, EXTCODECOPY, BLOCKHASH, COINBASE, TIMESTAMP, NUMBER,
    DIFFICULTY, GASLIMIT, POP, MLOAD, MSTORE, MSTORE8, SLOAD, SSTORE,
    JUMP, JUMPI, PC, MSIZE, GAS, JUMPDEST, CREATE, CALL, CALLCODE,
    RETURN, DELEGATECALL, SUICIDE,

    PUSH(M256),
    DUP(usize),
    SWAP(usize),
    LOG(usize),
}

/// Represents a program counter in EVM.
pub struct PC {
    position: usize,
    code: Vec<u8>,
    valids: Vec<bool>,
}

impl Default for PC {
    fn default() -> PC {
        PC {
            position: 0,
            code: Vec::new(),
            valids: Vec::new(),
        }
    }
}

impl PC {
    /// Create a new program counter from the given code.
    pub fn new(code: &[u8]) -> Self {
        let code: Vec<u8> = code.into();
        let mut valids: Vec<bool> = Vec::with_capacity(code.len());
        valids.resize(code.len(), false);

        let mut i = 0;
        while i < code.len() {
            let opcode: Opcode = code[i].into();
            match opcode {
                Opcode::JUMPDEST => {
                    valids[i] = true;
                    i = i + 1;
                },
                Opcode::PUSH(v) => {
                    i = i + v + 1;
                },
                _ => {
                    i = i + 1;
                }
            }
        }

        PC {
            position: 0,
            code: code,
            valids: valids,
        }
    }

    fn read_bytes(&self, from_position: usize, byte_count: usize) -> Result<M256, PCError> {
        if from_position > self.code.len() {
            return Err(PCError::Overflow);
        }
        let position = from_position;
        if position.checked_add(byte_count).is_none() {
            return Err(PCError::IndexNotSupported);
        }
        let max = min(position + byte_count, self.code.len());
        Ok(M256::from(&self.code[position..max]))
    }

    /// Jump to a position in the code. The destination must be valid
    /// to jump.
    pub fn jump(&mut self, position: usize) -> Result<(), PCError> {
        if position >= self.code.len() {
            return Err(PCError::Overflow);
        }

        if !self.valids[position] {
            return Err(PCError::BadJumpDest);
        }

        self.position = position;
        Ok(())
    }

    /// Get the current program counter position.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Check whether the position is a valid jump destination. If
    /// not, returns `PCError`.
    pub fn check_valid(&self, position: usize) -> Result<(), PCError> {
        if self.is_valid(position) {
            Ok(())
        } else {
            Err(PCError::BadJumpDest)
        }
    }

    /// Returns `true` if the position is a valid jump destination. If
    /// not, returns `false`.
    pub fn is_valid(&self, position: usize) -> bool {
        if position >= self.code.len() {
            return false;
        }

        if !self.valids[position] {
            return false;
        }

        return true;
    }

    /// Check whether the PC is ended. Next `read` on this PC would
    /// result in `PCError::PCOverflow`.
    pub fn is_end(&self) -> bool {
        self.position == self.code.len()
    }

    /// Peek the next instruction.
    pub fn peek(&self) -> Result<Instruction, PCError> {
        let position = self.position;
        if position >= self.code.len() {
            return Err(PCError::Overflow);
        }
        let opcode: Opcode = self.code[position].into();
        Ok(match opcode {
            Opcode::STOP => Instruction::STOP,
            Opcode::ADD => Instruction::ADD,
            Opcode::MUL => Instruction::MUL,
            Opcode::SUB => Instruction::SUB,
            Opcode::DIV => Instruction::DIV,
            Opcode::SDIV => Instruction::SDIV,
            Opcode::MOD => Instruction::MOD,
            Opcode::SMOD => Instruction::SMOD,
            Opcode::ADDMOD => Instruction::ADDMOD,
            Opcode::MULMOD => Instruction::MULMOD,
            Opcode::EXP => Instruction::EXP,
            Opcode::SIGNEXTEND => Instruction::SIGNEXTEND,

            Opcode::LT => Instruction::LT,
            Opcode::GT => Instruction::GT,
            Opcode::SLT => Instruction::SLT,
            Opcode::SGT => Instruction::SGT,
            Opcode::EQ => Instruction::EQ,
            Opcode::ISZERO => Instruction::ISZERO,
            Opcode::AND => Instruction::AND,
            Opcode::OR => Instruction::OR,
            Opcode::XOR => Instruction::XOR,
            Opcode::NOT => Instruction::NOT,
            Opcode::BYTE => Instruction::BYTE,

            Opcode::SHA3 => Instruction::SHA3,

            Opcode::ADDRESS => Instruction::ADDRESS,
            Opcode::BALANCE => Instruction::BALANCE,
            Opcode::ORIGIN => Instruction::ORIGIN,
            Opcode::CALLER => Instruction::CALLER,
            Opcode::CALLVALUE => Instruction::CALLVALUE,
            Opcode::CALLDATALOAD => Instruction::CALLDATALOAD,
            Opcode::CALLDATASIZE => Instruction::CALLDATASIZE,
            Opcode::CALLDATACOPY => Instruction::CALLDATACOPY,
            Opcode::CODESIZE => Instruction::CODESIZE,
            Opcode::CODECOPY => Instruction::CODECOPY,
            Opcode::GASPRICE => Instruction::GASPRICE,
            Opcode::EXTCODESIZE => Instruction::EXTCODESIZE,
            Opcode::EXTCODECOPY => Instruction::EXTCODECOPY,

            Opcode::BLOCKHASH => Instruction::BLOCKHASH,
            Opcode::COINBASE => Instruction::COINBASE,
            Opcode::TIMESTAMP => Instruction::TIMESTAMP,
            Opcode::NUMBER => Instruction::NUMBER,
            Opcode::DIFFICULTY => Instruction::DIFFICULTY,
            Opcode::GASLIMIT => Instruction::GASLIMIT,

            Opcode::POP => Instruction::POP,
            Opcode::MLOAD => Instruction::MLOAD,
            Opcode::MSTORE => Instruction::MSTORE,
            Opcode::MSTORE8 => Instruction::MSTORE8,
            Opcode::SLOAD => Instruction::SLOAD,
            Opcode::SSTORE => Instruction::SSTORE,
            Opcode::JUMP => Instruction::JUMP,
            Opcode::JUMPI => Instruction::JUMPI,
            Opcode::PC => Instruction::PC,
            Opcode::MSIZE => Instruction::MSIZE,
            Opcode::GAS => Instruction::GAS,
            Opcode::JUMPDEST => Instruction::JUMPDEST,

            Opcode::PUSH(v) => {
                let param = self.read_bytes(position + 1, v)?;
                Instruction::PUSH(param)
            },

            Opcode::DUP(v) => Instruction::DUP(v),
            Opcode::SWAP(v) => Instruction::SWAP(v),
            Opcode::LOG(v) => Instruction::LOG(v),

            Opcode::CREATE => Instruction::CREATE,
            Opcode::CALL => Instruction::CALL,
            Opcode::CALLCODE => Instruction::CALLCODE,
            Opcode::RETURN => Instruction::RETURN,
            Opcode::DELEGATECALL => Instruction::DELEGATECALL,

            Opcode::INVALID => {
                return Err(PCError::InvalidOpcode);
            },
            Opcode::SUICIDE => Instruction::SUICIDE,
        })
    }

    /// Read the next instruction and step the program counter.
    pub fn read(&mut self) -> Result<Instruction, PCError> {
        let result = self.peek()?;
        let opcode: Opcode = self.code[self.position].into();
        match opcode {
            Opcode::PUSH(v) => {
                self.position = min(self.position + v + 1, self.code.len());
            },
            _ => {
                self.position = self.position + 1;
            },
        }
        Ok(result)
    }
}
