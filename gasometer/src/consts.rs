pub const G_ZERO: u64 = 0;
pub const G_BASE: u64 = 2;
pub const G_VERYLOW: u64 = 3;
pub const G_LOW: u64 = 5;
pub const G_MID: u64 = 8;
pub const G_HIGH: u64 = 10;
pub const G_JUMPDEST: u64 = 1;
pub const R_SUICIDE: i64 = 24000;
pub const G_CREATE: u64 = 32000;
pub const G_CALLVALUE: u64 = 9000;
pub const G_NEWACCOUNT: u64 = 25000;
pub const G_EXP: u64 = 10;
pub const G_MEMORY: u64 = 3;
pub const G_LOG: u64 = 375;
pub const G_LOGDATA: u64 = 8;
pub const G_LOGTOPIC: u64 = 375;
pub const G_SHA3: u64 = 30;
pub const G_SHA3WORD: u64 = 6;
pub const G_COPY: u64 = 3;
pub const G_BLOCKHASH: u64 = 20;
pub const G_CODEDEPOSIT: u64 = 200;

// EIP-7702 gas constants
pub const PER_AUTH_BASE_COST: u64 = 12500;
pub const PER_EMPTY_ACCOUNT_COST: u64 = 25000;
