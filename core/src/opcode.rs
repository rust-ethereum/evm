/// Opcode enum. One-to-one corresponding to an `u8` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "with-codec",
	derive(scale_codec::Encode, scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Opcode(pub u8);

// Core opcodes.
impl Opcode {
	/// `STOP`
	pub const STOP: Self = Self(0x00);
	/// `ADD`
	pub const ADD: Self = Self(0x01);
	/// `MUL`
	pub const MUL: Self = Self(0x02);
	/// `SUB`
	pub const SUB: Self = Self(0x03);
	/// `DIV`
	pub const DIV: Self = Self(0x04);
	/// `SDIV`
	pub const SDIV: Self = Self(0x05);
	/// `MOD`
	pub const MOD: Self = Self(0x06);
	/// `SMOD`
	pub const SMOD: Self = Self(0x07);
	/// `ADDMOD`
	pub const ADDMOD: Self = Self(0x08);
	/// `MULMOD`
	pub const MULMOD: Self = Self(0x09);
	/// `EXP`
	pub const EXP: Self = Self(0x0a);
	/// `SIGNEXTEND`
	pub const SIGNEXTEND: Self = Self(0x0b);

	/// `LT`
	pub const LT: Self = Self(0x10);
	/// `GT`
	pub const GT: Self = Self(0x11);
	/// `SLT`
	pub const SLT: Self = Self(0x12);
	/// `SGT`
	pub const SGT: Self = Self(0x13);
	/// `EQ`
	pub const EQ: Self = Self(0x14);
	/// `ISZERO`
	pub const ISZERO: Self = Self(0x15);
	/// `AND`
	pub const AND: Self = Self(0x16);
	/// `OR`
	pub const OR: Self = Self(0x17);
	/// `XOR`
	pub const XOR: Self = Self(0x18);
	/// `NOT`
	pub const NOT: Self = Self(0x19);
	/// `BYTE`
	pub const BYTE: Self = Self(0x1a);

	/// `CALLDATALOAD`
	pub const CALLDATALOAD: Self = Self(0x35);
	/// `CALLDATASIZE`
	pub const CALLDATASIZE: Self = Self(0x36);
	/// `CALLDATACOPY`
	pub const CALLDATACOPY: Self = Self(0x37);
	/// `CODESIZE`
	pub const CODESIZE: Self = Self(0x38);
	/// `CODECOPY`
	pub const CODECOPY: Self = Self(0x39);

	/// `SHL`
	pub const SHL: Self = Self(0x1b);
	/// `SHR`
	pub const SHR: Self = Self(0x1c);
	/// `SAR`
	pub const SAR: Self = Self(0x1d);

	/// `POP`
	pub const POP: Self = Self(0x50);
	/// `MLOAD`
	pub const MLOAD: Self = Self(0x51);
	/// `MSTORE`
	pub const MSTORE: Self = Self(0x52);
	/// `MSTORE8`
	pub const MSTORE8: Self = Self(0x53);
	/// `JUMP`
	pub const JUMP: Self = Self(0x56);
	/// `JUMPI`
	pub const JUMPI: Self = Self(0x57);
	/// `PC`
	pub const PC: Self = Self(0x58);
	/// `MSIZE`
	pub const MSIZE: Self = Self(0x59);
	/// `JUMPDEST`
	pub const JUMPDEST: Self = Self(0x5b);

	/// `PUSHn`
	pub const PUSH1: Self = Self(0x60);
	pub const PUSH2: Self = Self(0x61);
	pub const PUSH3: Self = Self(0x62);
	pub const PUSH4: Self = Self(0x63);
	pub const PUSH5: Self = Self(0x64);
	pub const PUSH6: Self = Self(0x65);
	pub const PUSH7: Self = Self(0x66);
	pub const PUSH8: Self = Self(0x67);
	pub const PUSH9: Self = Self(0x68);
	pub const PUSH10: Self = Self(0x69);
	pub const PUSH11: Self = Self(0x6a);
	pub const PUSH12: Self = Self(0x6b);
	pub const PUSH13: Self = Self(0x6c);
	pub const PUSH14: Self = Self(0x6d);
	pub const PUSH15: Self = Self(0x6e);
	pub const PUSH16: Self = Self(0x6f);
	pub const PUSH17: Self = Self(0x70);
	pub const PUSH18: Self = Self(0x71);
	pub const PUSH19: Self = Self(0x72);
	pub const PUSH20: Self = Self(0x73);
	pub const PUSH21: Self = Self(0x74);
	pub const PUSH22: Self = Self(0x75);
	pub const PUSH23: Self = Self(0x76);
	pub const PUSH24: Self = Self(0x77);
	pub const PUSH25: Self = Self(0x78);
	pub const PUSH26: Self = Self(0x79);
	pub const PUSH27: Self = Self(0x7a);
	pub const PUSH28: Self = Self(0x7b);
	pub const PUSH29: Self = Self(0x7c);
	pub const PUSH30: Self = Self(0x7d);
	pub const PUSH31: Self = Self(0x7e);
	pub const PUSH32: Self = Self(0x7f);

	/// `DUPn`
	pub const DUP1: Self = Self(0x80);
	pub const DUP2: Self = Self(0x81);
	pub const DUP3: Self = Self(0x82);
	pub const DUP4: Self = Self(0x83);
	pub const DUP5: Self = Self(0x84);
	pub const DUP6: Self = Self(0x85);
	pub const DUP7: Self = Self(0x86);
	pub const DUP8: Self = Self(0x87);
	pub const DUP9: Self = Self(0x88);
	pub const DUP10: Self = Self(0x89);
	pub const DUP11: Self = Self(0x8a);
	pub const DUP12: Self = Self(0x8b);
	pub const DUP13: Self = Self(0x8c);
	pub const DUP14: Self = Self(0x8d);
	pub const DUP15: Self = Self(0x8e);
	pub const DUP16: Self = Self(0x8f);

	/// `SWAPn`
	pub const SWAP1: Self = Self(0x90);
	pub const SWAP2: Self = Self(0x91);
	pub const SWAP3: Self = Self(0x92);
	pub const SWAP4: Self = Self(0x93);
	pub const SWAP5: Self = Self(0x94);
	pub const SWAP6: Self = Self(0x95);
	pub const SWAP7: Self = Self(0x96);
	pub const SWAP8: Self = Self(0x97);
	pub const SWAP9: Self = Self(0x98);
	pub const SWAP10: Self = Self(0x99);
	pub const SWAP11: Self = Self(0x9a);
	pub const SWAP12: Self = Self(0x9b);
	pub const SWAP13: Self = Self(0x9c);
	pub const SWAP14: Self = Self(0x9d);
	pub const SWAP15: Self = Self(0x9e);
	pub const SWAP16: Self = Self(0x9f);

	/// `RETURN`
	pub const RETURN: Self = Self(0xf3);
	/// `REVERT`
	pub const REVERT: Self = Self(0xfd);

	/// `INVALID`
	pub const INVALID: Self = Self(0xfe);

	/// See [EIP-3541](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-3541.md)
	pub const EOFMAGIC: Self = Self(0xef);
}

// External opcodes
impl Opcode {
	/// `SHA3`
	pub const SHA3: Self = Self(0x20);
	/// `ADDRESS`
	pub const ADDRESS: Self = Self(0x30);
	/// `BALANCE`
	pub const BALANCE: Self = Self(0x31);
	/// `SELFBALANCE`
	pub const SELFBALANCE: Self = Self(0x47);
	/// `BASEFEE`
	pub const BASEFEE: Self = Self(0x48);
	/// `ORIGIN`
	pub const ORIGIN: Self = Self(0x32);
	/// `CALLER`
	pub const CALLER: Self = Self(0x33);
	/// `CALLVALUE`
	pub const CALLVALUE: Self = Self(0x34);
	/// `GASPRICE`
	pub const GASPRICE: Self = Self(0x3a);
	/// `EXTCODESIZE`
	pub const EXTCODESIZE: Self = Self(0x3b);
	/// `EXTCODECOPY`
	pub const EXTCODECOPY: Self = Self(0x3c);
	/// `EXTCODEHASH`
	pub const EXTCODEHASH: Self = Self(0x3f);
	/// `RETURNDATASIZE`
	pub const RETURNDATASIZE: Self = Self(0x3d);
	/// `RETURNDATACOPY`
	pub const RETURNDATACOPY: Self = Self(0x3e);
	/// `BLOCKHASH`
	pub const BLOCKHASH: Self = Self(0x40);
	/// `COINBASE`
	pub const COINBASE: Self = Self(0x41);
	/// `TIMESTAMP`
	pub const TIMESTAMP: Self = Self(0x42);
	/// `NUMBER`
	pub const NUMBER: Self = Self(0x43);
	/// `DIFFICULTY`
	pub const DIFFICULTY: Self = Self(0x44);
	/// `GASLIMIT`
	pub const GASLIMIT: Self = Self(0x45);
	/// `SLOAD`
	pub const SLOAD: Self = Self(0x54);
	/// `SSTORE`
	pub const SSTORE: Self = Self(0x55);
	/// `GAS`
	pub const GAS: Self = Self(0x5a);
	/// `LOGn`
	pub const LOG0: Self = Self(0xa0);
	pub const LOG1: Self = Self(0xa1);
	pub const LOG2: Self = Self(0xa2);
	pub const LOG3: Self = Self(0xa3);
	pub const LOG4: Self = Self(0xa4);
	/// `CREATE`
	pub const CREATE: Self = Self(0xf0);
	/// `CREATE2`
	pub const CREATE2: Self = Self(0xf5);
	/// `CALL`
	pub const CALL: Self = Self(0xf1);
	/// `CALLCODE`
	pub const CALLCODE: Self = Self(0xf2);
	/// `DELEGATECALL`
	pub const DELEGATECALL: Self = Self(0xf4);
	/// `STATICCALL`
	pub const STATICCALL: Self = Self(0xfa);
	/// `SUICIDE`
	pub const SUICIDE: Self = Self(0xff);
	/// `CHAINID`
	pub const CHAINID: Self = Self(0x46);
}

impl Opcode {
	/// Whether the opcode is a push opcode.
	pub fn is_push(&self) -> Option<u8> {
		let value = self.0;
		if (0x60..=0x7f).contains(&value) {
			Some(value - 0x60 + 1)
		} else {
			None
		}
	}

	#[inline]
	pub const fn as_u8(&self) -> u8 {
		self.0
	}

	#[inline]
	pub const fn as_usize(&self) -> usize {
		self.0 as usize
	}
}
