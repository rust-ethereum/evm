@0xda7d72d8f4b3c0c1;

using Vm = import "vm.capnp";

const stop: Vm.VMInput = (
  gas=[.Vm.STOP, .Vm.STOP],
  code=[.Vm.STOP, .Vm.STOP],
  data=[.Vm.STOP, .Vm.STOP]
);
