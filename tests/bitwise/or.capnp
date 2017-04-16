@0xda7d72d8f4b3c0c5;

using Test = import "../../src/schema/test.capnp";
using Op = import "../../src/schema/opcodes.capnp";
using Hierarchy = import "../../src/schema/hierarchy.capnp";

const all: Hierarchy.Tests = (
  name = "or",
  tests = [ .or1, .or2 ]
);

const or1: Test.InputOutput = (
  name = "or1",
  inputOutput = (
    input = (
      initialGas = 314159,
      code = [ Op.STOP, Op.STOP ],
      data = [ Op.STOP, Op.STOP ]
    ),
    output = (
      usedGas = 314159,
      code = [ Op.STOP ]
    )
  ),
  expectedOutput = (
    usedGas = 314159,
    code = [ Op.STOP ]
  )
);

const or2: Test.InputOutput = (
  name = "or2",
  inputOutput = (
    input = (
      initialGas = 314159,
      code = [ Op.STOP, Op.STOP ],
      data = [ Op.STOP, Op.STOP ]
    ),
    output = (
      usedGas = 314159,
      code = [ Op.STOP ]
    )
  ),
  expectedOutput = (
    usedGas = 314159,
    code = [ Op.STOP ]
  )
);
