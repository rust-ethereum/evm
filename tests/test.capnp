@0x9b740c183df7ef25;

using Vm = import "../schema/vm.capnp";

# for comparing Actual VM Output (see ../schema/vm.capnp) with Expected VM Output
struct ExpectedOutput {
  gas @0 :Int32;
  code @1 :List(Data);
}

struct InputOutput {
  name @0 :Text;
  inputOutput @1 :Vm.InputOutput;
  expectedOutput @2 :ExpectedOutput;
}
