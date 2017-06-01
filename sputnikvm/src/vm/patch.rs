pub struct Patch {
    pub callstack_limit: usize,
    pub gas_extcode: usize,
    pub gas_balance: usize,
    pub gas_sload: usize,
    pub gas_suicide: usize,
    pub gas_call: usize,
    pub gas_expbyte: usize,
}

pub static FRONTIER_PATCH: Patch = Patch {
    callstack_limit: 1024,
    gas_extcode: 20,
    gas_balance: 20,
    gas_sload: 50,
    gas_suicide: 0,
    gas_call: 40,
    gas_expbyte: 10,
};

pub static VMTEST_PATCH: Patch = Patch {
    callstack_limit: 2,
    gas_extcode: 20,
    gas_balance: 20,
    gas_sload: 50,
    gas_suicide: 0,
    gas_call: 40,
    gas_expbyte: 10,
};

pub static EIP150_PATCH: Patch = Patch {
    callstack_limit: 1024,
    gas_extcode: 700,
    gas_balance: 400,
    gas_sload: 200,
    gas_suicide: 5000,
    gas_call: 700,
    gas_expbyte: 10,
};

pub static EIP160_PATCH: Patch = Patch {
    callstack_limit: 1024,
    gas_extcode: 700,
    gas_balance: 400,
    gas_sload: 200,
    gas_suicide: 5000,
    gas_call: 700,
    gas_expbyte: 50,
};
