/// Opcode enum. One-to-one corresponding to an `u8` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "scale",
	derive(scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Opcode(pub u8);

// Core opcodes.
impl Opcode {
	/// `STOP`
	pub const STOP: Opcode = Opcode(0x00);
	/// `ADD`
	pub const ADD: Opcode = Opcode(0x01);
	/// `MUL`
	pub const MUL: Opcode = Opcode(0x02);
	/// `SUB`
	pub const SUB: Opcode = Opcode(0x03);
	/// `DIV`
	pub const DIV: Opcode = Opcode(0x04);
	/// `SDIV`
	pub const SDIV: Opcode = Opcode(0x05);
	/// `MOD`
	pub const MOD: Opcode = Opcode(0x06);
	/// `SMOD`
	pub const SMOD: Opcode = Opcode(0x07);
	/// `ADDMOD`
	pub const ADDMOD: Opcode = Opcode(0x08);
	/// `MULMOD`
	pub const MULMOD: Opcode = Opcode(0x09);
	/// `EXP`
	pub const EXP: Opcode = Opcode(0x0a);
	/// `SIGNEXTEND`
	pub const SIGNEXTEND: Opcode = Opcode(0x0b);

	/// `LT`
	pub const LT: Opcode = Opcode(0x10);
	/// `GT`
	pub const GT: Opcode = Opcode(0x11);
	/// `SLT`
	pub const SLT: Opcode = Opcode(0x12);
	/// `SGT`
	pub const SGT: Opcode = Opcode(0x13);
	/// `EQ`
	pub const EQ: Opcode = Opcode(0x14);
	/// `ISZERO`
	pub const ISZERO: Opcode = Opcode(0x15);
	/// `AND`
	pub const AND: Opcode = Opcode(0x16);
	/// `OR`
	pub const OR: Opcode = Opcode(0x17);
	/// `XOR`
	pub const XOR: Opcode = Opcode(0x18);
	/// `NOT`
	pub const NOT: Opcode = Opcode(0x19);
	/// `BYTE`
	pub const BYTE: Opcode = Opcode(0x1a);

	/// `SHL`
	pub const SHL: Opcode = Opcode(0x1b);
	/// `SHR`
	pub const SHR: Opcode = Opcode(0x1c);
	/// `SAR`
	pub const SAR: Opcode = Opcode(0x1d);

	/// `CALLDATALOAD`
	pub const CALLDATALOAD: Opcode = Opcode(0x35);
	/// `CALLDATASIZE`
	pub const CALLDATASIZE: Opcode = Opcode(0x36);
	/// `CALLDATACOPY`
	pub const CALLDATACOPY: Opcode = Opcode(0x37);
	/// `CODESIZE`
	pub const CODESIZE: Opcode = Opcode(0x38);
	/// `CODECOPY`
	pub const CODECOPY: Opcode = Opcode(0x39);

	/// `POP`
	pub const POP: Opcode = Opcode(0x50);
	/// `MLOAD`
	pub const MLOAD: Opcode = Opcode(0x51);
	/// `MSTORE`
	pub const MSTORE: Opcode = Opcode(0x52);
	/// `MSTORE8`
	pub const MSTORE8: Opcode = Opcode(0x53);

	/// `JUMP`
	pub const JUMP: Opcode = Opcode(0x56);
	/// `JUMPI`
	pub const JUMPI: Opcode = Opcode(0x57);
	/// `PC`
	pub const PC: Opcode = Opcode(0x58);
	/// `MSIZE`
	pub const MSIZE: Opcode = Opcode(0x59);

	/// `JUMPDEST`
	pub const JUMPDEST: Opcode = Opcode(0x5b);
	/// `MCOPY`
	pub const MCOPY: Opcode = Opcode(0x5e);

	/// `PUSH0`
	pub const PUSH0: Opcode = Opcode(0x5f);
	/// `PUSH1`
	pub const PUSH1: Opcode = Opcode(0x60);
	/// `PUSH2`
	pub const PUSH2: Opcode = Opcode(0x61);
	/// `PUSH3`
	pub const PUSH3: Opcode = Opcode(0x62);
	/// `PUSH4`
	pub const PUSH4: Opcode = Opcode(0x63);
	/// `PUSH5`
	pub const PUSH5: Opcode = Opcode(0x64);
	/// `PUSH6`
	pub const PUSH6: Opcode = Opcode(0x65);
	/// `PUSH7`
	pub const PUSH7: Opcode = Opcode(0x66);
	/// `PUSH8`
	pub const PUSH8: Opcode = Opcode(0x67);
	/// `PUSH9`
	pub const PUSH9: Opcode = Opcode(0x68);
	/// `PUSH10`
	pub const PUSH10: Opcode = Opcode(0x69);
	/// `PUSH11`
	pub const PUSH11: Opcode = Opcode(0x6a);
	/// `PUSH12`
	pub const PUSH12: Opcode = Opcode(0x6b);
	/// `PUSH13`
	pub const PUSH13: Opcode = Opcode(0x6c);
	/// `PUSH14`
	pub const PUSH14: Opcode = Opcode(0x6d);
	/// `PUSH15`
	pub const PUSH15: Opcode = Opcode(0x6e);
	/// `PUSH16`
	pub const PUSH16: Opcode = Opcode(0x6f);
	/// `PUSH17`
	pub const PUSH17: Opcode = Opcode(0x70);
	/// `PUSH18`
	pub const PUSH18: Opcode = Opcode(0x71);
	/// `PUSH19`
	pub const PUSH19: Opcode = Opcode(0x72);
	/// `PUSH20`
	pub const PUSH20: Opcode = Opcode(0x73);
	/// `PUSH21`
	pub const PUSH21: Opcode = Opcode(0x74);
	/// `PUSH22`
	pub const PUSH22: Opcode = Opcode(0x75);
	/// `PUSH23`
	pub const PUSH23: Opcode = Opcode(0x76);
	/// `PUSH24`
	pub const PUSH24: Opcode = Opcode(0x77);
	/// `PUSH25`
	pub const PUSH25: Opcode = Opcode(0x78);
	/// `PUSH26`
	pub const PUSH26: Opcode = Opcode(0x79);
	/// `PUSH27`
	pub const PUSH27: Opcode = Opcode(0x7a);
	/// `PUSH28`
	pub const PUSH28: Opcode = Opcode(0x7b);
	/// `PUSH29`
	pub const PUSH29: Opcode = Opcode(0x7c);
	/// `PUSH30`
	pub const PUSH30: Opcode = Opcode(0x7d);
	/// `PUSH31`
	pub const PUSH31: Opcode = Opcode(0x7e);
	/// `PUSH32`
	pub const PUSH32: Opcode = Opcode(0x7f);

	/// `DUP1`
	pub const DUP1: Opcode = Opcode(0x80);
	/// `DUP2`
	pub const DUP2: Opcode = Opcode(0x81);
	/// `DUP3`
	pub const DUP3: Opcode = Opcode(0x82);
	/// `DUP4`
	pub const DUP4: Opcode = Opcode(0x83);
	/// `DUP5`
	pub const DUP5: Opcode = Opcode(0x84);
	/// `DUP6`
	pub const DUP6: Opcode = Opcode(0x85);
	/// `DUP7`
	pub const DUP7: Opcode = Opcode(0x86);
	/// `DUP8`
	pub const DUP8: Opcode = Opcode(0x87);
	/// `DUP9`
	pub const DUP9: Opcode = Opcode(0x88);
	/// `DUP10`
	pub const DUP10: Opcode = Opcode(0x89);
	/// `DUP11`
	pub const DUP11: Opcode = Opcode(0x8a);
	/// `DUP12`
	pub const DUP12: Opcode = Opcode(0x8b);
	/// `DUP13`
	pub const DUP13: Opcode = Opcode(0x8c);
	/// `DUP14`
	pub const DUP14: Opcode = Opcode(0x8d);
	/// `DUP15`
	pub const DUP15: Opcode = Opcode(0x8e);
	/// `DUP16`
	pub const DUP16: Opcode = Opcode(0x8f);

	/// `SWAP1`
	pub const SWAP1: Opcode = Opcode(0x90);
	/// `SWAP2`
	pub const SWAP2: Opcode = Opcode(0x91);
	/// `SWAP3`
	pub const SWAP3: Opcode = Opcode(0x92);
	/// `SWAP4`
	pub const SWAP4: Opcode = Opcode(0x93);
	/// `SWAP5`
	pub const SWAP5: Opcode = Opcode(0x94);
	/// `SWAP6`
	pub const SWAP6: Opcode = Opcode(0x95);
	/// `SWAP7`
	pub const SWAP7: Opcode = Opcode(0x96);
	/// `SWAP8`
	pub const SWAP8: Opcode = Opcode(0x97);
	/// `SWAP9`
	pub const SWAP9: Opcode = Opcode(0x98);
	/// `SWAP10`
	pub const SWAP10: Opcode = Opcode(0x99);
	/// `SWAP11`
	pub const SWAP11: Opcode = Opcode(0x9a);
	/// `SWAP12`
	pub const SWAP12: Opcode = Opcode(0x9b);
	/// `SWAP13`
	pub const SWAP13: Opcode = Opcode(0x9c);
	/// `SWAP14`
	pub const SWAP14: Opcode = Opcode(0x9d);
	/// `SWAP15`
	pub const SWAP15: Opcode = Opcode(0x9e);
	/// `SWAP16`
	pub const SWAP16: Opcode = Opcode(0x9f);

	/// See [EIP-3541](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-3541.md)
	pub const EOFMAGIC: Opcode = Opcode(0xef);

	/// `RETURN`
	pub const RETURN: Opcode = Opcode(0xf3);

	/// `REVERT`
	pub const REVERT: Opcode = Opcode(0xfd);

	/// `INVALID`
	pub const INVALID: Opcode = Opcode(0xfe);
}

// External opcodes
impl Opcode {
	/// `SHA3`
	pub const SHA3: Opcode = Opcode(0x20);

	/// `ADDRESS`
	pub const ADDRESS: Opcode = Opcode(0x30);
	/// `BALANCE`
	pub const BALANCE: Opcode = Opcode(0x31);
	/// `ORIGIN`
	pub const ORIGIN: Opcode = Opcode(0x32);
	/// `CALLER`
	pub const CALLER: Opcode = Opcode(0x33);
	/// `CALLVALUE`
	pub const CALLVALUE: Opcode = Opcode(0x34);

	/// `GASPRICE`
	pub const GASPRICE: Opcode = Opcode(0x3a);
	/// `EXTCODESIZE`
	pub const EXTCODESIZE: Opcode = Opcode(0x3b);
	/// `EXTCODECOPY`
	pub const EXTCODECOPY: Opcode = Opcode(0x3c);
	/// `RETURNDATASIZE`
	pub const RETURNDATASIZE: Opcode = Opcode(0x3d);
	/// `RETURNDATACOPY`
	pub const RETURNDATACOPY: Opcode = Opcode(0x3e);
	/// `EXTCODEHASH`
	pub const EXTCODEHASH: Opcode = Opcode(0x3f);

	/// `BLOCKHASH`
	pub const BLOCKHASH: Opcode = Opcode(0x40);
	/// `COINBASE`
	pub const COINBASE: Opcode = Opcode(0x41);
	/// `TIMESTAMP`
	pub const TIMESTAMP: Opcode = Opcode(0x42);
	/// `NUMBER`
	pub const NUMBER: Opcode = Opcode(0x43);
	/// `DIFFICULTY`
	pub const DIFFICULTY: Opcode = Opcode(0x44);
	/// `GASLIMIT`
	pub const GASLIMIT: Opcode = Opcode(0x45);
	/// `CHAINID`
	pub const CHAINID: Opcode = Opcode(0x46);
	/// `SELFBALANCE`
	pub const SELFBALANCE: Opcode = Opcode(0x47);
	/// `BASEFEE`
	pub const BASEFEE: Opcode = Opcode(0x48);
	/// `BLOBHASH`
	pub const BLOBHASH: Opcode = Opcode(0x49);
	/// `BLOBBASEFEE`
	pub const BLOBBASEFEE: Opcode = Opcode(0x4a);

	/// `SLOAD`
	pub const SLOAD: Opcode = Opcode(0x54);
	/// `SSTORE`
	pub const SSTORE: Opcode = Opcode(0x55);

	/// `GAS`
	pub const GAS: Opcode = Opcode(0x5a);

	/// `TLOAD`
	pub const TLOAD: Opcode = Opcode(0x5c);
	/// `TSTORE`
	pub const TSTORE: Opcode = Opcode(0x5d);

	/// `LOG0`
	pub const LOG0: Opcode = Opcode(0xa0);
	/// `LOG1`
	pub const LOG1: Opcode = Opcode(0xa1);
	/// `LOG2`
	pub const LOG2: Opcode = Opcode(0xa2);
	/// `LOG3`
	pub const LOG3: Opcode = Opcode(0xa3);
	/// `LOG4`
	pub const LOG4: Opcode = Opcode(0xa4);

	/// `CREATE`
	pub const CREATE: Opcode = Opcode(0xf0);
	/// `CALL`
	pub const CALL: Opcode = Opcode(0xf1);
	/// `CALLCODE`
	pub const CALLCODE: Opcode = Opcode(0xf2);

	/// `DELEGATECALL`
	pub const DELEGATECALL: Opcode = Opcode(0xf4);
	/// `CREATE2`
	pub const CREATE2: Opcode = Opcode(0xf5);

	/// `STATICCALL`
	pub const STATICCALL: Opcode = Opcode(0xfa);

	/// `SUICIDE`
	pub const SUICIDE: Opcode = Opcode(0xff);
}

impl Opcode {
	/// Whether the opcode is a push opcode.
	#[must_use]
	pub fn is_push(&self) -> Option<u8> {
		let value = self.0;
		if (0x60..=0x7f).contains(&value) {
			Some(value - 0x60 + 1)
		} else {
			None
		}
	}

	/// Convert opcode to u8.
	#[inline]
	#[must_use]
	pub const fn as_u8(&self) -> u8 {
		self.0
	}

	/// Convert opcode to usize.
	#[inline]
	#[must_use]
	pub const fn as_usize(&self) -> usize {
		self.0 as usize
	}
}
