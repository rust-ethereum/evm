@0xe165e950e83b0d24;

using Vm = import "../vm.capnp";
using Hierarchy = import "../hierarchy.capnp";
using Basic = import "basic.capnp";

const all: Hierarchy.Files = (
  name = "arith",
  files = [ Basic.all ]
);
