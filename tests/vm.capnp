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

struct Test {
  input @0 :Input;
  output @1 :Output;
}
