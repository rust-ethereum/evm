//! EVM Program Counter.

#[cfg(not(feature = "std"))]
use alloc::Vec;

use bigint::M256;
use util::opcode::Opcode;
#[cfg(feature = "std")] use std::cmp::min;
#[cfg(feature = "std")] use std::marker::PhantomData;
#[cfg(not(feature = "std"))] use core::cmp::min;
#[cfg(not(feature = "std"))] use core::marker::PhantomData;

use super::Patch;
use super::errors::OnChainError;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[allow(missing_docs)]
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

pub struct Valids(Vec<bool>);

impl Valids {
    pub fn new(code: &[u8]) -> Self {
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

        Valids(valids)
    }

    pub fn len(&self) -> usize { self.0.len() }

    /// Returns `true` if the position is a valid jump destination. If
    /// not, returns `false`.
    pub fn is_valid(&self, position: usize) -> bool {
        if position >= self.0.len() {
            return false;
        }

        if !self.0[position] {
            return false;
        }

        return true;
    }
}

/// Represents a program counter in EVM.
pub struct PC<'a, P: Patch> {
    position: &'a usize,
    code: &'a [u8],
    valids: &'a Valids,
    _patch: PhantomData<P>,
}

impl<'a, P: Patch> PC<'a, P> {
    /// Create a new program counter from the given code.
    pub fn new(code: &'a [u8], valids: &'a Valids, position: &'a usize) -> Self {
        Self {
            code, valids, position,
            _patch: PhantomData,
        }
    }

    fn read_bytes(&self, from_position: usize, byte_count: usize) -> Result<M256, OnChainError> {
        if from_position > self.code.len() {
            return Err(OnChainError::PCOverflow);
        }
        let position = from_position;
        let max = min(position.saturating_add(byte_count), self.code.len());
        Ok(M256::from(&self.code[position..max]))
    }

    /// Get the code bytearray.
    pub fn code(&self) -> &[u8] {
        &self.code
    }

    /// Get the current program counter position.
    pub fn position(&self) -> usize {
        *self.position
    }

    /// Get the current opcode position. Should only be used when debugging.
    pub fn opcode_position(&self) -> usize {
        let mut o = 0;
        let mut i = 0;
        while i <= *self.position {
            let opcode: Opcode = self.code[i].into();
            match opcode {
                Opcode::PUSH(v) => {
                    i = i + v + 1;
                },
                _ => {
                    i = i + 1;
                }
            }
            o = o + 1;
        }
        o
    }

    /// Returns `true` if the position is a valid jump destination. If
    /// not, returns `false`.
    pub fn is_valid(&self, position: usize) -> bool {
        self.valids.is_valid(position)
    }

    /// Check whether the PC is ended. Next `read` on this PC would
    /// result in `PCError::PCOverflow`.
    pub fn is_end(&self) -> bool {
        *self.position == self.code.len()
    }

    /// Peek the next instruction.
    pub fn peek(&self) -> Result<Instruction, OnChainError> {
        let position = self.position;
        if *position >= self.code.len() {
            return Err(OnChainError::PCOverflow);
        }
        let opcode: Opcode = self.code[*position].into();
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
                let param = self.read_bytes(*position + 1, v)?;
                Instruction::PUSH(param)
            },

            Opcode::DUP(v) => Instruction::DUP(v),
            Opcode::SWAP(v) => Instruction::SWAP(v),
            Opcode::LOG(v) => Instruction::LOG(v),

            Opcode::CREATE => Instruction::CREATE,
            Opcode::CALL => Instruction::CALL,
            Opcode::CALLCODE => Instruction::CALLCODE,
            Opcode::RETURN => Instruction::RETURN,
            Opcode::DELEGATECALL => {
                if P::has_delegate_call() {
                    Instruction::DELEGATECALL
                } else {
                    return Err(OnChainError::InvalidOpcode);
                }
            },

