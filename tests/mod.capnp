@0xa8dd6a728e8f8499;

using Vm = import "vm.capnp";
using Arith = import "arith/mod.capnp";

const all: List(List(List(Vm.Test))) = [
  Arith.all
];
