use utils::u256::U256;
use utils::gas::Gas;
use super::Opcode;
use vm::{Machine, Memory, Stack, PC};

const G_ZERO: isize = 0;
const G_BASE: isize = 2;
const G_VERYLOW: isize = 3;
const G_LOW: isize = 5;
const G_MID: isize = 8;
const G_HIGH: isize = 10;
const G_EXTCODE: isize = 700;
const G_BALANCE: isize = 400;
const G_SLOAD: isize = 200;
const G_JUMPDEST: isize = 1;
const G_SSET: isize = 20000;
const G_SRESET: isize = 5000;
const R_SCLEAR: isize = 15000;
const R_SELFDESTRUCT: isize = 24000;
const G_SELFDESTRUCT: isize = 5000;
const G_CREATE: isize = 32000;
const G_CODEDEPOSITE: isize = 200;
const G_CALL: isize = 700;
const G_CALLVALUE: isize = 9000;
const G_CALLSTIPEND: isize = 2300;
const G_NEWACCOUNT: isize = 25000;
const G_EXP: isize = 10;
const G_EXPBYTE: isize = 10;
const G_MEMORY: isize = 3;
const G_TXCREATE: isize = 32000;
const G_TXDATAZERO: isize = 4;
const G_TXDATANONZERO: isize = 68;
const G_TRANSACTION: isize = 21000;
const G_LOG: isize = 375;
const G_LOGDATA: isize = 8;
const G_LOGTOPIC: isize = 375;
const G_SHA3: isize = 30;
const G_SHA3WORD: isize = 6;
const G_COPY: isize = 3;
const G_BLOCKHASH: isize = 20;

fn memory_cost(a: usize) -> Gas {
    let a = a as isize;
    (G_MEMORY * a + a * a / 512).into()
}

// TODO: Implement C_SSTORE, C_CALL and C_SELFDESTRUCT
// TODO: Use a machine_state struct instead of u_ip, u_i and u_s
// TODO: Use machine_state to implement gas cost for EXP,
// CALLDATACOPY, CODECOPY, EXTCODECOPY, LOG0-4, SHA3

impl Opcode {
    pub fn gas_cost_before<M: Machine>(&self, machine: &M) -> Gas {
        let ref stack = machine.stack();
        let ref memory = machine.memory();
        let opcode = self.clone();
        let self_cost: Gas = match opcode {
            // Unimplemented
            Opcode::SSTORE | Opcode::CALL | Opcode::CALLCODE |
            Opcode::DELEGATECALL | Opcode::SUICIDE =>
                unimplemented!(),

            Opcode::SHA3 => {
                let u_s1: u64 = (stack.peek(1)).into();
                (G_SHA3 + G_SHA3WORD * (u_s1 as isize / 32)).into()
            },

            Opcode::LOG(v) => {
                let u_s1: u64 = (stack.peek(1)).into();
                (G_LOG + G_LOGDATA * u_s1 as isize + (v as isize - 1) * G_LOGTOPIC).into()
            },

            Opcode::EXTCODECOPY => {
                // TODO: this value might exceed isize::max_value()
                let u_s3: u64 = (stack.peek(2)).into();
                (G_EXTCODE + G_COPY * (u_s3 as isize / 32)).into()
            },

            Opcode::CALLDATACOPY | Opcode::CODECOPY => {
                // TODO: this value might exceed isize::max_value()
                let u_s2: u64 = (stack.peek(2)).into();
                (G_VERYLOW + G_COPY * (u_s2 as isize / 32)).into()
            },

            Opcode::EXP => {
                if stack.peek(1) == U256::zero() {
                    G_EXP.into()
                } else {
                    (G_EXP + G_EXPBYTE * (1 + stack.peek(1).log2floor())).into()
                }
            }

            Opcode::CREATE => G_CREATE.into(),
            Opcode::JUMPDEST => G_JUMPDEST.into(),
            Opcode::SLOAD => G_SLOAD.into(),

            // W_zero
            Opcode::STOP | Opcode::RETURN
                => G_ZERO.into(),

            // W_base
            Opcode::ADDRESS | Opcode::ORIGIN | Opcode::CALLER |
            Opcode::CALLVALUE | Opcode::CALLDATASIZE |
            Opcode::CODESIZE | Opcode::GASPRICE | Opcode::COINBASE |
            Opcode::TIMESTAMP | Opcode::NUMBER | Opcode::DIFFICULTY |
            Opcode::GASLIMIT | Opcode::POP | Opcode::PC |
            Opcode::MSIZE | Opcode::GAS
                => G_BASE.into(),

            // W_verylow
            Opcode::ADD | Opcode::SUB | Opcode::NOT | Opcode::LT |
            Opcode::GT | Opcode::SLT | Opcode::SGT | Opcode::EQ |
            Opcode::ISZERO | Opcode::AND | Opcode::OR | Opcode::XOR |
            Opcode::BYTE | Opcode::CALLDATALOAD | Opcode::MLOAD |
            Opcode::MSTORE | Opcode::MSTORE8 | Opcode::PUSH(_) |
            Opcode::DUP(_) | Opcode::SWAP(_)
                => G_VERYLOW.into(),

            // W_low
            Opcode::MUL | Opcode::DIV | Opcode::SDIV | Opcode::MOD |
            Opcode::SMOD | Opcode::SIGNEXTEND
                => G_LOW.into(),

            // W_mid
            Opcode::ADDMOD | Opcode::MULMOD | Opcode::JUMP
                => G_MID.into(),

            // W_high
            Opcode::JUMPI => G_HIGH.into(),

            // W_extcode
            Opcode::EXTCODESIZE => G_EXTCODE.into(),

            Opcode::BALANCE => G_BALANCE.into(),
            Opcode::BLOCKHASH => G_BLOCKHASH.into(),
            Opcode::INVALID => Gas::zero(),
        };
        self_cost - memory_cost(machine.memory().active_len())
    }

    pub fn gas_cost_after<M: Machine>(&self, machine: &M) -> Gas {
        memory_cost(machine.memory().active_len())
    }
}
