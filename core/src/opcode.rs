/// Opcode enum. One-to-one corresponding to an `u8` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Opcode {
	/// `STOP`
	Stop = 0x00,
	/// `ADD`
	Add = 0x01,
	/// `MUL`
	Mul = 0x02,
	/// `SUB`
	Sub = 0x03,
	/// `DIV`
	Div = 0x04,
	/// `SDIV`
	SDiv = 0x05,
	/// `MOD`
	Mod = 0x06,
	/// `SMOD`
	SMod = 0x07,
	/// `ADDMOD`
	AddMod = 0x08,
	/// `MULMOD`
	MulMod = 0x09,
	/// `EXP`
	Exp = 0x0a,
	/// `SIGNEXTEND`
	SignExtend = 0x0b,

	/// `LT`
	Lt = 0x10,
	/// `GT`
	Gt = 0x11,
	/// `SLT`
	SLt = 0x12,
	/// `SGT`
	SGt = 0x13,
	/// `EQ`
	Eq = 0x14,
	/// `ISZERO`
	IsZero = 0x15,
	/// `AND`
	And = 0x16,
	/// `OR`
	Or = 0x17,
	/// `XOR`
	Xor = 0x18,
	/// `NOT`
	Not = 0x19,
	/// `BYTE`
	Byte = 0x1a,

	/// `CALLDATALOAD`
	CallDataLoad = 0x35,
	/// `CALLDATASIZE`
	CallDataSize = 0x36,
	/// `CALLDATACOPY`
	CallDataCopy = 0x37,
	/// `CODESIZE`
	CodeSize = 0x38,
	/// `CODECOPY`
	CodeCopy = 0x39,

	/// `SHL`
	Shl = 0x1b,
	/// `SHR`
	Shr = 0x1c,
	/// `SAR`
	Sar = 0x1d,

	/// `POP`
	Pop = 0x50,
	/// `MLOAD`
	MLoad = 0x51,
	/// `MSTORE`
	MStore = 0x52,
	/// `MSTORE8`
	MStore8 = 0x53,
	/// `JUMP`
	Jump = 0x56,
	/// `JUMPI`
	JumpI = 0x57,
	/// `PC`
	PC = 0x58,
	/// `MSIZE`
	MSize = 0x59,
	/// `JUMPDEST`
	JumpDest = 0x5b,

	/// `PUSHn`
	Push1 = 0x60,
	Push2 = 0x61,
	Push3 = 0x62,
	Push4 = 0x63,
	Push5 = 0x64,
	Push6 = 0x65,
	Push7 = 0x66,
	Push8 = 0x67,
	Push9 = 0x68,
	Push10 = 0x69,
	Push11 = 0x6a,
	Push12 = 0x6b,
	Push13 = 0x6c,
	Push14 = 0x6d,
	Push15 = 0x6e,
	Push16 = 0x6f,
	Push17 = 0x70,
	Push18 = 0x71,
	Push19 = 0x72,
	Push20 = 0x73,
	Push21 = 0x74,
	Push22 = 0x75,
	Push23 = 0x76,
	Push24 = 0x77,
	Push25 = 0x78,
	Push26 = 0x79,
	Push27 = 0x7a,
	Push28 = 0x7b,
	Push29 = 0x7c,
	Push30 = 0x7d,
	Push31 = 0x7e,
	Push32 = 0x7f,

	/// `DUPn`
	Dup1 = 0x80,
	Dup2 = 0x81,
	Dup3 = 0x82,
	Dup4 = 0x83,
	Dup5 = 0x84,
	Dup6 = 0x85,
	Dup7 = 0x86,
	Dup8 = 0x87,
	Dup9 = 0x88,
	Dup10 = 0x89,
	Dup11 = 0x8a,
	Dup12 = 0x8b,
	Dup13 = 0x8c,
	Dup14 = 0x8d,
	Dup15 = 0x8e,
	Dup16 = 0x8f,

	/// `SWAPn`
	Swap1 = 0x90,
	Swap2 = 0x91,
	Swap3 = 0x92,
	Swap4 = 0x93,
	Swap5 = 0x94,
	Swap6 = 0x95,
	Swap7 = 0x96,
	Swap8 = 0x97,
	Swap9 = 0x98,
	Swap10 = 0x99,
	Swap11 = 0x9a,
	Swap12 = 0x9b,
	Swap13 = 0x9c,
	Swap14 = 0x9d,
	Swap15 = 0x9e,
	Swap16 = 0x9f,

	/// `RETURN`
	Return = 0xf3,
	/// `REVERT`
	Revert = 0xfd,

	/// `INVALID`
	Invalid = 0xfe,
}

