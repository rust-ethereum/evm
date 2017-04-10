@0xe165e950e83b0d24;

using Hierarchy = import "../hierarchy.capnp";
using Add = import "add.capnp";
using Div = import "div.capnp";

const all: Hierarchy.Files = (
  name = "arith",
  files = [ Add.all, Div.all ]
);