            Opcode::INVALID => {
                return Err(OnChainError::InvalidOpcode);
            },
            Opcode::SUICIDE => Instruction::SUICIDE,
        })
    }
}

/// Represents a program counter in EVM.
pub struct PCMut<'a, P: Patch> {
    position: &'a mut usize,
    code: &'a [u8],
    valids: &'a Valids,
    _patch: PhantomData<P>,
}

impl<'a, P: Patch> PCMut<'a, P> {
    /// Create a new program counter from the given code.
    pub fn new(code: &'a [u8], valids: &'a Valids, position: &'a mut usize) -> Self {
        Self {
            code, valids, position,
            _patch: PhantomData,
        }
    }

    fn read_bytes(&self, from_position: usize, byte_count: usize) -> Result<M256, OnChainError> {
        if from_position > self.code.len() {
            return Err(OnChainError::PCOverflow);
        }
        let position = from_position;
        let max = min(position.saturating_add(byte_count), self.code.len());
        Ok(M256::from(&self.code[position..max]))
    }

    /// Get the code bytearray.
    pub fn code(&self) -> &[u8] {
        &self.code
    }

    /// Jump to a position in the code. The destination must be valid
    /// to jump.
    pub fn jump(&mut self, position: usize) -> Result<(), OnChainError> {
        if position >= self.code.len() {
            return Err(OnChainError::PCOverflow);
        }

        if !self.is_valid(position) {
            return Err(OnChainError::BadJumpDest);
        }

        *self.position = position;
        Ok(())
    }

    /// Get the current program counter position.
    pub fn position(&self) -> usize {
        *self.position
    }

    /// Get the current opcode position. Should only be used when debugging.
    pub fn opcode_position(&self) -> usize {
        let mut o = 0;
        let mut i = 0;
        while i <= *self.position {
            let opcode: Opcode = self.code[i].into();
            match opcode {
                Opcode::PUSH(v) => {
                    i = i + v + 1;
                },
                _ => {
                    i = i + 1;
                }
            }
            o = o + 1;
        }
        o
    }

    /// Returns `true` if the position is a valid jump destination. If
    /// not, returns `false`.
    pub fn is_valid(&self, position: usize) -> bool {
        self.valids.is_valid(position)
    }

    /// Check whether the PC is ended. Next `read` on this PC would
    /// result in `PCError::PCOverflow`.
    pub fn is_end(&self) -> bool {
        *self.position == self.code.len()
    }

    /// Peek the next instruction.
    pub fn peek(&self) -> Result<Instruction, OnChainError> {
        if *self.position >= self.code.len() {
            return Err(OnChainError::PCOverflow);
        }
        let opcode: Opcode = self.code[*self.position].into();
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
                let param = self.read_bytes(*self.position + 1, v)?;
                Instruction::PUSH(param)
            },

            Opcode::DUP(v) => Instruction::DUP(v),
            Opcode::SWAP(v) => Instruction::SWAP(v),
            Opcode::LOG(v) => Instruction::LOG(v),

            Opcode::CREATE => Instruction::CREATE,
            Opcode::CALL => Instruction::CALL,
            Opcode::CALLCODE => Instruction::CALLCODE,
            Opcode::RETURN => Instruction::RETURN,
            Opcode::DELEGATECALL => {
                if P::has_delegate_call() {
                    Instruction::DELEGATECALL
                } else {
                    return Err(OnChainError::InvalidOpcode);
                }
            },

            Opcode::INVALID => {
                return Err(OnChainError::InvalidOpcode);
            },
            Opcode::SUICIDE => Instruction::SUICIDE,
        })
    }

    /// Read the next instruction and step the program counter.
    pub fn read(&mut self) -> Result<Instruction, OnChainError> {
        let result = self.peek()?;
        let opcode: Opcode = self.code[*self.position].into();
        match opcode {
            Opcode::PUSH(v) => {
                *self.position = min(*self.position + v + 1, self.code.len());
            },
            _ => {
                *self.position = *self.position + 1;
            },
        }
        Ok(result)
    }
}