impl Opcode {
	/// Whether the opcode is a push opcode.
	pub fn is_push(&self) -> Option<u8> {
		let value = *self as u8;
		if value >= 0x60 && value <= 0x7f {
			Some(value - 0x60 + 1)
		} else {
			None
		}
	}

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
			0x0c => Err(ExternalOpcode::Other0c),
			0x0d => Err(ExternalOpcode::Other0d),
			0x0e => Err(ExternalOpcode::Other0e),
			0x0f => Err(ExternalOpcode::Other0f),

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
			0x1e => Err(ExternalOpcode::Other1e),
			0x1f => Err(ExternalOpcode::Other1f),

			0x20 => Err(ExternalOpcode::Sha3),
			0x21 => Err(ExternalOpcode::Other21),
			0x22 => Err(ExternalOpcode::Other22),
			0x23 => Err(ExternalOpcode::Other23),
			0x24 => Err(ExternalOpcode::Other24),
			0x25 => Err(ExternalOpcode::Other25),
			0x26 => Err(ExternalOpcode::Other26),
			0x27 => Err(ExternalOpcode::Other27),
			0x28 => Err(ExternalOpcode::Other28),
			0x29 => Err(ExternalOpcode::Other29),
			0x2a => Err(ExternalOpcode::Other2a),
			0x2b => Err(ExternalOpcode::Other2b),
			0x2c => Err(ExternalOpcode::Other2c),
			0x2d => Err(ExternalOpcode::Other2d),
			0x2e => Err(ExternalOpcode::Other2e),
			0x2f => Err(ExternalOpcode::Other2f),

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
			0x48 => Err(ExternalOpcode::Other48),
			0x49 => Err(ExternalOpcode::Other49),
			0x4a => Err(ExternalOpcode::Other4a),
			0x4b => Err(ExternalOpcode::Other4b),
			0x4c => Err(ExternalOpcode::Other4c),
			0x4d => Err(ExternalOpcode::Other4d),
			0x4e => Err(ExternalOpcode::Other4e),
			0x4f => Err(ExternalOpcode::Other4f),

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
			0x5c => Err(ExternalOpcode::Other5c),
			0x5d => Err(ExternalOpcode::Other5d),
			0x5e => Err(ExternalOpcode::Other5e),
			0x5f => Err(ExternalOpcode::Other5f),

			0x60 => Ok(Opcode::Push1),
			0x61 => Ok(Opcode::Push2),
			0x62 => Ok(Opcode::Push3),
			0x63 => Ok(Opcode::Push4),
			0x64 => Ok(Opcode::Push5),
			0x65 => Ok(Opcode::Push6),
			0x66 => Ok(Opcode::Push7),
			0x67 => Ok(Opcode::Push8),
			0x68 => Ok(Opcode::Push9),
			0x69 => Ok(Opcode::Push10),
			0x6a => Ok(Opcode::Push11),
			0x6b => Ok(Opcode::Push12),
			0x6c => Ok(Opcode::Push13),
			0x6d => Ok(Opcode::Push14),
			0x6e => Ok(Opcode::Push15),
			0x6f => Ok(Opcode::Push16),

			0x70 => Ok(Opcode::Push17),
			0x71 => Ok(Opcode::Push18),
			0x72 => Ok(Opcode::Push19),
			0x73 => Ok(Opcode::Push20),
			0x74 => Ok(Opcode::Push21),
			0x75 => Ok(Opcode::Push22),
			0x76 => Ok(Opcode::Push23),
			0x77 => Ok(Opcode::Push24),
			0x78 => Ok(Opcode::Push25),
			0x79 => Ok(Opcode::Push26),
			0x7a => Ok(Opcode::Push27),
			0x7b => Ok(Opcode::Push28),
			0x7c => Ok(Opcode::Push29),
			0x7d => Ok(Opcode::Push30),
			0x7e => Ok(Opcode::Push31),
			0x7f => Ok(Opcode::Push32),

			0x80 => Ok(Opcode::Dup1),
			0x81 => Ok(Opcode::Dup2),
			0x82 => Ok(Opcode::Dup3),
			0x83 => Ok(Opcode::Dup4),
			0x84 => Ok(Opcode::Dup5),
			0x85 => Ok(Opcode::Dup6),
			0x86 => Ok(Opcode::Dup7),
			0x87 => Ok(Opcode::Dup8),
			0x88 => Ok(Opcode::Dup9),
			0x89 => Ok(Opcode::Dup10),
			0x8a => Ok(Opcode::Dup11),
			0x8b => Ok(Opcode::Dup12),
			0x8c => Ok(Opcode::Dup13),
			0x8d => Ok(Opcode::Dup14),
			0x8e => Ok(Opcode::Dup15),
			0x8f => Ok(Opcode::Dup16),

