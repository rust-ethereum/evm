@0xda7d72d8f4b3c0c2;

using Test = import "../../src/schema/test.capnp";
using Op = import "../../src/schema/opcodes.capnp";
using Hierarchy = import "../../src/schema/hierarchy.capnp";

const all: Hierarchy.Tests = (
  name = "and",
  tests = [ .and1, .and2 ]
);

const and1: Test.InputOutput = (
  name = "and1",
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

const and2: Test.InputOutput = (
  name = "and2",
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
