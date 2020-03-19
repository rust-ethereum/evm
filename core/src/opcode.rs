/// Opcode enum. One-to-one corresponding to an `u8` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Opcode {
	/// `STOP`
	Stop,
	/// `ADD`
	Add,
	/// `MUL`
	Mul,
	/// `SUB`
	Sub,
	/// `DIV`
	Div,
	/// `SDIV`
	SDiv,
	/// `MOD`
	Mod,
	/// `SMOD`
	SMod,
	/// `ADDMOD`
	AddMod,
	/// `MULMOD`
	MulMod,
	/// `EXP`
	Exp,
	/// `SIGNEXTEND`
	SignExtend,

	/// `LT`
	Lt,
	/// `GT`
	Gt,
	/// `SLT`
	SLt,
	/// `SGT`
	SGt,
	/// `EQ`
	Eq,
	/// `ISZERO`
	IsZero,
	/// `AND`
	And,
	/// `OR`
	Or,
	/// `XOR`
	Xor,
	/// `NOT`
	Not,
	/// `BYTE`
	Byte,

	/// `CALLDATALOAD`
	CallDataLoad,
	/// `CALLDATASIZE`
	CallDataSize,
	/// `CALLDATACOPY`
	CallDataCopy,
	/// `CODESIZE`
	CodeSize,
	/// `CODECOPY`
	CodeCopy,

	/// `SHL`
	Shl,
	/// `SHR`
	Shr,
	/// `SAR`
	Sar,

	/// `POP`
	Pop,
	/// `MLOAD`
	MLoad,
	/// `MSTORE`
	MStore,
	/// `MSTORE8`
	MStore8,
	/// `JUMP`
	Jump,
	/// `JUMPI`
	JumpI,
	/// `PC`
	PC,
	/// `MSIZE`
	MSize,
	/// `JUMPDEST`
	JumpDest,

	/// `PUSHn`
	Push(u8),
	/// `DUPn`
	Dup(u8),
	/// `SWAPn`
	Swap(u8),

	/// `RETURN`
	Return,
	/// `REVERT`
	Revert,

	/// `INVALID`
	Invalid,
}