			0x90 => Ok(Opcode::Swap1),
			0x91 => Ok(Opcode::Swap2),
			0x92 => Ok(Opcode::Swap3),
			0x93 => Ok(Opcode::Swap4),
			0x94 => Ok(Opcode::Swap5),
			0x95 => Ok(Opcode::Swap6),
			0x96 => Ok(Opcode::Swap7),
			0x97 => Ok(Opcode::Swap8),
			0x98 => Ok(Opcode::Swap9),
			0x99 => Ok(Opcode::Swap10),
			0x9a => Ok(Opcode::Swap11),
			0x9b => Ok(Opcode::Swap12),
			0x9c => Ok(Opcode::Swap13),
			0x9d => Ok(Opcode::Swap14),
			0x9e => Ok(Opcode::Swap15),
			0x9f => Ok(Opcode::Swap16),

			0xa0 => Err(ExternalOpcode::Log0),
			0xa1 => Err(ExternalOpcode::Log1),
			0xa2 => Err(ExternalOpcode::Log2),
			0xa3 => Err(ExternalOpcode::Log3),
			0xa4 => Err(ExternalOpcode::Log4),
			0xa5 => Err(ExternalOpcode::Othera5),
			0xa6 => Err(ExternalOpcode::Othera6),
			0xa7 => Err(ExternalOpcode::Othera7),
			0xa8 => Err(ExternalOpcode::Othera8),
			0xa9 => Err(ExternalOpcode::Othera9),
			0xaa => Err(ExternalOpcode::Otheraa),
			0xab => Err(ExternalOpcode::Otherab),
			0xac => Err(ExternalOpcode::Otherac),
			0xad => Err(ExternalOpcode::Otherad),
			0xae => Err(ExternalOpcode::Otherae),
			0xaf => Err(ExternalOpcode::Otheraf),

			0xb0 => Err(ExternalOpcode::Otherb0),
			0xb1 => Err(ExternalOpcode::Otherb1),
			0xb2 => Err(ExternalOpcode::Otherb2),
			0xb3 => Err(ExternalOpcode::Otherb3),
			0xb4 => Err(ExternalOpcode::Otherb4),
			0xb5 => Err(ExternalOpcode::Otherb5),
			0xb6 => Err(ExternalOpcode::Otherb6),
			0xb7 => Err(ExternalOpcode::Otherb7),
			0xb8 => Err(ExternalOpcode::Otherb8),
			0xb9 => Err(ExternalOpcode::Otherb9),
			0xba => Err(ExternalOpcode::Otherba),
			0xbb => Err(ExternalOpcode::Otherbb),
			0xbc => Err(ExternalOpcode::Otherbc),
			0xbd => Err(ExternalOpcode::Otherbd),
			0xbe => Err(ExternalOpcode::Otherbe),
			0xbf => Err(ExternalOpcode::Otherbf),

			0xc0 => Err(ExternalOpcode::Otherc0),
			0xc1 => Err(ExternalOpcode::Otherc1),
			0xc2 => Err(ExternalOpcode::Otherc2),
			0xc3 => Err(ExternalOpcode::Otherc3),
			0xc4 => Err(ExternalOpcode::Otherc4),
			0xc5 => Err(ExternalOpcode::Otherc5),
			0xc6 => Err(ExternalOpcode::Otherc6),
			0xc7 => Err(ExternalOpcode::Otherc7),
			0xc8 => Err(ExternalOpcode::Otherc8),
			0xc9 => Err(ExternalOpcode::Otherc9),
			0xca => Err(ExternalOpcode::Otherca),
			0xcb => Err(ExternalOpcode::Othercb),
			0xcc => Err(ExternalOpcode::Othercc),
			0xcd => Err(ExternalOpcode::Othercd),
			0xce => Err(ExternalOpcode::Otherce),
			0xcf => Err(ExternalOpcode::Othercf),

