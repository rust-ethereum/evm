#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

fn main() {
    let matches = clap_app!(gaslighter =>
        (version: "0.1.0")
        (author: "Stewart Mackenzie <setori88@gmail.com>")
        (about: "Gaslighter - Ethereum Virtual Machine tester.")
        (@arg CLONE_OF_EVM_TESTS: -t --test_dir +takes_value +required "Sets a mandatory path to the root of a clone of https://github.com/ethereumproject/tests")
        (@arg ARTEFACT_DIRECTORY_ROOT: -a --artefact_dir +takes_value +required "Sets the root directory that contains the artefacts `libsputnikvm.so` and command line interface executable called `svm`")
    ).get_matches();
    let test_dir = match matches.value_of("CLONE_OF_EVM_TESTS") {
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
