#[macro_use]
extern crate log;
extern crate crypto;
extern crate merkle;
extern crate capnp;
extern crate libc;
extern crate serde_json;

pub mod vm;
pub mod transaction;
pub mod blockchain;

mod utils;
mod ffi;

pub use utils::bigint::{U256, M256, MI256};
pub use utils::gas::Gas;
pub use utils::hash::H256;
pub use utils::address::Address;
pub use utils::read_hex;

use std::io::BufReader;
use capnp::{serialize, message, Word};
use log::LogLevel;
use vm::{Machine, FakeVectorMachine, VectorMachine};
use ffi::{JSONVectorBlock, create_block, create_transaction};
use serde_json::{Value, Error};

use libc::size_t;
use std::slice;
use std::str::FromStr;

#[repr(C)]
pub struct SputnikVM {
    svm: VectorMachine<JSONVectorBlock, Box<JSONVectorBlock>>
}

impl SputnikVM {
    fn new(v: &Value) -> SputnikVM {
        let block = create_block(v);
        let transaction = create_transaction(v);

        let gas = Gas::from_str(v["exec"]["gas"].as_str().unwrap()).unwrap();
        let code = read_hex(v["exec"]["code"].as_str().unwrap()).unwrap();
        let data = read_hex(v["exec"]["data"].as_str().unwrap()).unwrap();

        SputnikVM {
            svm: VectorMachine::new(code.as_ref(), data.as_ref(), gas, transaction, Box::new(block))
        }
    }

    fn return_values(&mut self) ->  &[u8] {
        self.svm.return_values()
    }

    fn fire(&mut self) {
        self.svm.fire();
    }
}

#[no_mangle]
pub extern fn sputnikvm_new(v: &Value) -> *mut SputnikVM {
    Box::into_raw(Box::new(SputnikVM::new(v)))
}

#[no_mangle]
pub extern fn sputnikvm_fire(ptr: *mut SputnikVM) {
    let mut svm = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    svm.fire();
}

#[no_mangle]
pub extern fn sputnikvm_free(ptr: *mut SputnikVM) {
    if ptr.is_null() { return }
    unsafe { Box::from_raw(ptr); }
}
