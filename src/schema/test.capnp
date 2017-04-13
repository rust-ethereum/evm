@0x9b740c183df7ef25;

using Vm = import "vm.capnp";

# A test compares the actual output with the expected output
# see vm.capnp for the actual output
# see the struct ExpectedOutput for the expected output

struct ExpectedOutput {
  gas @0 :Int32;
  code @1 :List(Data);
}

struct InputOutput {
  name @0 :Text;
  inputOutput @1 :Vm.InputOutput;
  expectedOutput @2 :ExpectedOutput;
}
