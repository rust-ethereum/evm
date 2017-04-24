@0x9b740c183df7ef25;

using Vm = import "vm.capnp";

# A test compares the actual output with the expected output
# see vm.capnp for the actual output
# see the struct ExpectedOutput for the expected output

struct ExpectedOutput {
  gas @0 :Data;
  out @1 :Data;
  balance @2 :Data;
  code @3 :List(Data);
  nonce @4 :Data;
  storage @5 :List(Data);
}

struct InputOutput {
  name @0 :Text;
  inputOutput @1 :Vm.InputOutput;
  expectedOutput @2 :ExpectedOutput;
}
