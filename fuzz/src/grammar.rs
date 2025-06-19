// Copyright 2025 Security Research Labs GmbH
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
#![allow(clippy::upper_case_acronyms)]

const CONTRACT_TWO_ADDRESS: &str = "83769beeb7e5405ef0b7dc3c66c43e3a51a6d27f";

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct FuzzData {
	pub contract_one: Vec<Opcode>,
	pub contract_two: Vec<Opcode>,
	pub call_data: Vec<u8>,
}

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub enum Opcode {
	STOP,
	ADD,
	MUL,
	SUB,
	DIV,
	SDIV,
	MOD,
	SMOD,
	ADDMOD,
	MULMOD,
	EXP,
	SIGNEXTEND,
	LT,
	GT,
	SLT,
	SGT,
	EQ,
	ISZERO,
	AND,
	OR,
	XOR,
	NOT,
	BYTE,
	SHL,
	SHR,
	SAR,
	SHA3,
	ADDRESS,
	BALANCE,
	ORIGIN,
	CALLER,
	CALLVALUE,
	CALLDATALOAD,
	CALLDATASIZE,
	CALLDATACOPY,
	CODESIZE,
	CODECOPY,
	GASPRICE,
	EXTCODESIZE,
	EXTCODECOPY,
	RETURNDATASIZE,
	RETURNDATACOPY,
	EXTCODEHASH,
	BLOCKHASH,
	COINBASE,
	TIMESTAMP,
	NUMBER,
	DIFFICULTY,
	GASLIMIT,
	CHAINID,
	SELFBALANCE,
	BASEFEE,
	POP,
	MLOAD,
	MSTORE,
	MSTORE8,
	SLOAD,
	SSTORE,
	JUMP,
	JUMPI,
	PC,
	MSIZE,
	GAS,
	JUMPDEST,
	MCOPY,
	TLOAD,
	TSTORE,
	PUSH0,
	PUSH1([u8; 1]),
	PUSH2([u8; 2]),
	PUSH3([u8; 3]),
	PUSH4([u8; 4]),
	PUSH5([u8; 5]),
	PUSH6([u8; 6]),
	PUSH7([u8; 7]),
	PUSH8([u8; 8]),
	PUSH9([u8; 9]),
	PUSH10([u8; 10]),
	PUSH11([u8; 11]),
	PUSH12([u8; 12]),
	PUSH13([u8; 13]),
	PUSH14([u8; 14]),
	PUSH15([u8; 15]),
	PUSH16([u8; 16]),
	PUSH17([u8; 17]),
	PUSH18([u8; 18]),
	PUSH19([u8; 19]),
	PUSH20([u8; 20]),
	PUSH21([u8; 21]),
	PUSH22([u8; 22]),
	PUSH23([u8; 23]),
	PUSH24([u8; 24]),
	PUSH25([u8; 25]),
	PUSH26([u8; 26]),
	PUSH27([u8; 27]),
	PUSH28([u8; 28]),
	PUSH29([u8; 29]),
	PUSH30([u8; 30]),
	PUSH31([u8; 31]),
	PUSH32([u8; 32]),
	DUP1,
	DUP2,
	DUP3,
	DUP4,
	DUP5,
	DUP6,
	DUP7,
	DUP8,
	DUP9,
	DUP10,
	DUP11,
	DUP12,
	DUP13,
	DUP14,
	DUP15,
	DUP16,
	SWAP1,
	SWAP2,
	SWAP3,
	SWAP4,
	SWAP5,
	SWAP6,
	SWAP7,
	SWAP8,
	SWAP9,
	SWAP10,
	SWAP11,
	SWAP12,
	SWAP13,
	SWAP14,
	SWAP15,
	SWAP16,
	LOG0,
	LOG1,
	LOG2,
	LOG3,
	LOG4,
	CREATE,
	CALL,
	CALLCODE,
	RETURN,
	DELEGATECALL,
	CREATE2,
	STATICCALL,
	REVERT,
	INVALID,
	SELFDESTRUCT,
	BLOBBASEFEE,
	BLOBHASH,
	FuzzCall,
}

