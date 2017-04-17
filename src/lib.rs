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
pub use utils::read_hex;

use std::io::BufReader;
use capnp::{serialize, message};
use log::LogLevel;

use vm_capnp::input_output::Reader;
use vm::{Machine, FakeVectorMachine};


#[no_mangle]
pub extern fn evaluate(ptr: *mut Reader) {
    let msg = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    // let io_reader = serialize::read_message(&mut msg, message::ReaderOptions::new()).expect("read message failed.");
    // let io = io_reader.get_root::<Reader>().expect("Failed to get VM IO.");
    // println!("{}", io.get_input().expect("FAILED").get_gas());

    let mut message = message::Builder::new_default();
    let mut msg = message.init_root::<vm_capnp::input_output::Builder>();
    msg.borrow().init_input();
    {
        let mut code = msg.borrow().get_input().unwrap().init_code(4);
        for i in 0..code.len() {
            code.set(i, &[b'0']);
        }
    }
    {
        let mut data = msg.borrow().get_input().unwrap().init_data(4);
        for i in 0..data.len() {
            data.set(i, &[b'0']);
        }
    }
    {
        msg.borrow().get_input().unwrap().set_gas(444);
    }
    let msg_reader = msg.as_reader();
    let mut code_vec = Vec::new();
    let mut data_vec = Vec::new();
    let in_code = msg_reader.get_input().unwrap().get_code().expect("").iter();
    let in_data = msg_reader.get_input().unwrap().get_data().expect("").iter();
    for in_char in in_code {
        code_vec.push(in_char.expect("character expected")[0]);
    }
    for in_char in in_data {
        data_vec.push(in_char.expect("character expected")[0]);
    }
    let gas = msg_reader.get_input().unwrap().get_gas();

    let mut machine = FakeVectorMachine::new(
        code_vec.as_slice()
        , data_vec.as_slice()
        , Gas::from(gas as isize));
    machine.fire();
    println!("gas used: {:?}", machine.used_gas());
    if log_enabled!(LogLevel::Info) {
        info!("gas used: {:?}", machine.used_gas());
    }
}