			0xd0 => Err(ExternalOpcode::Otherd0),
			0xd1 => Err(ExternalOpcode::Otherd1),
			0xd2 => Err(ExternalOpcode::Otherd2),
			0xd3 => Err(ExternalOpcode::Otherd3),
			0xd4 => Err(ExternalOpcode::Otherd4),
			0xd5 => Err(ExternalOpcode::Otherd5),
			0xd6 => Err(ExternalOpcode::Otherd6),
			0xd7 => Err(ExternalOpcode::Otherd7),
			0xd8 => Err(ExternalOpcode::Otherd8),
			0xd9 => Err(ExternalOpcode::Otherd9),
			0xda => Err(ExternalOpcode::Otherda),
			0xdb => Err(ExternalOpcode::Otherdb),
			0xdc => Err(ExternalOpcode::Otherdc),
			0xdd => Err(ExternalOpcode::Otherdd),
			0xde => Err(ExternalOpcode::Otherde),
			0xdf => Err(ExternalOpcode::Otherdf),

			0xe0 => Err(ExternalOpcode::Othere0),
			0xe1 => Err(ExternalOpcode::Othere1),
			0xe2 => Err(ExternalOpcode::Othere2),
			0xe3 => Err(ExternalOpcode::Othere3),
			0xe4 => Err(ExternalOpcode::Othere4),
			0xe5 => Err(ExternalOpcode::Othere5),
			0xe6 => Err(ExternalOpcode::Othere6),
			0xe7 => Err(ExternalOpcode::Othere7),
			0xe8 => Err(ExternalOpcode::Othere8),
			0xe9 => Err(ExternalOpcode::Othere9),
			0xea => Err(ExternalOpcode::Otherea),
			0xeb => Err(ExternalOpcode::Othereb),
			0xec => Err(ExternalOpcode::Otherec),
			0xed => Err(ExternalOpcode::Othered),
			0xee => Err(ExternalOpcode::Otheree),
			0xef => Err(ExternalOpcode::Otheref),

			0xf0 => Err(ExternalOpcode::Create),
			0xf1 => Err(ExternalOpcode::Call),
			0xf2 => Err(ExternalOpcode::CallCode),
			0xf3 => Ok(Opcode::Return),
			0xf4 => Err(ExternalOpcode::DelegateCall),
			0xf5 => Err(ExternalOpcode::Create2),
			0xf6 => Err(ExternalOpcode::Otherf6),
			0xf7 => Err(ExternalOpcode::Otherf7),
			0xf8 => Err(ExternalOpcode::Otherf8),
			0xf9 => Err(ExternalOpcode::Otherf9),
			0xfa => Err(ExternalOpcode::StaticCall),
			0xfb => Err(ExternalOpcode::Otherfb),
			0xfc => Err(ExternalOpcode::Otherfc),
			0xfd => Ok(Opcode::Revert),
			0xfe => Ok(Opcode::Invalid),
			0xff => Err(ExternalOpcode::Suicide),
		}
	}
}

