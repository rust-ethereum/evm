@0xa8dd6a728e8f8499;

using Hierarchy = import "../src/schema/hierarchy.capnp";

using Arith = import "arith/mod.capnp";
using Bitwise = import "bitwise/mod.capnp";

const all: Hierarchy.Directories = (
  name = "top",
  dirs = [ Arith.all]
);
