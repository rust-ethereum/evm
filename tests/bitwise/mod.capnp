@0xdcfa74f6e8423ba1;

using Hierarchy = import "../hierarchy.capnp";
using And = import "and.capnp";
using Or = import "or.capnp";

const all: Hierarchy.Files = (
  name = "bitwise",
  files = [ And.all, Or.all ]
);
