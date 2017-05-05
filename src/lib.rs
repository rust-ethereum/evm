#[macro_use]
extern crate log;
extern crate crypto;
extern crate merkle;
extern crate libc;
extern crate serde_json;

#[macro_use]
mod rescue;

pub mod vm;

mod utils;
// mod ffi;

pub use utils::bigint::{U256, M256, MI256};
pub use utils::gas::Gas;
pub use utils::hash::H256;
pub use utils::address::Address;
pub use utils::opcode::Opcode;
pub use utils::read_hex;

use std::io::BufReader;
use log::LogLevel;
use vm::{Machine};
use ffi::{JSONVectorBlock, create_block, create_transaction};
use serde_json::{Value, Error};
use libc::{size_t, uint8_t};

use std::os::raw::c_char;
use std::str::FromStr;
use std::ffi::CStr;

// #[repr(C)]
// pub struct SputnikVM {
//     svm: VectorMachine<JSONVectorBlock, Box<JSONVectorBlock>>
// }

// impl SputnikVM {
//     fn new(v: &Value) -> SputnikVM {
//         let block = create_block(v);
//         let transaction = create_transaction(v);

//         let gas = Gas::from_str(v["exec"]["gas"].as_str().unwrap()).unwrap();
//         let code = read_hex(v["exec"]["code"].as_str().unwrap()).unwrap();
//         let data = read_hex(v["exec"]["data"].as_str().unwrap()).unwrap();

//         SputnikVM {
//             svm: VectorMachine::new(code.as_ref(), data.as_ref(), gas, transaction, Box::new(block))
//         }
//     }

//     fn return_values(&mut self) ->  &[u8] {
//         let ret = self.svm.return_values();
//         ret
//     }

//     fn fire(&mut self) {
//         self.svm.fire();
//     }
// }

// #[no_mangle]
// pub extern fn sputnikvm_new(v: &Value) -> *mut SputnikVM {
//     Box::into_raw(Box::new(SputnikVM::new(v)))
// }

// #[no_mangle]
// pub extern fn sputnikvm_fire(ptr: *mut SputnikVM) {
//     let mut svm = unsafe {
//         assert!(!ptr.is_null());
//         &mut *ptr
//     };
//     svm.fire();
// }

// #[no_mangle]
// pub extern fn sputnikvm_return_values_len(ptr: *mut SputnikVM) -> size_t {
//     let mut svm = unsafe {
//         assert!(!ptr.is_null());
//         &mut *ptr
//     };
//     let ret = svm.return_values();
//     ret.len()
// }

// #[no_mangle]
// pub extern fn sputnikvm_return_values_copy(ptr: *mut SputnikVM, array_ptr: *mut uint8_t, len: size_t) {
//     use std::slice::from_raw_parts_mut;
//     assert!(!array_ptr.is_null());

//     let mut svm = unsafe {
//         assert!(!ptr.is_null());
//         &mut *ptr
//     };

//     let ret = svm.return_values();
//     let s = unsafe { from_raw_parts_mut(array_ptr, len) };

//     for i in 0..len {
//         if i < ret.len() {
//             s[i] = ret[i];
//         }
//     }
// }
// #[no_mangle]
// pub extern fn sputnikvm_free(ptr: *mut SputnikVM) {
//     if ptr.is_null() { return }
//     unsafe { Box::from_raw(ptr); }
// }
