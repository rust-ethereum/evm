@0xea2c8e2dc7ce97f8;

using Vm = import "../schema/vm.capnp";

struct Tests {
  name @0 :Text;
  tests @1 :List(Vm.InputOutput);
}

struct Files {
  name @0 :Text;
  files @1 :List(Tests);
}

struct Directories {
  name @0 :Text;
  dirs @1 :List(Files);
}