impl Opcode {
	/// Parse a byte into an opcode.
	pub fn parse(opcode: u8) -> Result<Opcode, ExternalOpcode> {
		match opcode {
			0x00 => Ok(Opcode::Stop),
			0x01 => Ok(Opcode::Add),
			0x02 => Ok(Opcode::Mul),
			0x03 => Ok(Opcode::Sub),
			0x04 => Ok(Opcode::Div),
			0x05 => Ok(Opcode::SDiv),
			0x06 => Ok(Opcode::Mod),
			0x07 => Ok(Opcode::SMod),
			0x08 => Ok(Opcode::AddMod),
			0x09 => Ok(Opcode::MulMod),
			0x0a => Ok(Opcode::Exp),
			0x0b => Ok(Opcode::SignExtend),

			0x10 => Ok(Opcode::Lt),
			0x11 => Ok(Opcode::Gt),
			0x12 => Ok(Opcode::SLt),
			0x13 => Ok(Opcode::SGt),
			0x14 => Ok(Opcode::Eq),
			0x15 => Ok(Opcode::IsZero),
			0x16 => Ok(Opcode::And),
			0x17 => Ok(Opcode::Or),
			0x18 => Ok(Opcode::Xor),
			0x19 => Ok(Opcode::Not),
			0x1a => Ok(Opcode::Byte),

			0x1b => Ok(Opcode::Shl),
			0x1c => Ok(Opcode::Shr),
			0x1d => Ok(Opcode::Sar),

			0x20 => Err(ExternalOpcode::Sha3),

			0x30 => Err(ExternalOpcode::Address),
			0x31 => Err(ExternalOpcode::Balance),
			0x32 => Err(ExternalOpcode::Origin),
			0x33 => Err(ExternalOpcode::Caller),
			0x34 => Err(ExternalOpcode::CallValue),
			0x35 => Ok(Opcode::CallDataLoad),
			0x36 => Ok(Opcode::CallDataSize),
			0x37 => Ok(Opcode::CallDataCopy),
			0x38 => Ok(Opcode::CodeSize),
			0x39 => Ok(Opcode::CodeCopy),
			0x3a => Err(ExternalOpcode::GasPrice),
			0x3b => Err(ExternalOpcode::ExtCodeSize),
			0x3c => Err(ExternalOpcode::ExtCodeCopy),
			0x3d => Err(ExternalOpcode::ReturnDataSize),
			0x3e => Err(ExternalOpcode::ReturnDataCopy),
			0x3f => Err(ExternalOpcode::ExtCodeHash),

			0x40 => Err(ExternalOpcode::BlockHash),
			0x41 => Err(ExternalOpcode::Coinbase),
			0x42 => Err(ExternalOpcode::Timestamp),
			0x43 => Err(ExternalOpcode::Number),
			0x44 => Err(ExternalOpcode::Difficulty),
			0x45 => Err(ExternalOpcode::GasLimit),
			0x46 => Err(ExternalOpcode::ChainId),
			0x47 => Err(ExternalOpcode::SelfBalance),

			0x50 => Ok(Opcode::Pop),
			0x51 => Ok(Opcode::MLoad),
			0x52 => Ok(Opcode::MStore),
			0x53 => Ok(Opcode::MStore8),
			0x54 => Err(ExternalOpcode::SLoad),
			0x55 => Err(ExternalOpcode::SStore),
			0x56 => Ok(Opcode::Jump),
			0x57 => Ok(Opcode::JumpI),
			0x58 => Ok(Opcode::PC),
			0x59 => Ok(Opcode::MSize),
			0x5a => Err(ExternalOpcode::Gas),
			0x5b => Ok(Opcode::JumpDest),

			0x60 => Ok(Opcode::Push(1)),
			0x61 => Ok(Opcode::Push(2)),
			0x62 => Ok(Opcode::Push(3)),
			0x63 => Ok(Opcode::Push(4)),
			0x64 => Ok(Opcode::Push(5)),
			0x65 => Ok(Opcode::Push(6)),
			0x66 => Ok(Opcode::Push(7)),
			0x67 => Ok(Opcode::Push(8)),
			0x68 => Ok(Opcode::Push(9)),
			0x69 => Ok(Opcode::Push(10)),
			0x6a => Ok(Opcode::Push(11)),
			0x6b => Ok(Opcode::Push(12)),
			0x6c => Ok(Opcode::Push(13)),
			0x6d => Ok(Opcode::Push(14)),
			0x6e => Ok(Opcode::Push(15)),
			0x6f => Ok(Opcode::Push(16)),
			0x70 => Ok(Opcode::Push(17)),
			0x71 => Ok(Opcode::Push(18)),
			0x72 => Ok(Opcode::Push(19)),
			0x73 => Ok(Opcode::Push(20)),
			0x74 => Ok(Opcode::Push(21)),
			0x75 => Ok(Opcode::Push(22)),
			0x76 => Ok(Opcode::Push(23)),
			0x77 => Ok(Opcode::Push(24)),
			0x78 => Ok(Opcode::Push(25)),
			0x79 => Ok(Opcode::Push(26)),
			0x7a => Ok(Opcode::Push(27)),
			0x7b => Ok(Opcode::Push(28)),
			0x7c => Ok(Opcode::Push(29)),
			0x7d => Ok(Opcode::Push(30)),
			0x7e => Ok(Opcode::Push(31)),
			0x7f => Ok(Opcode::Push(32)),

			0x80 => Ok(Opcode::Dup(1)),
			0x81 => Ok(Opcode::Dup(2)),
			0x82 => Ok(Opcode::Dup(3)),
			0x83 => Ok(Opcode::Dup(4)),
			0x84 => Ok(Opcode::Dup(5)),
			0x85 => Ok(Opcode::Dup(6)),
			0x86 => Ok(Opcode::Dup(7)),
			0x87 => Ok(Opcode::Dup(8)),
			0x88 => Ok(Opcode::Dup(9)),
			0x89 => Ok(Opcode::Dup(10)),
			0x8a => Ok(Opcode::Dup(11)),
			0x8b => Ok(Opcode::Dup(12)),
			0x8c => Ok(Opcode::Dup(13)),
			0x8d => Ok(Opcode::Dup(14)),
			0x8e => Ok(Opcode::Dup(15)),
			0x8f => Ok(Opcode::Dup(16)),

			0x90 => Ok(Opcode::Swap(1)),
			0x91 => Ok(Opcode::Swap(2)),
			0x92 => Ok(Opcode::Swap(3)),
			0x93 => Ok(Opcode::Swap(4)),
			0x94 => Ok(Opcode::Swap(5)),
			0x95 => Ok(Opcode::Swap(6)),
			0x96 => Ok(Opcode::Swap(7)),
			0x97 => Ok(Opcode::Swap(8)),
			0x98 => Ok(Opcode::Swap(9)),
			0x99 => Ok(Opcode::Swap(10)),
			0x9a => Ok(Opcode::Swap(11)),
			0x9b => Ok(Opcode::Swap(12)),
			0x9c => Ok(Opcode::Swap(13)),
			0x9d => Ok(Opcode::Swap(14)),
			0x9e => Ok(Opcode::Swap(15)),
			0x9f => Ok(Opcode::Swap(16)),

			0xa0 => Err(ExternalOpcode::Log(0)),
			0xa1 => Err(ExternalOpcode::Log(1)),
			0xa2 => Err(ExternalOpcode::Log(2)),
			0xa3 => Err(ExternalOpcode::Log(3)),
			0xa4 => Err(ExternalOpcode::Log(4)),

			0xf0 => Err(ExternalOpcode::Create),
			0xf1 => Err(ExternalOpcode::Call),
			0xf2 => Err(ExternalOpcode::CallCode),
			0xf3 => Ok(Opcode::Return),
			0xf4 => Err(ExternalOpcode::DelegateCall),
			0xf5 => Err(ExternalOpcode::Create2),
			0xfa => Err(ExternalOpcode::StaticCall),
			0xfd => Ok(Opcode::Revert),

			0xff => Err(ExternalOpcode::Suicide),
			other => Err(ExternalOpcode::Other(other)),
		}
	}
}

