//! SputnikVM EVM implementation.

#![deny(unused_import_braces, unused_imports,
        unused_comparisons, unused_must_use,
        unused_variables, non_shorthand_field_patterns,
        unreachable_code)]

extern crate log;
extern crate tiny_keccak;
extern crate etcommon_rlp as rlp;
extern crate etcommon_bigint as bigint;
extern crate etcommon_util;
extern crate etcommon_block;
extern crate ripemd160;
extern crate sha2;
extern crate secp256k1;
extern crate digest;

mod util;
pub mod vm;

pub use util::bigint::{U256, M256, MI256};
pub use util::gas::Gas;
pub use util::address::Address;
pub use util::opcode::Opcode;
pub use util::read_hex;
