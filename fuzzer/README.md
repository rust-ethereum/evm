# Fuzzing the rust evm
This provides a simple fuzzing harness that can be used a start when fuzzing the rust evm.
The fuzzer will take a byte input from honggfuzz, split it at a fixed delimiter
and give some of it as code and some of it as data to the evm.

# Running the fuzzer
Run the fuzzer like so:
```
cargo hfuzz run evm_fuzz
```
Honggfuzz saves the fuzzing queue in the directory `hfuzz_workspace/evm_fuzz/input`.
To debug the fuzzer, you can also compile the fuzzer as a normal binary via `cargo build`.
Then, to debug a certain input in the fuzzing queue, run 
`./target/debug/evm_fuzz hfuzz_workspace/evm_fuzz/input/<file>`

# Notes
Because this fuzzer does not implement any gasometer, honggfuzz will report some timeouts.
A reasonable approach to improve the perfomance would be to extend this fuzzer with a gasometer.
