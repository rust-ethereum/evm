use bigint::{Address, Gas, H160, U256};
use evm::{
    AccountPatch, Patch, Precompiled, ECREC_PRECOMPILED, ID_PRECOMPILED, RIP160_PRECOMPILED, SHA256_PRECOMPILED,
};

#[rustfmt::skip]
pub static ELLA_PRECOMPILEDS: [(Address, Option<&'static [u8]>, &'static Precompiled); 4] = [
    (H160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01]),
     None,
     &ECREC_PRECOMPILED),
    (H160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x02]),
     None,
     &SHA256_PRECOMPILED),
    (H160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x03]),
     None,
     &RIP160_PRECOMPILED),
    (H160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x04]),
     None,
     &ID_PRECOMPILED),
];

/// Mainnet account patch
#[derive(Debug, Copy, Clone, Default)]
pub struct MainnetAccountPatch;

#[rustfmt::skip]
impl AccountPatch for MainnetAccountPatch {
    fn initial_nonce(&self) -> U256 { U256::zero() }
    fn initial_create_nonce(&self) -> U256 { self.initial_nonce() }
    fn empty_considered_exists(&self) -> bool { true }
}

/// EIP160 patch.
#[derive(Debug, Copy, Clone, Default)]
pub struct EIP160Patch<A: AccountPatch>(A);
pub type MainnetEIP160Patch = EIP160Patch<MainnetAccountPatch>;

#[rustfmt::skip]
impl<A: AccountPatch> Patch for EIP160Patch<A> {
    type Account = A;

    fn account_patch(&self) -> &Self::Account { &self.0 }
    fn code_deposit_limit(&self) -> Option<usize> { None }
    fn callstack_limit(&self) -> usize { 1024 }
    fn gas_extcode(&self) -> Gas { Gas::from(700usize) }
    fn gas_balance(&self) -> Gas { Gas::from(400usize) }
    fn gas_sload(&self) -> Gas { Gas::from(200usize) }
    fn gas_suicide(&self) -> Gas { Gas::from(5000usize) }
    fn gas_suicide_new_account(&self) -> Gas { Gas::from(25000usize) }
    fn gas_call(&self) -> Gas { Gas::from(700usize) }
    fn gas_expbyte(&self) -> Gas { Gas::from(50usize) }
    fn gas_transaction_create(&self) -> Gas { Gas::from(32000usize) }
    fn force_code_deposit(&self) -> bool { false }
    fn has_delegate_call(&self) -> bool { true }
    fn has_static_call(&self) -> bool { false }
    fn has_revert(&self) -> bool { false }
    fn has_return_data(&self) -> bool { false }
    fn has_bitwise_shift(&self) -> bool { false }
    fn has_create2(&self) -> bool { false }
    fn has_extcodehash(&self) -> bool { false }
    fn has_reduced_sstore_gas_metering(&self) -> bool { false }
    fn err_on_call_with_more_gas(&self) -> bool { false }
    fn call_create_l64_after_gas(&self) -> bool { true }
    fn memory_limit(&self) -> usize { usize::max_value() }
    fn is_precompiled_contract_enabled(&self, address: &Address) -> bool {
        match address.low_u64() {
            0x1 | 0x2 | 0x3 | 0x4 => true,
            _ => false,
        }
    }
    fn precompileds(&self) -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
        &ELLA_PRECOMPILEDS
    }
}
