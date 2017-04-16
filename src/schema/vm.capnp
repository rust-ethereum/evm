@0xc98fff04bdc3a38a;

struct Input {
  initialGas @0 :Int32;
  code @1 :List(Data);
  data @2 :List(Data);
}

struct Output {
  usedGas @0 :Int32;
  code @1 :List(Data);
}

struct InputOutput {
  input @0 :Input;
  output @1 :Output;
}
