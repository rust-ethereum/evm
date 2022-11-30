# Fuzzing the rust evm
This provides a simple fuzzing harness that can be used a start when fuzzing the rust evm.
The fuzzer will take an structured input using the Arbitrary trait in the form of an `EVMInput` struct.
This is then given to the EVM, such that the fuzzer controls parameters, code and data.
This will then spawn a afl++, honggfuzz and libfuzzer fuzzer, controlled by [ziggy](https://crates.io/crates/ziggy).

# Running the fuzzer
Install [ziggy](https://crates.io/crates/ziggy), [honggfuzz](https://crates.io/crates/honggfuzz), [libfuzzer](https://crates.io/crates/libfuzzer-sys) and [afl++](https://crates.io/crates/afl):
```
cargo install ziggy fuzz honggfuzz afl
```

Run the fuzzer like so:
```
cargo ziggy fuzz
```
See the [ziggy crate](https://crates.io/crates/ziggy) for more options

ziggy saves the fuzzing queue in the directory `output/<target-name>/shared_corpus/`.
To debug a certain input in the fuzzing queue, run 
`cargo ziggy run -i output/evm-fuzzer/shared_corpus/<input_file>`
