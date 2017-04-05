@0xda7d72d8f4b3c0c1;

using Vm = import "../vm.capnp";
using Op = import "../opcodes.capnp";
using Hierarchy = import "../hierarchy.capnp";

const all: Hierarchy.Tests = (
  name = "basic",
  tests = [ .stop, .add, .multiply ]
);

const stop: Vm.Test = (
  name = "stop",
  input = (
    gas = 314159,
    code = [ Op.STOP, Op.STOP ],
    data = [ Op.STOP, Op.STOP ]
  ),
  output = (
    gas = 314159,
    code = [ Op.STOP ]
  )
);

const add: Vm.Test = (
  name = "add",
  input = (
    gas = 314159,
    code = [ Op.ADD, Op.ADD ],
    data = [ Op.ADD, Op.ADD ]
  ),
  output = (
    gas = 314159,
    code = [ Op.STOP ]
  )
);

const multiply: Vm.Test = (
  name = "multiply",
  input = (
    gas = 314159,
    code = [ Op.MUL, Op.MUL ],
    data = [ Op.MUL, Op.MUL ]
  ),
  output = (
    gas = 314159,
    code = [ Op.STOP ]
  )
);
