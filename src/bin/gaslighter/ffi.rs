use libloading;
use sputnikvm::{SputnikVM};
use serde_json::{Value, Error};
use std::io::{BufReader, Write, stdout};



pub fn test_ffi_transaction(name: &str, v: &Value, debug: bool, sputnikvm_path: &str) {
    let mut ffi = FFI(libloading::Library::new(sputnikvm_path).unwrap_or_else(|error| panic!("{}",error)));
    println!("Testing {} ... ", name);
    if debug {
        print!("\n");
    }
    stdout().flush();

    let svm = ffi.sputnikvm_new(v);
    // ffi.sputnikvm_free(svm);
}

struct FFI(libloading::Library);

impl FFI {
    fn sputnikvm_new(&self, v: &Value) {
        unsafe {
            let f = self.0.get::<fn(v: &Value)> (
                b"sputnikvm_new\0"
            ).unwrap();
            f(v)
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
