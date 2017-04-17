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
      initialGas = 314159,
      code = [ Op.STOP, Op.STOP ],
      data = [ Op.STOP, Op.STOP ]
    ),
    output = (
      usedGas = 314159,
      code = [ Op.ADD, Op.LT, Op.GT, Op.GT ]
    )
  ),
  expectedOutput = (
    usedGas = 314159,
    code = [ Op.ADD, Op.LT, Op.GT, Op.GT ]
  )
);

const add2: Test.InputOutput = (
  name = "add2",
  inputOutput = (
    input = (
      initialGas = 314159,
      code = [ Op.STOP, Op.STOP ],
      data = [ Op.STOP, Op.STOP ]
    ),
    output = (
      usedGas = 314159,
      code = [ Op.STOP, Op.STOP ]
    )
  ),
  expectedOutput = (
    usedGas = 314159,
    code = [ Op.STOP, Op.STOP]
  )
);
