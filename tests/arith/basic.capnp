@0xda7d72d8f4b3c0c1;

using Vm = import "../vm.capnp";
using Op = import "../opcodes.capnp";

const all: List(Vm.Input) = [
  .stop, .add, .multiply
];

const stop: Vm.Input = (
  gas= 314159,
  code=[ Op.STOP, Op.STOP ],
  data=[ Op.STOP, Op.STOP ]
);

const add: Vm.Input = (
  gas= 314159,
  code=[ Op.ADD, Op.ADD ],
  data=[ Op.ADD, Op.ADD ]
);

const multiply: Vm.Input = (
  gas= 314159,
  code=[ Op.MUL, Op.MUL ],
  data=[ Op.MUL, Op.MUL ]
);
