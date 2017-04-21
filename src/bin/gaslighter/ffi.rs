use capnp;
use libloading;
use libc;

use std;
use std::io::BufReader;
use std::fs::File;

use hierarchy_capnp::directories;
use vm_capnp;
use test_capnp;
use capnp::{serialize, message};

struct ExecuteTest<'a> {
    pub name: String,
    pub capnp: test_capnp::input_output::Reader<'a>,
}

pub fn execute(file: File, test_to_run: &str, sputnikvm_path: &str, keep_going: bool) -> bool {
    let (dir_to_run, file_to_run, test_to_run) = test_scope(test_to_run.into());
    let mut contents = BufReader::new(file);
    let tests_reader = serialize::read_message(&mut contents, message::ReaderOptions::new()).expect("read message failed.");
    let mut tests_to_execute: std::vec::Vec<ExecuteTest> = Vec::new();
    let top_level_tests = tests_reader.get_root::<directories::Reader>().expect("failed to get top level test root.");
    for dir in top_level_tests.get_dirs().expect("failed to directories.").iter() {
        let mut add_dir = false;
        let dirname = dir.get_name().expect("failed to get directory name.");
        if dirname == dir_to_run || dir_to_run == "" { add_dir = true; }
        for file in dir.get_files().expect("failed to files.").iter() {
            let mut add_file = false;
            let filename = file.get_name().expect("failed to get filename.");
            if filename == file_to_run || file_to_run == "" { add_file = true; }
            for test in file.get_tests().expect("failed to get tests.").iter() {
                let mut add_test = false;
                let testname = test.get_name().expect("failed to get test name.");
                if testname == test_to_run || test_to_run == "" {
                    add_test = true;
                    if add_dir && add_file && add_test {
                        let execute_test = ExecuteTest {
                            name: format!("{}::{}::{}", dirname, filename, testname),
                            capnp: test,
                        };
                        tests_to_execute.push(execute_test);
                    }
                }
            }
        }
    }
    has_all_ffi_tests_passed(tests_to_execute, keep_going, sputnikvm_path)
}

fn test_scope(test_to_run: String) -> (String, String, String) {
    let vec: Vec<&str> = test_to_run.split("/").collect();
    (vec[0].into(), vec[1].into(), vec[2].into())
}

use libc::size_t;
use std::slice;

struct Sputnikvm(libloading::Library);

impl Sputnikvm {
    fn evaluate(&self, vm_io: *const capnp::Word, len: size_t) {
        unsafe {
            let f = self.0.get::<fn(vm_io: *const capnp::Word, len: size_t)> (
                b"evaluate\0"
            ).unwrap();
            f(vm_io, len)
        }
    }
}

fn construct_vec_word(vm_io: *const capnp::Word, len: size_t) -> Vec<capnp::Word> {
    let vm_input_output = unsafe {
        assert!(!vm_io.is_null());
        slice::from_raw_parts(vm_io, len as usize)
    };
    vm_input_output.to_vec()
}

fn has_all_ffi_tests_passed(tests_to_execute: std::vec::Vec<ExecuteTest>
    , keep_going: bool
    , sputnikvm_path: &str) -> bool {
    println!("running {} tests", tests_to_execute.len());
    let mut has_all_ffi_tests_passed = true;
    let mut sputnikvm = Sputnikvm(libloading::Library::new(sputnikvm_path).unwrap_or_else(|error| panic!("{}", error)));
    for test in tests_to_execute {
        print!("sputnikvm test {} ", test.name);
        let test = test.capnp;
        let eo = test.get_expected_output().expect("failed to get expected output");
        let io = test.get_input_output().expect("failed to get actual input output");
        let mut message = message::Builder::new_default();
        message.set_root(io);
        let mut vm_resp = serialize::write_message_to_words(&message);
        let response = sputnikvm.evaluate(vm_resp.as_ptr(), vm_resp.len());
        // let vm_resp = construct_vec_word(response.capnp, response.len);
        let io = serialize::read_message_from_words(&vm_resp, message::ReaderOptions::new()).expect("failed to read vm response");
        let io = io.get_root::<vm_capnp::input_output::Reader>().expect("failed to get root");
        let ao_gas = io.get_output().expect("failed to get output").get_used_gas();
        let eo_gas = eo.get_used_gas();
        let ao_code = io.get_output().expect("").get_code().expect("").iter();
        let eo_code = eo.get_code().expect("").iter();
        let mut ao_vec = Vec::new();
        let mut eo_vec = Vec::new();
        let mut has_this_test_failed = false;
        for ao_char in ao_code {
            ao_vec.push(ao_char.expect("character expected")[0]);
        }
        for eo_char in eo_code {
            eo_vec.push(eo_char.expect("character expected")[0]);
        }
        let length = eo_vec.len();
        let matching = ao_vec.iter().zip(eo_vec.iter()).filter(|&(a, b)| a == b).count();
        if matching != length {
            has_this_test_failed = true;
            print!("\n\n code equality fail: only {} actual output opcodes matched the {} opcodes of the expected output.\n", matching, length);
            println!(" actual code output:\t{:?}", ao_vec);
            println!(" expected code output:\t{:?}", eo_vec);
        }
        if eo_gas != ao_gas {
            has_this_test_failed = true;
            print!("\n gas equality fail: actual output gas value of {} doesn't equal the expected output gas value of {}.", ao_gas, eo_gas);
            print!("\n actual gas output:\t{}", ao_gas);
            print!("\n expected gas output:\t{}\n", eo_gas);
        }
        if has_this_test_failed {
            print!("\n");
            has_all_ffi_tests_passed = false;
            if !keep_going {
                return has_all_ffi_tests_passed;
            }
        } else {
            print!("... ok\n")
        }
    }
    has_all_ffi_tests_passed
}
