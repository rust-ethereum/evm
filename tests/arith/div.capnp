@0xda7d72d8f4b3c0c6;

using Test = import "../../src/schema/test.capnp";
using Op = import "../../src/schema/opcodes.capnp";
using Hierarchy = import "../../src/schema/hierarchy.capnp";

const all: Hierarchy.Tests = (
  name = "div",
  tests = [ .div1, .div2 ]
);

const div1: Test.InputOutput = (
  name = "div1",
  inputOutput = (
    input = (
      initialGas = 0x"314159",
      code = [ Op.STOP, Op.STOP ],
      data = [ Op.STOP, Op.STOP ]
    ),
    output = (
      usedGas = 0x"314159",
      code = [ Op.STOP ]
    )
  ),
  expectedOutput = (
    usedGas = 0x"314159",
    code = [ Op.STOP ]
  )
);

const div2: Test.InputOutput = (
  name = "div2",
  inputOutput = (
    input = (
      initialGas = 0x"314159",
      code = [ Op.STOP, Op.STOP ],
      data = [ Op.STOP, Op.STOP ]
    ),
    output = (
      usedGas = 0x"314159",
      code = [ Op.STOP ]
    )
  ),
  expectedOutput = (
    usedGas = 0x"314159",
    code = [ Op.STOP ]
  )
);
