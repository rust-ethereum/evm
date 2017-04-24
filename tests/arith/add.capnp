@0xda7d72d8f4b3c0c1;

using Test = import "../../src/schema/test.capnp";
using Op = import "../../src/schema/opcodes.capnp";
using Hierarchy = import "../../src/schema/hierarchy.capnp";

const all: Hierarchy.Tests = (
  name = "add",
  tests = [ .add1 ]
);

const add1: Test.InputOutput = (
  name = "add1",
  inputOutput = (
    input = (
      gas = 0x"314159",
      code = [ 0x"7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
      , Op.ADD
      , Op.PUSH1
      , 0x"00"
      , Op.SSTORE ],
      data = [ 0x"00" ]
    ),
    output = (
      gas = 0x"314159",
      out = 0x"00",
      balance = 0x"0de0b6b3a7640000",
      code = [ 0x"7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
      , Op.ADD
      , Op.PUSH1
      , 0x"00"
      , Op.SSTORE ],
      nonce = 0x"00",
      storage = [ ]
    )
  ),
  expectedOutput = (
    gas = 0x"314159",
    out = 0x"00",
    balance = 0x"0de0b6b3a7640000",
    code = [ 0x"7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    , Op.ADD
    , Op.PUSH1
    , 0x"00"
    , Op.SSTORE ],
    nonce = 0x"00",
    storage = []

  )
);
#
# const add2: Test.InputOutput = (
#   name = "add2",
#   inputOutput = (
#     input = (
#       initialGas = 0x"314159",
#       code = [ Op.STOP, Op.STOP ],
#       data = [ Op.STOP, Op.STOP ]
#     ),
#     output = (
#       usedGas = 0x"314159",
#       code = [ Op.STOP, Op.STOP ]
#     )
#   ),
#   expectedOutput = (
#     usedGas = 0x"314159",
#     code = [ Op.STOP, Op.STOP]
#   )
# );