impl Opcode {
	pub fn to_bytes(&self, output: &mut Vec<u8>) {
		match self {
			Opcode::FuzzCall => {
				output.push(0x73);
				output.extend_from_slice(&hex::decode(CONTRACT_TWO_ADDRESS).unwrap());
			}
			Opcode::STOP => output.push(0x00),
			Opcode::ADD => output.push(0x01),
			Opcode::MUL => output.push(0x02),
			Opcode::SUB => output.push(0x03),
			Opcode::DIV => output.push(0x04),
			Opcode::SDIV => output.push(0x05),
			Opcode::MOD => output.push(0x06),
			Opcode::SMOD => output.push(0x07),
			Opcode::ADDMOD => output.push(0x08),
			Opcode::MULMOD => output.push(0x09),
			Opcode::EXP => output.push(0x0A),
			Opcode::SIGNEXTEND => output.push(0x0B),
			Opcode::LT => output.push(0x10),
			Opcode::GT => output.push(0x11),
			Opcode::SLT => output.push(0x12),
			Opcode::SGT => output.push(0x13),
			Opcode::EQ => output.push(0x14),
			Opcode::ISZERO => output.push(0x15),
			Opcode::AND => output.push(0x16),
			Opcode::OR => output.push(0x17),
			Opcode::XOR => output.push(0x18),
			Opcode::NOT => output.push(0x19),
			Opcode::BYTE => output.push(0x1A),
			Opcode::SHL => output.push(0x1B),
			Opcode::SHR => output.push(0x1C),
			Opcode::SAR => output.push(0x1D),
			Opcode::SHA3 => output.push(0x20),
			Opcode::ADDRESS => output.push(0x30),
			Opcode::BALANCE => output.push(0x31),
			Opcode::ORIGIN => output.push(0x32),
			Opcode::CALLER => output.push(0x33),
			Opcode::CALLVALUE => output.push(0x34),
			Opcode::CALLDATALOAD => output.push(0x35),
			Opcode::CALLDATASIZE => output.push(0x36),
			Opcode::CALLDATACOPY => output.push(0x37),
			Opcode::CODESIZE => output.push(0x38),
			Opcode::CODECOPY => output.push(0x39),
			Opcode::GASPRICE => output.push(0x3A),
			Opcode::EXTCODESIZE => output.push(0x3B),
			Opcode::EXTCODECOPY => output.push(0x3C),
			Opcode::RETURNDATASIZE => output.push(0x3D),
			Opcode::RETURNDATACOPY => output.push(0x3E),
			Opcode::EXTCODEHASH => output.push(0x3F),
			Opcode::BLOCKHASH => output.push(0x40),
			Opcode::COINBASE => output.push(0x41),
			Opcode::TIMESTAMP => output.push(0x42),
			Opcode::NUMBER => output.push(0x43),
			Opcode::DIFFICULTY => output.push(0x44),
			Opcode::GASLIMIT => output.push(0x45),
			Opcode::CHAINID => output.push(0x46),
			Opcode::SELFBALANCE => output.push(0x47),
			Opcode::BASEFEE => output.push(0x48),
			Opcode::POP => output.push(0x50),
			Opcode::MLOAD => output.push(0x51),
			Opcode::MSTORE => output.push(0x52),
			Opcode::MSTORE8 => output.push(0x53),
			Opcode::SLOAD => output.push(0x54),
			Opcode::SSTORE => output.push(0x55),
			Opcode::JUMP => output.push(0x56),
			Opcode::JUMPI => output.push(0x57),
			Opcode::PC => output.push(0x58),
			Opcode::MSIZE => output.push(0x59),
			Opcode::GAS => output.push(0x5A),
			Opcode::JUMPDEST => output.push(0x5B),
			Opcode::MCOPY => output.push(0x5C),
			Opcode::TLOAD => output.push(0x5D),
			Opcode::TSTORE => output.push(0x5E),

			// Handle PUSH operations - add both the opcode and the pushed bytes
			Opcode::PUSH0 => {
				output.push(0x5F);
				output.push(0x00);
			}
			Opcode::PUSH1(data) => {
				output.push(0x60);
				output.extend_from_slice(data);
			}
			Opcode::PUSH2(data) => {
				output.push(0x61);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH3(data) => {
				output.push(0x62);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH4(data) => {
				output.push(0x63);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH5(data) => {
				output.push(0x64);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH6(data) => {
				output.push(0x65);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH7(data) => {
				output.push(0x66);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH8(data) => {
				output.push(0x67);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH9(data) => {
				output.push(0x68);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH10(data) => {
				output.push(0x69);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH11(data) => {
				output.push(0x6A);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH12(data) => {
				output.push(0x6B);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH13(data) => {
				output.push(0x6C);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH14(data) => {
				output.push(0x6D);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH15(data) => {
				output.push(0x6E);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH16(data) => {
				output.push(0x6F);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH17(data) => {
				output.push(0x70);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH18(data) => {
				output.push(0x71);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH19(data) => {
				output.push(0x72);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH20(data) => {
				output.push(0x73);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH21(data) => {
				output.push(0x74);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH22(data) => {
				output.push(0x75);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH23(data) => {
				output.push(0x76);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH24(data) => {
				output.push(0x77);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH25(data) => {
				output.push(0x78);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH26(data) => {
				output.push(0x79);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH27(data) => {
				output.push(0x7A);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH28(data) => {
				output.push(0x7B);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH29(data) => {
				output.push(0x7C);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH30(data) => {
				output.push(0x7D);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH31(data) => {
				output.push(0x7E);
				output.extend_from_slice(data.as_slice());
			}
			Opcode::PUSH32(data) => {
				output.push(0x7F);
				output.extend_from_slice(data.as_slice());
			}

			// Remaining opcodes
			Opcode::DUP1 => output.push(0x80),
			Opcode::DUP2 => output.push(0x81),
			Opcode::DUP3 => output.push(0x82),
			Opcode::DUP4 => output.push(0x83),
			Opcode::DUP5 => output.push(0x84),
			Opcode::DUP6 => output.push(0x85),
			Opcode::DUP7 => output.push(0x86),
			Opcode::DUP8 => output.push(0x87),
			Opcode::DUP9 => output.push(0x88),
			Opcode::DUP10 => output.push(0x89),
			Opcode::DUP11 => output.push(0x8A),
			Opcode::DUP12 => output.push(0x8B),
			Opcode::DUP13 => output.push(0x8C),
			Opcode::DUP14 => output.push(0x8D),
			Opcode::DUP15 => output.push(0x8E),
			Opcode::DUP16 => output.push(0x8F),
			Opcode::SWAP1 => output.push(0x90),
			Opcode::SWAP2 => output.push(0x91),
			Opcode::SWAP3 => output.push(0x92),
			Opcode::SWAP4 => output.push(0x93),
			Opcode::SWAP5 => output.push(0x94),
			Opcode::SWAP6 => output.push(0x95),
			Opcode::SWAP7 => output.push(0x96),
			Opcode::SWAP8 => output.push(0x97),
			Opcode::SWAP9 => output.push(0x98),
			Opcode::SWAP10 => output.push(0x99),
			Opcode::SWAP11 => output.push(0x9A),
			Opcode::SWAP12 => output.push(0x9B),
			Opcode::SWAP13 => output.push(0x9C),
			Opcode::SWAP14 => output.push(0x9D),
			Opcode::SWAP15 => output.push(0x9E),
			Opcode::SWAP16 => output.push(0x9F),
			Opcode::LOG0 => output.push(0xA0),
			Opcode::LOG1 => output.push(0xA1),
			Opcode::LOG2 => output.push(0xA2),
			Opcode::LOG3 => output.push(0xA3),
			Opcode::LOG4 => output.push(0xA4),
			Opcode::CREATE => output.push(0xF0),
			Opcode::CALL => output.push(0xF1),
			Opcode::CALLCODE => output.push(0xF2),
			Opcode::RETURN => output.push(0xF3),
			Opcode::DELEGATECALL => output.push(0xF4),
			Opcode::CREATE2 => output.push(0xF5),
			Opcode::STATICCALL => output.push(0xFA),
			Opcode::REVERT => output.push(0xFD),
			Opcode::INVALID => output.push(0xFE),
			Opcode::SELFDESTRUCT => output.push(0xFF),
			Opcode::BLOBBASEFEE => output.push(0x4A),
			Opcode::BLOBHASH => output.push(0x49),
		}
	}
}
