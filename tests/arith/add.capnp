@0xda7d72d8f4b3c0c1;

using Test = import "../../src/schema/test.capnp";
using Op = import "../../src/schema/opcodes.capnp";
using Hierarchy = import "../../src/schema/hierarchy.capnp";

const all: Hierarchy.Tests = (
  name = "add",
  tests = [ .add1, .add2 ]
);

const add1: Test.InputOutput = (
  name = "add1",
  inputOutput = (
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

const add2: Test.InputOutput = (
  name = "add2",
  inputOutput = (
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
