#[macro_use]
extern crate clap;

fn main() {
    let matches = clap_app!(gaslighter =>
        (version: "0.1.0")
        (author: "Stewart Mackenzie <setori88@gmail.com>")
        (about: "Gaslighter - Ethereum Virtual Machine tester.")
        (@arg CAPNPROTO_TYPECHECKED_TEST_BIN: -t --capnp_test_bin +takes_value "Absolute path to a typechecked binary compiled by the capnp tool. The source of this artefact is in the tests directory. Please run `$ capnp eval -b mod.capnp all >> test.bin` in the tests directory.")
        (@arg ARTEFACT_DIRECTORY_ROOT: -a --artefact_dir +takes_value +required "Sets the root directory that contains the artefacts `libsputnikvm.so` and command line interface executable called `svm`")
    ).get_matches();
    let test_dir = match matches.value_of("CAPNPROTO_TYPECHECKED_TEST_BIN") {
        Some(c) => c,
        None => "",
    };
    println!("test_dir: {:?}", test_dir);
    let artefact_dir = match matches.value_of("ARTEFACT_DIRECTORY_ROOT") {
        Some(c) => c,
        None => "",
    };
    println!("artefact_dir: {:?}", artefact_dir);

}
