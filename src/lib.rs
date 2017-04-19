#[macro_use]
extern crate log;
extern crate crypto;
extern crate merkle;
extern crate capnp;

pub mod vm;
pub mod transaction;
pub mod blockchain;

mod utils;
mod vm_capnp;

pub use utils::u256::U256;
pub use utils::gas::Gas;
pub use utils::hash::H256;
pub use utils::address::Address;
pub use utils::read_hex;

use std::io::BufReader;
use capnp::{serialize, message, Word};
use log::LogLevel;
use vm_capnp::input_output::Reader;
use vm::{Machine, FakeVectorMachine};

#[no_mangle]
pub extern fn evaluate(ptr: *mut std::vec::Vec<capnp::Word>) {
    let msg = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    let message = serialize::read_message_from_words(&msg, message::ReaderOptions::new()).expect("");
    let msg_reader = message.get_root::<vm_capnp::input_output::Reader>().expect("");
    let mut code_vec = Vec::new();
    let mut data_vec = Vec::new();
    let in_code = msg_reader.get_input().expect("input fail").get_code().expect("input::code fail").iter();
    let in_data = msg_reader.get_input().expect("input fail").get_data().expect("input::data fail").iter();
    for in_char in in_code {
        code_vec.push(in_char.expect("character expected")[0]);
    }
    for in_char in in_data {
        data_vec.push(in_char.expect("character expected")[0]);
    }
    let gas = msg_reader.get_input().expect("failed3").get_initial_gas();

    let mut machine = FakeVectorMachine::fake(
        code_vec.as_slice()
        , data_vec.as_slice()
        , Gas::from(gas as isize));
    machine.fire();
    println!("gas used: {:?}", machine.used_gas());
    if log_enabled!(LogLevel::Info) {
        info!("gas used: {:?}", machine.used_gas());
    }
}
