@0xda7d72d8f4b3c0c1;

using Vm = import "../vm.capnp";

const all: List(Vm.VMInput) = [
  .stop, .add, .multiply
];

const stop: Vm.VMInput = (
  gas= 314159,
  code=[ Vm.STOP, Vm.STOP ],
  data=[ Vm.STOP, Vm.STOP ]
);

const add: Vm.VMInput = (
  gas= 314159,
  code=[ Vm.ADD, Vm.ADD ],
  data=[ Vm.ADD, Vm.ADD ]
);

const multiply: Vm.VMInput = (
  gas= 314159,
  code=[ Vm.MUL, Vm.MUL ],
  data=[ Vm.MUL, Vm.MUL ]
);
