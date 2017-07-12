//! Patch of a VM, indicating different hard-fork of the Ethereum
//! block range.

/// Represents different block range context.
pub struct Patch {
    /// Limit of the call stack.
    pub callstack_limit: usize,
    /// Gas paid for extcode.
    pub gas_extcode: usize,
    /// Gas paid for BALANCE opcode.
    pub gas_balance: usize,
    /// Gas paid for SLOAD opcode.
    pub gas_sload: usize,
    /// Gas paid for SUICIDE opcode.
    pub gas_suicide: usize,
    /// Gas paid for SUICIDE opcode when it hits a new account.
    pub gas_suicide_new_account: usize,
    /// Gas paid for CALL opcode.
    pub gas_call: usize,
    /// Gas paid for EXP opcode for every byte.
    pub gas_expbyte: usize,
    /// Gas paid for a contract creation transaction.
    pub gas_transaction_create: usize,
    /// Whether to force code deposit even if it does not have enough
    /// gas.
    pub force_code_deposit: bool,
    /// Whether the EVM has DELEGATECALL opcode.
    pub has_delegate_call: bool,
    /// Whether to throw out of gas error when
    /// CALL/CALLCODE/DELEGATECALL requires more than maximum amount
    /// of gas.
    pub err_on_call_with_more_gas: bool,
    /// If true, only consume at maximum l64(after_gas) when
    /// CALL/CALLCODE/DELEGATECALL.
    pub call_create_l64_after_gas: bool,
}

/// Frontier patch.
pub static FRONTIER_PATCH: Patch = Patch {
    callstack_limit: 1024,
    gas_extcode: 20,
    gas_balance: 20,
    gas_sload: 50,
    gas_suicide: 0,
    gas_suicide_new_account: 0,
    gas_call: 40,
    gas_expbyte: 10,
    gas_transaction_create: 0,
    force_code_deposit: true,
    has_delegate_call: false,
    err_on_call_with_more_gas: true,
    call_create_l64_after_gas: false,
};

/// Homestead patch.
pub static HOMESTEAD_PATCH: Patch = Patch {
    callstack_limit: 1024,
    gas_extcode: 20,
    gas_balance: 20,
    gas_sload: 50,
    gas_suicide: 0,
    gas_suicide_new_account: 0,
    gas_call: 40,
    gas_expbyte: 10,
    gas_transaction_create: 32000,
    force_code_deposit: false,
    has_delegate_call: true,
    err_on_call_with_more_gas: true,
    call_create_l64_after_gas: false,
};

/// Patch specific for the `jsontests` crate.
pub static VMTEST_PATCH: Patch = Patch {
    callstack_limit: 2,
    gas_extcode: 20,
    gas_balance: 20,
    gas_sload: 50,
    gas_suicide: 0,
    gas_suicide_new_account: 0,
    gas_call: 40,
    gas_expbyte: 10,
    gas_transaction_create: 0,
    force_code_deposit: true,
    has_delegate_call: false,
    err_on_call_with_more_gas: true,
    call_create_l64_after_gas: false,
};

/// EIP150 patch.
pub static EIP150_PATCH: Patch = Patch {
    callstack_limit: 1024,
    gas_extcode: 700,
    gas_balance: 400,
    gas_sload: 200,
    gas_suicide: 5000,
    gas_suicide_new_account: 25000,
    gas_call: 700,
    gas_expbyte: 10,
    gas_transaction_create: 32000,
    force_code_deposit: false,
    has_delegate_call: true,
    err_on_call_with_more_gas: false,
    call_create_l64_after_gas: true,
};

/// EIP160 patch.
pub static EIP160_PATCH: Patch = Patch {
    callstack_limit: 1024,
    gas_extcode: 700,
    gas_balance: 400,
    gas_sload: 200,
    gas_suicide: 5000,
    gas_suicide_new_account: 25000,
    gas_call: 700,
    gas_expbyte: 50,
    gas_transaction_create: 32000,
    force_code_deposit: false,
    has_delegate_call: true,
    err_on_call_with_more_gas: false,
    call_create_l64_after_gas: true,
};
