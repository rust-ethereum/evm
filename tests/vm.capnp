@0xb8d0e016e09e5605;

const STOP :Data = 0x"00";
const ADD :Data = 0x"01";
const MUL :Data = 0x"02";
const SUB :Data = 0x"03";
const DIV :Data = 0x"04";
const SDIV :Data = 0x"05";
const MOD :Data = 0x"06";
const SMOD :Data = 0x"07";
const ADDMOD :Data = 0x"08";
const MULMOD :Data = 0x"09";
const EXP :Data = 0x"0a";
const SIGNEXTEND :Data = 0x"0b";

const LT :Data = 0x"10";
const GT :Data = 0x"11";
const SLT :Data = 0x"12";
const SGT :Data = 0x"13";
const EQ :Data = 0x"14";
const ISZERO :Data = 0x"15";
const AND :Data = 0x"16";
const OR :Data = 0x"17";
const XOR :Data = 0x"18";
const NOT :Data = 0x"19";
const BYTE :Data = 0x"1a";

const SHA3 :Data = 0x"20";

const ADDRESS :Data = 0x"30";
const BALANCE :Data = 0x"31";
const ORIGIN :Data = 0x"32";
const CALLER :Data = 0x"33";
const CALLVALUE :Data = 0x"34";
const CALLDATALOAD :Data = 0x"35";
const CALLDATASIZE :Data = 0x"36";
const CALLDATACOPY :Data = 0x"37";
const CODESIZE :Data = 0x"38";
const CODECOPY :Data = 0x"39";
const GASPRICE :Data = 0x"3a";
const EXTCODESIZE :Data = 0x"3b";
const EXTCODECOPY :Data = 0x"3c";

const BLOCKHASH :Data = 0x"40";
const COINBASE :Data = 0x"41";
const TIMESTAMP :Data = 0x"42";
const NUMBER :Data = 0x"43";
const DIFFICULTY :Data = 0x"44";
const GASLIMIT :Data = 0x"45";

const POP :Data = 0x"50";
const MLOAD :Data = 0x"51";
const MSTORE :Data = 0x"52";
const MSTORE8 :Data = 0x"53";
const SLOAD :Data = 0x"54";
const SSTORE :Data = 0x"55";
const JUMP :Data = 0x"56";
const JUMPI :Data = 0x"57";
const PC :Data = 0x"58";
const MSIZE :Data = 0x"59";
const GAS :Data = 0x"5a";
const JUMPDEST :Data = 0x"5b";

const PUSH1 :Data = 0x"60";
const PUSH2 :Data = 0x"61";
const PUSH3 :Data = 0x"62";
const PUSH4 :Data = 0x"63";
const PUSH5 :Data = 0x"64";
const PUSH6 :Data = 0x"65";
const PUSH7 :Data = 0x"66";
const PUSH8 :Data = 0x"67";
const PUSH9 :Data = 0x"68";
const PUSH10 :Data = 0x"69";
const PUSH11 :Data = 0x"6a";
const PUSH12 :Data = 0x"6b";
const PUSH13 :Data = 0x"6c";
const PUSH14 :Data = 0x"6d";
const PUSH15 :Data = 0x"6e";
const PUSH16 :Data = 0x"6f";
const PUSH17 :Data = 0x"70";
const PUSH18 :Data = 0x"71";
const PUSH19 :Data = 0x"72";
const PUSH20 :Data = 0x"73";
const PUSH21 :Data = 0x"74";
const PUSH22 :Data = 0x"75";
const PUSH23 :Data = 0x"76";
const PUSH24 :Data = 0x"77";
const PUSH25 :Data = 0x"78";
const PUSH26 :Data = 0x"79";
const PUSH27 :Data = 0x"7a";
const PUSH28 :Data = 0x"7b";
const PUSH29 :Data = 0x"7c";
const PUSH30 :Data = 0x"7d";
const PUSH31 :Data = 0x"7e";
const PUSH32 :Data = 0x"7f";

const DUP1 :Data = 0x"80";
const DUP2 :Data = 0x"81";
const DUP3 :Data = 0x"82";
const DUP4 :Data = 0x"83";
const DUP5 :Data = 0x"84";
const DUP6 :Data = 0x"85";
const DUP7 :Data = 0x"86";
const DUP8 :Data = 0x"87";
const DUP9 :Data = 0x"88";
const DUP10 :Data = 0x"89";
const DUP11 :Data = 0x"8a";
const DUP12 :Data = 0x"8b";
const DUP13 :Data = 0x"8c";
const DUP14 :Data = 0x"8d";
const DUP15 :Data = 0x"8e";
const DUP16 :Data = 0x"8f";

const SWAP1 :Data = 0x"90";
const SWAP2 :Data = 0x"91";
const SWAP3 :Data = 0x"92";
const SWAP4 :Data = 0x"93";
const SWAP5 :Data = 0x"94";
const SWAP6 :Data = 0x"95";
const SWAP7 :Data = 0x"96";
const SWAP8 :Data = 0x"97";
const SWAP9 :Data = 0x"98";
const SWAP10 :Data = 0x"99";
const SWAP11 :Data = 0x"9a";
const SWAP12 :Data = 0x"9b";
const SWAP13 :Data = 0x"9c";
const SWAP14 :Data = 0x"9d";
const SWAP15 :Data = 0x"9e";
const SWAP16 :Data = 0x"9f";

const LOG0 :Data = 0x"a0";
const LOG1 :Data = 0x"a1";
const LOG2 :Data = 0x"a2";
const LOG3 :Data = 0x"a3";
const LOG4 :Data = 0x"a4";

const CREATE :Data = 0x"f0";
const CALL :Data = 0x"f1";
const CALLCODE :Data = 0x"f2";
const RETURN :Data = 0x"f3";
const DELEGATECALL :Data = 0x"f4";

const SELFDESTRUCT :Data = 0x"ff";

struct VMInput {
  gas @0 :Int32;
  code @1 :List(Data);
  data @2 :List(Data);
}
