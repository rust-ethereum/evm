#[macro_use]
extern crate clap;
extern crate capnp;
extern crate libloading;
extern crate libc;

mod hierarchy_capnp;
mod vm_capnp;
mod test_capnp;

use std::fs::File;
use std::path::Path;
use std::io::BufReader;
use std::process;

use hierarchy_capnp::{directories};
use capnp::{serialize, message};

struct ExecuteTest<'a> {
    pub name: String,
    pub capnp: test_capnp::input_output::Reader<'a>,
}

fn main() {
    let matches = clap_app!(gaslighter =>
        (version: "0.1.0")
        (author: "Stewart Mackenzie <setori88@gmail.com>")
        (about: "Gaslighter - Ethereum Virtual Machine tester.")
        (@arg CAPNPROTO_TYPECHECKED_TEST_BIN: -t --capnp_test_bin +takes_value "Path to a type checked binary compiled by the capnp tool. The source of this artefact is in the tests directory. Please run `$ capnp eval -b tests/mod.capnp all > tests.bin` in the root directory to generate the binary.")
        (@arg TESTS_TO_RUN: -r --run_test +takes_value +required "The format is [directory]/[file]/[test] e.g. `--run_test arith/add/add1` will run the arith/add/add1 test, `--run_test arith/add/` will run every test in the tests/arith/add.capnp file. Likewise `--run_test arith//` will run every test in every file of the `arith` directory. Lastly `--run_test //` will run every single test available.")
        (@arg KEEP_GOING: -k --keep_going "Don't exit the program even if a test fails.")
    ).get_matches();
    let capnp_test_bin = match matches.value_of("CAPNPROTO_TYPECHECKED_TEST_BIN") {
        Some(c) => c,
        None => "",
    };
    let test_to_run = match matches.value_of("TESTS_TO_RUN") {
        Some(c) => c,
        None => "",
    };
    let keep_going = if matches.is_present("KEEP_GOING") { true } else { false };
    let path = Path::new(capnp_test_bin);
    let display = path.display();
    let file = match File::open(&path) {
        Err(_) => panic!("couldn't open {}", display),
        Ok(file) => file,
    };
    let (dir_to_run, file_to_run, test_to_run) = test_scope(test_to_run.into());
    let mut contents = BufReader::new(file);
    let tests_reader = serialize::read_message(&mut contents, message::ReaderOptions::new()).expect("read message failed.");
    let mut tests_to_execute :std::vec::Vec<ExecuteTest> = Vec::new();
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
    if has_all_tests_passed(tests_to_execute, keep_going) {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

fn test_scope(test_to_run: String) -> (String, String, String) {
    let vec: Vec<&str> = test_to_run.split("/").collect();
    (vec[0].into(), vec[1].into(), vec[2].into())
}

const LIB_PATH: &'static str = "libsputnikvm.so";
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

fn has_all_tests_passed(tests_to_execute: std::vec::Vec<ExecuteTest>, keep_going: bool) -> bool {
    println!("running {} tests", tests_to_execute.len());
    let mut has_all_tests_passed = true;
    let mut sputnikvm = Sputnikvm(libloading::Library::new(LIB_PATH).unwrap_or_else(|error| panic!("{}", error)));
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
            has_all_tests_passed = false;
            if !keep_going {
                return has_all_tests_passed;
            }
        } else {
            print!("... ok\n")
        }
    }
    has_all_tests_passed
}
