#[macro_use]
extern crate log;
extern crate crypto;
extern crate merkle;
extern crate capnp;

pub mod vm;
pub mod account;
pub mod transaction;
pub mod blockchain;
mod utils;
mod vm_capnp;

pub use utils::u256::U256;
pub use utils::gas::Gas;
pub use utils::hash::H256;
pub use utils::address::Address;
use std::io::BufReader;
use capnp::{serialize, message};
use capnp::traits::FromPointerBuilder;

use vm_capnp::input_output::Reader;

#[no_mangle]
pub extern fn evaluate(ptr: *mut Reader) {
    let msg = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    println!("{:?}", ptr);
    // let io_reader = serialize::read_message(&mut msg, message::ReaderOptions::new()).expect("read message failed.");
    // let io = io_reader.get_root::<Reader>().expect("Failed to get VM IO.");
    // println!("{}", io.get_input().expect("FAILED").get_gas());
}
