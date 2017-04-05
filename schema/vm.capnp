@0xc98fff04bdc3a38a;

struct Input {
  gas @0 :Int32;
  code @1 :List(Data);
  data @2 :List(Data);
}

struct Output {
  gas @0 :Int32;
  code @1 :List(Data);
}

struct InputOutput {
  name @0 :Text;
  input @1 :Input;
  output @2 :Output;
}
