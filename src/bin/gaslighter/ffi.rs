use libloading;
use sputnikvm::{SputnikVM};
use serde_json::{Value, Error};
use std::io::{BufReader, Write, stdout};
use libc::{size_t, uint8_t};
use std::slice;

pub fn test_ffi_transaction(name: &str, v: &Value, debug: bool, sputnikvm_path: &str) {
    let mut ffi = FFI(libloading::Library::new(sputnikvm_path).unwrap_or_else(|error| panic!("{}",error)));
    println!("Testing {} ... ", name);
    if debug {
        print!("\n");
    }
    stdout().flush();

    let svm = ffi.sputnikvm_new(v);
    ffi.sputnikvm_fire(svm);
    let values = ffi.sputnikvm_return_values(svm);
    let values = unsafe {
        assert!(!values.data.is_null());
        slice::from_raw_parts(values.data, values.len as usize)
    };
    // println!("values: {:?}", values);
    // uncomment the above to trigger the segfault

    ffi.sputnikvm_free(svm);
}

#[repr(C)]
pub struct Tuple {
    data: *const uint8_t,
    len: size_t,
}
struct FFI(libloading::Library);

impl FFI {
    fn sputnikvm_new(&self, v: &Value) -> *mut SputnikVM {
        unsafe {
            let f = self.0.get::<fn(v: &Value) -> *mut SputnikVM > (
                b"sputnikvm_new\0"
            ).unwrap();
            f(v)
        }
    }
    fn sputnikvm_fire(&self, svm: *mut SputnikVM) {
        unsafe {
            let f = self.0.get::<fn(svm: *mut SputnikVM)> (
                b"sputnikvm_fire\0"
            ).unwrap();
            f(svm)
        }
    }

    fn sputnikvm_return_values(&self, svm: *mut SputnikVM) -> Tuple {
        unsafe {
            let f = self.0.get::<fn(svm: *mut SputnikVM) -> Tuple> (
                b"sputnikvm_return_values\0"
            ).unwrap();
            f(svm)
        }
    }
    fn sputnikvm_free(&self, svm: *mut SputnikVM) {
        unsafe {
            let f = self.0.get::<fn(svm: *mut SputnikVM)> (
                b"sputnikvm_free\0"
            ).unwrap();
            f(svm)
        }
    }
}
