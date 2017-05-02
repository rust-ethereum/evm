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
    let len = ffi.sputnikvm_return_values_len(svm);

    let mut return_values = [0u8; 32];
    ffi.sputnikvm_return_values_copy(svm, &mut return_values);

    println!("values: {:?}", return_values);
    ffi.sputnikvm_free(svm);
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

    fn sputnikvm_return_values_copy(&self, svm: *mut SputnikVM, array: &mut [u8]) {
        unsafe {
            let f = self.0.get::<fn(svm: *mut SputnikVM, array_ptr: *mut uint8_t, len: size_t)>(
                b"sputnikvm_return_values_copy\0"
            ).unwrap();
            f(svm, array.as_mut_ptr(), array.len() as size_t);
        }
    }

    fn sputnikvm_return_values_len(&self, svm: *mut SputnikVM) -> usize {
        unsafe {
            let f = self.0.get::<fn(svm: *mut SputnikVM) -> size_t> (
                b"sputnikvm_return_values_len\0"
            ).unwrap();
            let len = f(svm);
            len
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