/// External opcodes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ExternalOpcode {
	/// `SHA3`
	Sha3 = 0x20,
	/// `ADDRESS`
	Address = 0x30,
	/// `BALANCE`
	Balance = 0x31,
	/// `SELFBALANCE`
	SelfBalance = 0x47,
	/// `ORIGIN`
	Origin = 0x32,
	/// `CALLER`
	Caller = 0x33,
	/// `CALLVALUE`
	CallValue = 0x34,
	/// `GASPRICE`
	GasPrice = 0x3a,
	/// `EXTCODESIZE`
	ExtCodeSize = 0x3b,
	/// `EXTCODECOPY`
	ExtCodeCopy = 0x3c,
	/// `EXTCODEHASH`
	ExtCodeHash = 0x3f,
	/// `RETURNDATASIZE`
	ReturnDataSize = 0x3d,
	/// `RETURNDATACOPY`
	ReturnDataCopy = 0x3e,
	/// `BLOCKHASH`
	BlockHash = 0x40,
	/// `COINBASE`
	Coinbase = 0x41,
	/// `TIMESTAMP`
	Timestamp = 0x42,
	/// `NUMBER`
	Number = 0x43,
	/// `DIFFICULTY`
	Difficulty = 0x44,
	/// `GASLIMIT`
	GasLimit = 0x45,
	/// `SLOAD`
	SLoad = 0x54,
	/// `SSTORE`
	SStore = 0x55,
	/// `GAS`
	Gas = 0x5a,
	/// `LOGn`
	Log0 = 0xa0,
	Log1 = 0xa1,
	Log2 = 0xa2,
	Log3 = 0xa3,
	Log4 = 0xa4,
	/// `CREATE`
	Create = 0xf0,
	/// `CREATE2`
	Create2 = 0xf5,
	/// `CALL`
	Call = 0xf1,
	/// `CALLCODE`
	CallCode = 0xf2,
	/// `DELEGATECALL`
	DelegateCall = 0xf4,
	/// `STATICCALL`
	StaticCall = 0xfa,
	/// `SUICIDE`
	Suicide = 0xff,
	/// `CHAINID`
	ChainId = 0x46,

	// Other unknown opcodes.

	Other0c = 0x0c,
	Other0d = 0x0d,
	Other0e = 0x0e,
	Other0f = 0x0f,

	Other1e = 0x1e,
	Other1f = 0x1f,

	Other21 = 0x21,
	Other22 = 0x22,
	Other23 = 0x23,
	Other24 = 0x24,
	Other25 = 0x25,
	Other26 = 0x26,
	Other27 = 0x27,
	Other28 = 0x28,
	Other29 = 0x29,
	Other2a = 0x2a,
	Other2b = 0x2b,
	Other2c = 0x2c,
	Other2d = 0x2d,
	Other2e = 0x2e,
	Other2f = 0x2f,

	Other48 = 0x48,
	Other49 = 0x49,
	Other4a = 0x4a,
	Other4b = 0x4b,
	Other4c = 0x4c,
	Other4d = 0x4d,
	Other4e = 0x4e,
	Other4f = 0x4f,

	Other5c = 0x5c,
	Other5d = 0x5d,
	Other5e = 0x5e,
	Other5f = 0x5f,

	Othera5 = 0xa5,
	Othera6 = 0xa6,
	Othera7 = 0xa7,
	Othera8 = 0xa8,
	Othera9 = 0xa9,
	Otheraa = 0xaa,
	Otherab = 0xab,
	Otherac = 0xac,
	Otherad = 0xad,
	Otherae = 0xae,
	Otheraf = 0xaf,

	Otherb0 = 0xb0,
	Otherb1 = 0xb1,
	Otherb2 = 0xb2,
	Otherb3 = 0xb3,
	Otherb4 = 0xb4,
	Otherb5 = 0xb5,
	Otherb6 = 0xb6,
	Otherb7 = 0xb7,
	Otherb8 = 0xb8,
	Otherb9 = 0xb9,
	Otherba = 0xba,
	Otherbb = 0xbb,
	Otherbc = 0xbc,
	Otherbd = 0xbd,
	Otherbe = 0xbe,
	Otherbf = 0xbf,

	Otherc0 = 0xc0,
	Otherc1 = 0xc1,
	Otherc2 = 0xc2,
	Otherc3 = 0xc3,
	Otherc4 = 0xc4,
	Otherc5 = 0xc5,
	Otherc6 = 0xc6,
	Otherc7 = 0xc7,
	Otherc8 = 0xc8,
	Otherc9 = 0xc9,
	Otherca = 0xca,
	Othercb = 0xcb,
	Othercc = 0xcc,
	Othercd = 0xcd,
	Otherce = 0xce,
	Othercf = 0xcf,

	Otherd0 = 0xd0,
	Otherd1 = 0xd1,
	Otherd2 = 0xd2,
	Otherd3 = 0xd3,
	Otherd4 = 0xd4,
	Otherd5 = 0xd5,
	Otherd6 = 0xd6,
	Otherd7 = 0xd7,
	Otherd8 = 0xd8,
	Otherd9 = 0xd9,
	Otherda = 0xda,
	Otherdb = 0xdb,
	Otherdc = 0xdc,
	Otherdd = 0xdd,
	Otherde = 0xde,
	Otherdf = 0xdf,

	Othere0 = 0xe0,
	Othere1 = 0xe1,
	Othere2 = 0xe2,
	Othere3 = 0xe3,
	Othere4 = 0xe4,
	Othere5 = 0xe5,
	Othere6 = 0xe6,
	Othere7 = 0xe7,
	Othere8 = 0xe8,
	Othere9 = 0xe9,
	Otherea = 0xea,
	Othereb = 0xeb,
	Otherec = 0xec,
	Othered = 0xed,
	Otheree = 0xee,
	Otheref = 0xef,

	Otherf6 = 0xf6,
	Otherf7 = 0xf7,
	Otherf8 = 0xf8,
	Otherf9 = 0xf9,
	Otherfb = 0xfb,
	Otherfc = 0xfc,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn opcode_u8_match() {
		for v in 0..=u8::max_value() {
			let opcode = Opcode::parse(v);

			match opcode {
				Ok(opcode) => assert_eq!(opcode as u8, v),
				Err(opcode) => assert_eq!(opcode as u8, v),
			}
		}
	}
}
