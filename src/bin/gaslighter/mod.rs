#[macro_use]
extern crate clap;
extern crate capnp;

mod hierarchy_capnp;
mod vm_capnp;

use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::io::BufReader;

use hierarchy_capnp::{directories, files};
use capnp::{serialize, message};


fn main() {
    let matches = clap_app!(gaslighter =>
        (version: "0.1.0")
        (author: "Stewart Mackenzie <setori88@gmail.com>")
        (about: "Gaslighter - Ethereum Virtual Machine tester.")
        (@arg CAPNPROTO_TYPECHECKED_TEST_BIN: -t --capnp_test_bin +takes_value "Absolute path to a typechecked binary compiled by the capnp tool. The source of this artefact is in the tests directory. Please run `$ capnp eval -b mod.capnp all >> test.bin` in the tests directory.")
        (@arg TESTS_TO_RUN: -r --run_test +takes_value "The format is <directory>/<file>/<test> e.g. `--run_test arith/basic/add` will run the arith/basic/add test, `--run_test arith/basic/` will run all the tests in the tests/arith/basic.capnp file. Likewise `--run_test arith//` will run all the tests in all the files of the `arith` directory. Lastly `--run_test //` will run all the tests.")
    ).get_matches();
    let capnp_test_bin = match matches.value_of("CAPNPROTO_TYPECHECKED_TEST_BIN") {
        Some(c) => c,
        None => "",
    };
    let test_to_run = match matches.value_of("TESTS_TO_RUN") {
        Some(c) => c,
        None => "",
    };
    println!("capnp_test_bin: {:?}", capnp_test_bin);
    let path = Path::new(capnp_test_bin);
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(_) => panic!("couldn't open {}", display),
        Ok(file) => file,
    };

    let (dir_to_run, file_to_run, test_to_run) = test_scope(test_to_run.into());
    let mut add_dir = false;
    let mut add_file = false;
    let mut add_test = false;
    let mut contents = BufReader::new(file);
    let tests_reader = serialize::read_message(&mut contents, message::ReaderOptions::new()).unwrap();
    let mut tests_to_execute = Vec::new();
    let top_level_tests = tests_reader.get_root::<directories::Reader>().unwrap();
    for dir in top_level_tests.get_dirs().unwrap().iter() {
        let dirname = dir.get_name().unwrap();
        if dirname == dir_to_run || dir_to_run == "" { add_dir = true; }
        else { add_dir = false; }
        for file in dir.get_files().unwrap().iter() {
            let filename = file.get_name().unwrap();
            if filename == file_to_run || file_to_run == "" { add_file = true; }
            else { add_file = false; }
            for test in file.get_tests().unwrap().iter() {
                let testname = test.get_name().unwrap();
                if testname == test_to_run || test_to_run == "" {
                    add_test = true;
                    if add_dir && add_file && add_test {
                        tests_to_execute.push(test);
                    }
                } else { add_test = false; }
            }
        }
    }
    println!("tests_to_execute length: {:?}", tests_to_execute.len() );
}

fn test_scope(test_to_run: String) -> (String, String, String) {
    let vec: Vec<&str> = test_to_run.split("/").collect();
    (vec[0].into(), vec[1].into(), vec[2].into())
}
