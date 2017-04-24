@0xc98fff04bdc3a38a;

struct Input {
  gas @0 :Data;
  code @1 :List(Data);
  data @2 :List(Data);
}

struct Output {
  gas @0 :Data;
  out @1 :Data;
  balance @2 :Data;
  code @3 :List(Data);
  nonce @4 :Data;
  storage @5 :List(Data);
}

struct InputOutput {
  input @0 :Input;
  output @1 :Output;
}