/// External opcodes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalOpcode {
	/// `SHA3`
	Sha3,
	/// `ADDRESS`
	Address,
	/// `BALANCE`
	Balance,
	/// `SELFBALANCE`
	SelfBalance,
	/// `ORIGIN`
	Origin,
	/// `CALLER`
	Caller,
	/// `CALLVALUE`
	CallValue,
	/// `GASPRICE`
	GasPrice,
	/// `EXTCODESIZE`
	ExtCodeSize,
	/// `EXTCODECOPY`
	ExtCodeCopy,
	/// `EXTCODEHASH`
	ExtCodeHash,
	/// `RETURNDATASIZE`
	ReturnDataSize,
	/// `RETURNDATACOPY`
	ReturnDataCopy,
	/// `BLOCKHASH`
	BlockHash,
	/// `COINBASE`
	Coinbase,
	/// `TIMESTAMP`
	Timestamp,
	/// `NUMBER`
	Number,
	/// `DIFFICULTY`
	Difficulty,
	/// `GASLIMIT`
	GasLimit,
	/// `SLOAD`
	SLoad,
	/// `SSTORE`
	SStore,
	/// `GAS`
	Gas,
	/// `LOGn`
	Log(u8),
	/// `CREATE`
	Create,
	/// `CREATE2`
	Create2,
	/// `CALL`
	Call,
	/// `CALLCODE`
	CallCode,
	/// `DELEGATECALL`
	DelegateCall,
	/// `STATICCALL`
	StaticCall,
	/// `SUICIDE`
	Suicide,
	/// `CHAINID`
	ChainId,
	/// Other unknown opcodes.
	Other(u8),
}
