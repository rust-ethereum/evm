@0xa8dd6a728e8f8499;

using Hierarchy = import "hierarchy.capnp";

# test modules
using Arith = import "arith/mod.capnp";

const all: Hierarchy.Directories = (
  name = "top",
  dirs = [ Arith.all ]
);
