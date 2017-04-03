use super::Opcode;
use vm::{Machine, Memory, Stack, PC, Gas};

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
    pub fn gas_cost_before<M: Memory, S: Stack>(&self, machine: &Machine<M, S>) -> Gas {
        let opcode = self.clone();
        let self_cost: Gas = match opcode {
            // Unimplemented
            Opcode::SSTORE | Opcode::EXP | Opcode::CALLDATACOPY |
            Opcode::CODECOPY | Opcode::EXTCODECOPY | Opcode::LOG(_) |
            Opcode::CALL | Opcode::CALLCODE | Opcode::DELEGATECALL |
            Opcode::SELFDESTRUCT | Opcode::SHA3 | Opcode::EXTCODESIZE
                => unimplemented!(),

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

            Opcode::BALANCE => G_BALANCE.into(),
            Opcode::BLOCKHASH => G_BLOCKHASH.into(),
            Opcode::INVALID => Gas::zero(),
        };
        self_cost - memory_cost(machine.memory.active_len())
    }

    pub fn gas_cost_after<M: Memory, S: Stack>(&self, machine: &Machine<M, S>) -> Gas {
        memory_cost(machine.memory.active_len())
    }
}
