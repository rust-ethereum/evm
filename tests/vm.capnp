@0xb8d0e016e09e5605;

const stop :Data = 0x"00";

struct VMInput {
  gas @0 :List(Data);
  code @1 :List(Data);
  data @2 :List(Data);
}
