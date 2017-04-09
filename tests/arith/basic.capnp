@0xda7d72d8f4b3c0c1;

using Test = import "../test.capnp";
using Op = import "../opcodes.capnp";
using Hierarchy = import "../hierarchy.capnp";

const all: Hierarchy.Tests = (
  name = "basic",
  tests = [ .stop, .add ]
);

const stop: Test.InputOutput = (
  inputOutput = (
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
  ),
  expectedOutput = (
    gas = 314159,
    code = [ Op.STOP ]
  )
);

const add: Test.InputOutput = (
  inputOutput = (
    name = "add",
    input = (
      gas = 314159,
      code = [ Op.STOP, Op.STOP ],
      data = [ Op.STOP, Op.STOP ]
    ),
    output = (
      gas = 314159,
      code = [ Op.STOP ]
    )
  ),
  expectedOutput = (
    gas = 314159,
    code = [ Op.STOP ]
  )
);
